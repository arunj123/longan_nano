#include "usbip_server.hpp"
#include <iostream>
#include <iomanip>
#include <vector>
#include <cstring>
#include <cerrno>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <poll.h>

namespace {

// Standard Linux errno values
constexpr int32_t ERR_SUCCESS = 0;
constexpr int32_t ERR_EPIPE   = -32; // Broken pipe / Endpoint Stalled

// Descriptors
#pragma pack(push, 1)
struct StandardDeviceDesc {
    uint8_t  bLength{18};
    uint8_t  bDescriptorType{1};
    uint16_t bcdUSB{0x0200};
    uint8_t  bDeviceClass{0};
    uint8_t  bDeviceSubClass{0};
    uint8_t  bDeviceProtocol{0};
    uint8_t  bMaxPacketSize0{64};
    uint16_t idVendor{0x28E9};
    uint16_t idProduct{0xAB39};
    uint16_t bcdDevice{0x0100};
    uint8_t  iManufacturer{1};
    uint8_t  iProduct{2};
    uint8_t  iSerialNumber{3};
    uint8_t  bNumberConfigurations{1};
};

struct UsbMscConfigDescSet {
    // Configuration Descriptor (9 bytes)
    uint8_t  cfg_bLength{9};
    uint8_t  cfg_bDescriptorType{2};
    uint16_t cfg_wTotalLength{32};
    uint8_t  cfg_bNumInterfaces{1};
    uint8_t  cfg_bConfigurationValue{1};
    uint8_t  cfg_iConfiguration{0};
    uint8_t  cfg_bmAttributes{0x80}; // Bus powered
    uint8_t  cfg_bMaxPower{50};      // 100 mA

    // Interface Descriptor (9 bytes)
    uint8_t  if_bLength{9};
    uint8_t  if_bDescriptorType{4};
    uint8_t  if_bInterfaceNumber{0};
    uint8_t  if_bAlternateSetting{0};
    uint8_t  if_bNumEndpoints{2};
    uint8_t  if_bInterfaceClass{0x08};    // Mass Storage
    uint8_t  if_bInterfaceSubClass{0x06}; // SCSI transparent
    uint8_t  if_bInterfaceProtocol{0x50}; // Bulk-Only Transport
    uint8_t  if_iInterface{0};

    // Bulk-OUT Endpoint (7 bytes)
    uint8_t  epout_bLength{7};
    uint8_t  epout_bDescriptorType{5};
    uint8_t  epout_bEndpointAddress{0x01}; // EP 1 OUT
    uint8_t  epout_bmAttributes{0x02};     // Bulk
    uint16_t epout_wMaxPacketSize{64};
    uint8_t  epout_bInterval{0};

    // Bulk-IN Endpoint (7 bytes)
    uint8_t  epin_bLength{7};
    uint8_t  epin_bDescriptorType{5};
    uint8_t  epin_bEndpointAddress{0x81};  // EP 1 IN
    uint8_t  epin_bmAttributes{0x02};      // Bulk
    uint16_t epin_wMaxPacketSize{64};
    uint8_t  epin_bInterval{0};
};
#pragma pack(pop)

static const StandardDeviceDesc  kDeviceDesc{};
static const UsbMscConfigDescSet kConfigDescSet{};

// String 0: Language ID 0x0409
static const uint8_t kStringLangId[] = { 0x04, 0x03, 0x09, 0x04 };

// String 1: "Sipeed" in UTF-16LE
static const uint8_t kStringMfr[] = {
    14, 0x03,
    'S', 0, 'i', 0, 'p', 0, 'e', 0, 'e', 0, 'd', 0
};

// String 2: "Longan Nano SD Reader" in UTF-16LE
static const uint8_t kStringProd[] = {
    44, 0x03,
    'L', 0, 'o', 0, 'n', 0, 'g', 0, 'a', 0, 'n', 0, ' ', 0,
    'N', 0, 'a', 0, 'n', 0, 'o', 0, ' ', 0, 'S', 0, 'D', 0,
    ' ', 0, 'R', 0, 'e', 0, 'a', 0, 'd', 0, 'e', 0, 'r', 0
};

// String 3: "LNMSC0000073" in UTF-16LE
static const uint8_t kStringSerial[] = {
    26, 0x03,
    'L', 0, 'N', 0, 'M', 0, 'S', 0, 'C', 0,
    '0', 0, '0', 0, '0', 0, '0', 0, '0', 0, '7', 0, '3', 0
};

bool read_exact(int fd, void* dest, size_t len) {
    uint8_t* ptr = reinterpret_cast<uint8_t*>(dest);
    size_t total = 0;
    while (total < len) {
        ssize_t ret = recv(fd, ptr + total, len - total, 0);
        if (ret <= 0) {
            return false;
        }
        total += ret;
    }
    return true;
}

bool write_exact(int fd, const void* src, size_t len) {
    const uint8_t* ptr = reinterpret_cast<const uint8_t*>(src);
    size_t total = 0;
    while (total < len) {
        ssize_t ret = send(fd, ptr + total, len - total, 0);
        if (ret <= 0) {
            return false;
        }
        total += ret;
    }
    return true;
}

} // namespace

UsbipServer::UsbipServer(VirtualDisk& disk, uint16_t port)
    : disk_(disk), engine_(disk), port_(port) {}

UsbipServer::~UsbipServer() {
    stop();
}

void UsbipServer::stop() noexcept {
    running_ = false;
    if (server_fd_ >= 0) {
        close(server_fd_);
        server_fd_ = -1;
    }
}

bool UsbipServer::run() {
    server_fd_ = socket(AF_INET, SOCK_STREAM, 0);
    if (server_fd_ < 0) {
        std::cerr << "[USBIP] Failed to create socket: " << strerror(errno) << "\n";
        return false;
    }

    int opt = 1;
    setsockopt(server_fd_, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));

    sockaddr_in addr{};
    addr.sin_family      = AF_INET;
    addr.sin_addr.s_addr = INADDR_ANY;
    addr.sin_port        = htons(port_);

    if (bind(server_fd_, reinterpret_cast<sockaddr*>(&addr), sizeof(addr)) < 0) {
        std::cerr << "[USBIP] Failed to bind port " << port_ << ": " << strerror(errno) << "\n";
        close(server_fd_);
        server_fd_ = -1;
        return false;
    }

    if (listen(server_fd_, 5) < 0) {
        std::cerr << "[USBIP] Listen failed: " << strerror(errno) << "\n";
        close(server_fd_);
        server_fd_ = -1;
        return false;
    }

    running_ = true;
    std::cout << "[USBIP] Server listening on port " << port_ << "...\n";

    while (running_) {
        pollfd pfd{server_fd_, POLLIN, 0};
        int pr = poll(&pfd, 1, 1000);
        if (pr <= 0) continue;

        sockaddr_in client_addr{};
        socklen_t client_len = sizeof(client_addr);
        int client_fd = accept(server_fd_, reinterpret_cast<sockaddr*>(&client_addr), &client_len);
        if (client_fd < 0) {
            if (running_) std::cerr << "[USBIP] Accept failed: " << strerror(errno) << "\n";
            continue;
        }

        int nodelay = 1;
        setsockopt(client_fd, IPPROTO_TCP, TCP_NODELAY, &nodelay, sizeof(nodelay));

        char ip_str[INET_ADDRSTRLEN];
        inet_ntop(AF_INET, &client_addr.sin_addr, ip_str, sizeof(ip_str));
        std::cout << "[USBIP] Connection accepted from " << ip_str << "\n";

        handle_client(client_fd);
        close(client_fd);
        std::cout << "[USBIP] Client disconnected\n";
    }

    return true;
}

bool UsbipServer::handle_client(int client_fd) {
    while (running_) {
        usbip::OpHeader req_hdr{};
        if (!read_exact(client_fd, &req_hdr, sizeof(req_hdr))) {
            return false;
        }

        req_hdr.version = usbip::from_be(req_hdr.version);
        req_hdr.code    = usbip::from_be(req_hdr.code);
        req_hdr.status  = usbip::from_be(req_hdr.status);

        if (req_hdr.code == usbip::OP_REQ_DEVLIST) {
            std::cout << "[USBIP] Received OP_REQ_DEVLIST\n";
            if (!handle_devlist(client_fd, req_hdr)) return false;
            // DEVLIST connection closes immediately per protocol
            return true;
        } else if (req_hdr.code == usbip::OP_REQ_IMPORT) {
            std::cout << "[USBIP] Received OP_REQ_IMPORT\n";
            if (!handle_import(client_fd, req_hdr)) return false;
            // IMPORT transitions to URB loop on the same socket
            return handle_urb_loop(client_fd);
        } else {
            std::cerr << "[USBIP] Unknown OP code: 0x" << std::hex << req_hdr.code << std::dec << "\n";
            return false;
        }
    }
    return true;
}

bool UsbipServer::handle_devlist(int client_fd, const usbip::OpHeader& /*req_hdr*/) {
    usbip::OpDevlistReplyHeader rep_hdr{};
    rep_hdr.base.version = usbip::to_be(usbip::USBIP_VERSION);
    rep_hdr.base.code    = usbip::to_be(usbip::OP_REP_DEVLIST);
    rep_hdr.base.status  = usbip::to_be(0U);
    rep_hdr.ndev         = usbip::to_be(1U);

    usbip::UsbDeviceDesc dev{};
    std::strncpy(dev.path, "/sys/devices/virtual/usb0/1-1", sizeof(dev.path));
    std::strncpy(dev.busid, "1-1", sizeof(dev.busid));
    dev.busnum              = usbip::to_be(1U);
    dev.devnum              = usbip::to_be(2U);
    dev.speed               = usbip::to_be(2U); // High-speed (or 3 full-speed)
    dev.idVendor            = usbip::to_be(kDeviceDesc.idVendor);
    dev.idProduct           = usbip::to_be(kDeviceDesc.idProduct);
    dev.bcdDevice           = usbip::to_be(kDeviceDesc.bcdDevice);
    dev.bDeviceClass        = kDeviceDesc.bDeviceClass;
    dev.bDeviceSubClass     = kDeviceDesc.bDeviceSubClass;
    dev.bDeviceProtocol     = kDeviceDesc.bDeviceProtocol;
    dev.bConfigurationValue = kConfigDescSet.cfg_bConfigurationValue;
    dev.bNumConfigurations  = kDeviceDesc.bNumberConfigurations;
    dev.bNumInterfaces      = kConfigDescSet.cfg_bNumInterfaces;

    usbip::UsbInterfaceDesc intf{};
    intf.bInterfaceClass    = kConfigDescSet.if_bInterfaceClass;
    intf.bInterfaceSubClass = kConfigDescSet.if_bInterfaceSubClass;
    intf.bInterfaceProtocol = kConfigDescSet.if_bInterfaceProtocol;
    intf.padding            = 0;

    if (!write_exact(client_fd, &rep_hdr, sizeof(rep_hdr))) return false;
    if (!write_exact(client_fd, &dev, sizeof(dev))) return false;
    if (!write_exact(client_fd, &intf, sizeof(intf))) return false;

    return true;
}

bool UsbipServer::handle_import(int client_fd, const usbip::OpHeader& /*req_hdr*/) {
    char busid[usbip::SYSFS_BUS_ID_SIZE]{};
    if (!read_exact(client_fd, busid, sizeof(busid))) return false;

    std::cout << "[USBIP] Importing busid: " << busid << "\n";

    usbip::OpImportReply rep{};
    rep.base.version = usbip::to_be(usbip::USBIP_VERSION);
    rep.base.code    = usbip::to_be(usbip::OP_REP_IMPORT);
    rep.base.status  = usbip::to_be(0U);

    std::strncpy(rep.udev.path, "/sys/devices/virtual/usb0/1-1", sizeof(rep.udev.path));
    std::strncpy(rep.udev.busid, "1-1", sizeof(rep.udev.busid));
    rep.udev.busnum              = usbip::to_be(1U);
    rep.udev.devnum              = usbip::to_be(2U);
    rep.udev.speed               = usbip::to_be(2U);
    rep.udev.idVendor            = usbip::to_be(kDeviceDesc.idVendor);
    rep.udev.idProduct           = usbip::to_be(kDeviceDesc.idProduct);
    rep.udev.bcdDevice           = usbip::to_be(kDeviceDesc.bcdDevice);
    rep.udev.bDeviceClass        = kDeviceDesc.bDeviceClass;
    rep.udev.bDeviceSubClass     = kDeviceDesc.bDeviceSubClass;
    rep.udev.bDeviceProtocol     = kDeviceDesc.bDeviceProtocol;
    rep.udev.bConfigurationValue = kConfigDescSet.cfg_bConfigurationValue;
    rep.udev.bNumConfigurations  = kDeviceDesc.bNumberConfigurations;
    rep.udev.bNumInterfaces      = kConfigDescSet.cfg_bNumInterfaces;

    return write_exact(client_fd, &rep, sizeof(rep));
}

bool UsbipServer::handle_urb_loop(int client_fd) {
    std::cout << "[USBIP] Entered URB submission loop\n";
    engine_.reset();

    while (running_) {
        usbip::UsbipPacketHeader cmd_hdr{};
        if (!read_exact(client_fd, &cmd_hdr, sizeof(cmd_hdr))) {
            std::cout << "[USBIP] Connection closed or read error in URB loop\n";
            return false;
        }

        cmd_hdr.swap_from_network();

        if (cmd_hdr.base.command == usbip::USBIP_CMD_SUBMIT) {
            auto& submit = cmd_hdr.u.cmd_submit;
            submit.transfer_flags         = usbip::from_be(submit.transfer_flags);
            submit.transfer_buffer_length = usbip::from_be(submit.transfer_buffer_length);

            std::cout << "[USBIP] CMD_SUBMIT: seq=" << cmd_hdr.base.seqnum
                      << " ep=" << cmd_hdr.base.ep
                      << " dir=" << (cmd_hdr.base.direction == usbip::DIR_IN ? "IN" : "OUT")
                      << " len=" << submit.transfer_buffer_length;

            if (cmd_hdr.base.ep == 0) {
                const uint8_t* s = submit.setup;
                std::cout << " [SETUP: " << std::hex << std::setfill('0')
                          << "bmReq=0x" << std::setw(2) << (int)s[0]
                          << " bReq=0x"  << std::setw(2) << (int)s[1]
                          << " wVal=0x"  << std::setw(4) << (s[2] | (s[3] << 8))
                          << " wIdx=0x"  << std::setw(4) << (s[4] | (s[5] << 8))
                          << " wLen="    << std::dec << (s[6] | (s[7] << 8)) << "]\n";
                handle_ep0_control(client_fd, cmd_hdr);
            } else if (cmd_hdr.base.ep == 1) {
                std::cout << "\n";
                if (cmd_hdr.base.direction == usbip::DIR_OUT) {
                    handle_ep1_bulk_out(client_fd, cmd_hdr);
                } else {
                    handle_ep1_bulk_in(client_fd, cmd_hdr);
                }
            } else {
                std::cout << " [UNEXPECTED EP]\n";
                std::cerr << "[USBIP] Unexpected EP: " << cmd_hdr.base.ep << "\n";
                send_ret_submit(client_fd, cmd_hdr, ERR_EPIPE, {});
            }
        } else if (cmd_hdr.base.command == usbip::USBIP_CMD_UNLINK) {
            std::cout << "[USBIP] Unlink URB seqnum: " << cmd_hdr.base.seqnum << "\n";
            usbip::UsbipPacketHeader ret_hdr{};
            ret_hdr.base.command = usbip::USBIP_RET_UNLINK;
            ret_hdr.base.seqnum  = cmd_hdr.base.seqnum;
            ret_hdr.u.ret_unlink.status = 0;
            ret_hdr.swap_to_network();
            write_exact(client_fd, &ret_hdr, sizeof(ret_hdr));
        } else {
            std::cerr << "[USBIP] Unknown command: 0x" << std::hex << cmd_hdr.base.command << std::dec << "\n";
            return false;
        }
    }
    return true;
}

void UsbipServer::handle_ep0_control(int client_fd, const usbip::UsbipPacketHeader& cmd_hdr) {
    const uint8_t* setup = cmd_hdr.u.cmd_submit.setup;
    uint8_t  bmRequestType = setup[0];
    uint8_t  bRequest      = setup[1];
    uint16_t wValue        = setup[2] | (static_cast<uint16_t>(setup[3]) << 8);
    uint16_t wIndex        = setup[4] | (static_cast<uint16_t>(setup[5]) << 8);
    uint16_t wLength       = setup[6] | (static_cast<uint16_t>(setup[7]) << 8);

    // If control OUT transfer includes data payload, drain it
    if (cmd_hdr.base.direction == usbip::DIR_OUT && cmd_hdr.u.cmd_submit.transfer_buffer_length > 0) {
        std::vector<uint8_t> out_payload(cmd_hdr.u.cmd_submit.transfer_buffer_length);
        read_exact(client_fd, out_payload.data(), out_payload.size());
    }

    // Standard Request: GET_STATUS (0x00)
    if ((bmRequestType & 0x7F) == 0x00 && bRequest == 0x00) {
        uint16_t status_word = 0;
        uint8_t recip = bmRequestType & 0x1F;
        if (recip == 0x02) { // Endpoint
            uint8_t ep = static_cast<uint8_t>(wIndex);
            if (ep & 0x80) {
                status_word = engine_.is_stall_in() ? 1 : 0;
            } else {
                status_word = engine_.is_stall_out() ? 1 : 0;
            }
        }
        size_t send_len = std::min<size_t>(wLength, sizeof(status_word));
        send_ret_submit(client_fd, cmd_hdr, ERR_SUCCESS,
                        std::span<const uint8_t>(reinterpret_cast<const uint8_t*>(&status_word), send_len));
        return;
    }

    // Standard Request: GET_DESCRIPTOR (0x06)
    if (bmRequestType == 0x80 && bRequest == 0x06) {
        uint8_t desc_type  = static_cast<uint8_t>(wValue >> 8);
        uint8_t desc_index = static_cast<uint8_t>(wValue & 0xFF);

        if (desc_type == 0x01) { // DEVICE
            size_t send_len = std::min<size_t>(wLength, sizeof(kDeviceDesc));
            send_ret_submit(client_fd, cmd_hdr, ERR_SUCCESS, 
                            std::span<const uint8_t>(reinterpret_cast<const uint8_t*>(&kDeviceDesc), send_len));
            return;
        } else if (desc_type == 0x02) { // CONFIGURATION
            size_t send_len = std::min<size_t>(wLength, sizeof(kConfigDescSet));
            send_ret_submit(client_fd, cmd_hdr, ERR_SUCCESS,
                            std::span<const uint8_t>(reinterpret_cast<const uint8_t*>(&kConfigDescSet), send_len));
            return;
        } else if (desc_type == 0x03) { // STRING
            std::span<const uint8_t> str_span{};
            if (desc_index == 0) str_span = kStringLangId;
            else if (desc_index == 1) str_span = kStringMfr;
            else if (desc_index == 2) str_span = kStringProd;
            else if (desc_index == 3) str_span = kStringSerial;

            if (!str_span.empty()) {
                size_t send_len = std::min<size_t>(wLength, str_span.size());
                send_ret_submit(client_fd, cmd_hdr, ERR_SUCCESS, str_span.subspan(0, send_len));
                return;
            }
        }
        send_ret_submit(client_fd, cmd_hdr, ERR_EPIPE, {});
        return;
    }

    // Standard Request: SET_CONFIGURATION (0x09)
    if (bmRequestType == 0x00 && bRequest == 0x09) {
        std::cout << "[USBIP] SET_CONFIGURATION: " << wValue << "\n";
        send_ret_submit(client_fd, cmd_hdr, ERR_SUCCESS, {});
        return;
    }

    // Standard Request: GET_CONFIGURATION (0x08)
    if (bmRequestType == 0x80 && bRequest == 0x08) {
        static const uint8_t cfg_val = 1;
        send_ret_submit(client_fd, cmd_hdr, ERR_SUCCESS, std::span<const uint8_t>(&cfg_val, 1));
        return;
    }

    // Standard Request: CLEAR_FEATURE(ENDPOINT_HALT) (0x01)
    if ((bmRequestType == 0x02) && (bRequest == 0x01) && (wValue == 0)) {
        uint8_t ep = static_cast<uint8_t>(wIndex);
        std::cout << "[USBIP] CLEAR_FEATURE(HALT) for EP: 0x" << std::hex << (int)ep << std::dec << "\n";
        engine_.clear_stall(ep);
        send_ret_submit(client_fd, cmd_hdr, ERR_SUCCESS, {});
        return;
    }

    // Standard Request: SET_FEATURE(ENDPOINT_HALT) (0x03)
    if ((bmRequestType == 0x02) && (bRequest == 0x03) && (wValue == 0)) {
        uint8_t ep = static_cast<uint8_t>(wIndex);
        std::cout << "[USBIP] SET_FEATURE(HALT) for EP: 0x" << std::hex << (int)ep << std::dec << "\n";
        send_ret_submit(client_fd, cmd_hdr, ERR_SUCCESS, {});
        return;
    }

    // Standard Request: GET_INTERFACE (0x0A)
    if ((bmRequestType == 0x81) && (bRequest == 0x0A)) {
        static const uint8_t alt_setting = 0;
        send_ret_submit(client_fd, cmd_hdr, ERR_SUCCESS, std::span<const uint8_t>(&alt_setting, 1));
        return;
    }

    // Standard Request: SET_INTERFACE (0x0B)
    if ((bmRequestType == 0x01) && (bRequest == 0x0B)) {
        send_ret_submit(client_fd, cmd_hdr, ERR_SUCCESS, {});
        return;
    }

    // Class Request: GET_MAX_LUN (0xFE)
    if ((bmRequestType == 0xA1) && (bRequest == 0xFE)) {
        static const uint8_t max_lun = 0; // LUN 0
        send_ret_submit(client_fd, cmd_hdr, ERR_SUCCESS, std::span<const uint8_t>(&max_lun, 1));
        return;
    }

    // Class Request: BULK_ONLY_RESET (0xFF)
    if ((bmRequestType == 0x21) && (bRequest == 0xFF)) {
        std::cout << "[USBIP] Bulk-Only Reset\n";
        engine_.reset();
        send_ret_submit(client_fd, cmd_hdr, ERR_SUCCESS, {});
        return;
    }

    // Unhandled control request
    std::cout << "[USBIP] Unhandled control: bmReq=0x" << std::hex << (int)bmRequestType
              << " bReq=0x" << (int)bRequest << " wVal=0x" << wValue << std::dec << "\n";
    send_ret_submit(client_fd, cmd_hdr, ERR_EPIPE, {});
}

void UsbipServer::handle_ep1_bulk_out(int client_fd, const usbip::UsbipPacketHeader& cmd_hdr) {
    const int32_t len = cmd_hdr.u.cmd_submit.transfer_buffer_length;
    if (len <= 0) {
        send_ret_submit(client_fd, cmd_hdr, ERR_SUCCESS, {});
        return;
    }

    std::vector<uint8_t> payload(len);
    if (!read_exact(client_fd, payload.data(), len)) {
        std::cerr << "[USBIP] Failed to read OUT payload of " << len << " bytes\n";
        return;
    }

    if (engine_.is_waiting_data_out()) {
        engine_.on_data_out(payload);
    } else if (len == msc::BBB_CBW_LENGTH) {
        engine_.on_cbw(payload);
    } else {
        std::cerr << "[USBIP] Unexpected Bulk-OUT payload length: " << len << "\n";
    }

    int32_t status = engine_.is_stall_out() ? ERR_EPIPE : ERR_SUCCESS;
    send_ret_submit(client_fd, cmd_hdr, status, {});
}

void UsbipServer::handle_ep1_bulk_in(int client_fd, const usbip::UsbipPacketHeader& cmd_hdr) {
    const int32_t requested_len = cmd_hdr.u.cmd_submit.transfer_buffer_length;

    if (engine_.is_stall_in()) {
        send_ret_submit(client_fd, cmd_hdr, ERR_EPIPE, {});
        return;
    }

    if (engine_.has_pending_in_data()) {
        auto data_slice = engine_.get_in_data(requested_len);
        send_ret_submit(client_fd, cmd_hdr, ERR_SUCCESS, data_slice);
        engine_.consume_in_data(data_slice.size());
    } else if (engine_.is_waiting_csw()) {
        auto csw_slice = engine_.get_csw();
        send_ret_submit(client_fd, cmd_hdr, ERR_SUCCESS, csw_slice);
    } else {
        // Zero-length packet or idle
        send_ret_submit(client_fd, cmd_hdr, ERR_SUCCESS, {});
    }
}

bool UsbipServer::send_ret_submit(int client_fd, const usbip::UsbipPacketHeader& cmd_hdr,
                                 int32_t status, std::span<const uint8_t> data) {
    usbip::UsbipPacketHeader ret_hdr{};
    ret_hdr.base.command   = usbip::USBIP_RET_SUBMIT;
    ret_hdr.base.seqnum    = cmd_hdr.base.seqnum;
    ret_hdr.base.devid     = 0;
    ret_hdr.base.direction = 0;
    ret_hdr.base.ep        = 0;

    int32_t actual_len = 0;
    if (cmd_hdr.base.direction == usbip::DIR_IN) {
        actual_len = static_cast<int32_t>(data.size());
    } else {
        actual_len = (status == ERR_SUCCESS) ? cmd_hdr.u.cmd_submit.transfer_buffer_length : 0;
    }

    ret_hdr.u.ret_submit.status            = usbip::to_be(status);
    ret_hdr.u.ret_submit.actual_length     = usbip::to_be(actual_len);
    ret_hdr.u.ret_submit.start_frame       = 0;
    ret_hdr.u.ret_submit.number_of_packets = 0;
    ret_hdr.u.ret_submit.error_count       = 0;

    ret_hdr.swap_to_network();

    if (!write_exact(client_fd, &ret_hdr, sizeof(ret_hdr))) {
        return false;
    }

    if (!data.empty() && actual_len > 0) {
        if (!write_exact(client_fd, data.data(), actual_len)) {
            return false;
        }
    }

    return true;
}
