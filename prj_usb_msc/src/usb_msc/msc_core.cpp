#include "msc_core.hpp"
#include "msc_disk.hpp"
#include "usbd_descriptors.hpp"
#include "drivers/usb/drv_usbd_int.h"
#include <cstring>
#include <cstdio>

namespace msc {

// BBB (Bulk-Only Transport) Constants & Enums
constexpr uint32_t BBB_CBW_SIGNATURE = 0x43425355U; // "USBC"
constexpr uint32_t BBB_CSW_SIGNATURE = 0x53425355U; // "USBS"
constexpr uint8_t  BBB_CBW_LENGTH    = 31U;
constexpr uint8_t  BBB_CSW_LENGTH    = 13U;

constexpr uint8_t  REQ_GET_MAX_LUN   = 0xFEU;
constexpr uint8_t  REQ_BBB_RESET     = 0xFFU;

enum class BbbState : uint8_t {
    IDLE = 0,
    DATA_OUT,
    DATA_IN,
    LAST_DATA_IN,
    SEND_DATA
};

enum class BbbStatus : uint8_t {
    NORMAL = 0,
    RECOVERY,
    ERROR
};

enum class CswStatus : uint8_t {
    CMD_PASSED  = 0x00,
    CMD_FAILED  = 0x01,
    PHASE_ERROR = 0x02
};

// SCSI Commands
enum class ScsiCmd : uint8_t {
    TEST_UNIT_READY        = 0x00,
    REQUEST_SENSE          = 0x03,
    READ_6                 = 0x08,
    WRITE_6                = 0x0A,
    INQUIRY                = 0x12,
    MODE_SENSE_6           = 0x1A,
    START_STOP_UNIT        = 0x1B,
    PREVENT_ALLOW_REMOVAL  = 0x1E,
    READ_FORMAT_CAPACITIES = 0x23,
    READ_CAPACITY_10       = 0x25,
    READ_10                = 0x28,
    WRITE_10               = 0x2A,
    VERIFY_10              = 0x2F,
    SYNCHRONIZE_CACHE_10   = 0x35,
    MODE_SENSE_10          = 0x5A,
    SERVICE_ACTION_IN_16   = 0x9E,
    SECURITY_PROTOCOL_IN   = 0xA2
};

// SCSI Sense Keys & ASC
enum class SenseKey : uint8_t {
    NO_SENSE        = 0x00,
    RECOVERED_ERROR = 0x01,
    NOT_READY       = 0x02,
    MEDIUM_ERROR    = 0x03,
    HARDWARE_ERROR  = 0x04,
    ILLEGAL_REQUEST = 0x05,
    UNIT_ATTENTION  = 0x06,
    DATA_PROTECT    = 0x07
};

enum class Asc : uint8_t {
    NO_ASC                   = 0x00,
    UNRECOVERED_READ_ERROR   = 0x11,
    WRITE_FAULT              = 0x03,
    INVALID_CDB              = 0x20,
    ADDRESS_OUT_OF_RANGE     = 0x21,
    INVALID_FIELD_IN_COMMAND = 0x24,
    WRITE_PROTECTED          = 0x27,
    MEDIUM_NOT_PRESENT       = 0x3A
};

#pragma pack(push, 1)
struct BbbCbw {
    uint32_t dCBWSignature;
    uint32_t dCBWTag;
    uint32_t dCBWDataTransferLength;
    uint8_t  bmCBWFlags;
    uint8_t  bCBWLUN;
    uint8_t  bCBWCBLength;
    uint8_t  CBWCB[16];
};

struct BbbCsw {
    uint32_t dCSWSignature;
    uint32_t dCSWTag;
    uint32_t dCSWDataResidue;
    uint8_t  bCSWStatus;
};
#pragma pack(pop)

struct MscContext {
    alignas(4) uint8_t media_buffer[MSC_MEDIA_PACKET_SIZE];
    alignas(4) uint8_t cbw_buf[32]; // Aligned to 4 bytes, padded to 32 bytes for 8-word FIFO read
    alignas(4) uint8_t csw_buf[16]; // Aligned to 4 bytes, padded to 16 bytes for 4-word FIFO write
    BbbState state{BbbState::IDLE};
    BbbStatus status{BbbStatus::NORMAL};
    uint32_t data_len{0};
    uint32_t lba{0};
    uint32_t total_blocks{0};
    uint32_t remaining_bytes{0};
    SenseKey sense_key{SenseKey::NO_SENSE};
    Asc sense_asc{Asc::NO_ASC};
    uint8_t max_lun{0};
    volatile bool read_pending{false};
    volatile bool write_pending{false};

    inline BbbCbw& cbw() { return *reinterpret_cast<BbbCbw*>(cbw_buf); }
    inline const BbbCbw& cbw() const { return *reinterpret_cast<const BbbCbw*>(cbw_buf); }
    inline BbbCsw& csw() { return *reinterpret_cast<BbbCsw*>(csw_buf); }
    inline const BbbCsw& csw() const { return *reinterpret_cast<const BbbCsw*>(csw_buf); }
};

static MscContext ctx;

MscTraceEntry g_msc_trace[MSC_TRACE_MAX] = {};
volatile uint8_t g_msc_trace_head = 0;
volatile uint8_t g_msc_trace_tail = 0;

void msc_trace_record(uint8_t type, uint8_t opcode, uint8_t val8, uint8_t status, uint32_t val32_1, uint32_t val32_2, const uint8_t *cdb) {
    uint8_t next_head = (g_msc_trace_head + 1U) % MSC_TRACE_MAX;
    if (next_head != g_msc_trace_tail) {
        auto &entry = g_msc_trace[g_msc_trace_head];
        entry = {type, opcode, val8, status, val32_1, val32_2, {}};
        if (cdb) {
            std::memcpy(entry.cdb, cdb, 10);
        }
        g_msc_trace_head = next_head;
    }
}

alignas(4) static const uint8_t kInquiryData[36] = {
    0x00,                   // Direct access block device
    0x80,                   // Removable medium
    0x02,                   // ANSI SCSI-2
    0x02,                   // Response data format
    31,                     // Additional length (36 - 5)
    0x00, 0x00, 0x00,       // Reserved
    'S', 'i', 'p', 'e', 'e', 'd', ' ', ' ',         // Vendor: 8 bytes
    'L', 'o', 'n', 'g', 'a', 'n', ' ', 'N', 'a', 'n', 'o', ' ', 'S', 'D', ' ', ' ', // Product: 16 bytes
    '1', '.', '0', '0'                              // Revision: 4 bytes
};

static void set_sense(SenseKey key, Asc asc) {
    ctx.sense_key = key;
    ctx.sense_asc = asc;
}

static void csw_send(usb_core_driver *udev, CswStatus status) {
    auto &csw = ctx.csw();
    auto &cbw = ctx.cbw();
    csw.dCSWSignature = BBB_CSW_SIGNATURE;
    csw.dCSWTag = cbw.dCBWTag;
    csw.dCSWDataResidue = ctx.remaining_bytes;
    csw.bCSWStatus = static_cast<uint8_t>(status);
    ctx.state = BbbState::IDLE;
    ctx.read_pending = false;
    ctx.write_pending = false;

    msc_trace_record(3, cbw.CBWCB[0], static_cast<uint8_t>(status), 0, csw.dCSWDataResidue, 0);

    usbd_ep_send(udev, MSC_IN_EP, ctx.csw_buf, BBB_CSW_LENGTH);
    usbd_ep_recev(udev, MSC_OUT_EP, ctx.cbw_buf, BBB_CBW_LENGTH);
}

static void data_send(usb_core_driver *udev, uint8_t *pbuf, uint32_t len) {
    len = USB_MIN(ctx.remaining_bytes, len);
    ctx.remaining_bytes -= len;
    ctx.csw().dCSWDataResidue = ctx.remaining_bytes;
    ctx.state = BbbState::SEND_DATA;
    usbd_ep_send(udev, MSC_IN_EP, pbuf, len);
}

static void abort_transfer(usb_core_driver *udev) {
    auto &cbw = ctx.cbw();
    ctx.remaining_bytes = cbw.dCBWDataTransferLength;
    ctx.read_pending = false;
    ctx.write_pending = false;
    if (cbw.dCBWDataTransferLength == 0) {
        csw_send(udev, CswStatus::CMD_FAILED);
        return;
    }
    if ((cbw.bmCBWFlags == 0) && (ctx.status == BbbStatus::NORMAL)) {
        usbd_ep_stall(udev, MSC_OUT_EP);
    }
    usbd_ep_stall(udev, MSC_IN_EP);
    if (ctx.status == BbbStatus::ERROR) {
        usbd_ep_recev(udev, MSC_OUT_EP, ctx.cbw_buf, BBB_CBW_LENGTH);
    }
}

static int8_t process_scsi(usb_core_driver *udev) {
    auto &cbw = ctx.cbw();
    const uint8_t *cmd = cbw.CBWCB;
    auto opcode = static_cast<ScsiCmd>(cmd[0]);

    switch (opcode) {
        case ScsiCmd::TEST_UNIT_READY: {
            if (cbw.dCBWDataTransferLength != 0) {
                set_sense(SenseKey::ILLEGAL_REQUEST, Asc::INVALID_CDB);
                return -1;
            }
            if (!msc_disk_ready()) {
                set_sense(SenseKey::NOT_READY, Asc::MEDIUM_NOT_PRESENT);
                return -1;
            }
            ctx.data_len = 0;
            return 0;
        }

        case ScsiCmd::REQUEST_SENSE: {
            std::memset(ctx.media_buffer, 0, 18);
            ctx.media_buffer[0] = 0x70; // Current error
            ctx.media_buffer[2] = static_cast<uint8_t>(ctx.sense_key);
            ctx.media_buffer[7] = 10;   // Additional length = 18 - 8
            ctx.media_buffer[12] = static_cast<uint8_t>(ctx.sense_asc);

            // Reset sense after reporting
            ctx.sense_key = SenseKey::NO_SENSE;
            ctx.sense_asc = Asc::NO_ASC;

            ctx.data_len = USB_MIN(cbw.dCBWDataTransferLength, 18U);
            return 0;
        }

        case ScsiCmd::READ_6: {
            if ((cbw.bmCBWFlags & 0x80) == 0 || !msc_disk_ready()) {
                set_sense(SenseKey::ILLEGAL_REQUEST, Asc::INVALID_CDB);
                return -1;
            }
            ctx.lba = ((static_cast<uint32_t>(cmd[1]) & 0x1F) << 16) |
                      (static_cast<uint32_t>(cmd[2]) << 8) |
                      static_cast<uint32_t>(cmd[3]);
            uint32_t blocks = (cmd[4] == 0) ? 256 : cmd[4];
            if (blocks == 0 && cbw.dCBWDataTransferLength > 0) {
                blocks = cbw.dCBWDataTransferLength / 512;
            }
            uint32_t blk_cnt = 0, blk_sz = 512;
            msc_disk_get_capacity(blk_cnt, blk_sz);
            if (ctx.lba + blocks > blk_cnt) {
                set_sense(SenseKey::ILLEGAL_REQUEST, Asc::ADDRESS_OUT_OF_RANGE);
                return -1;
            }
            ctx.state = BbbState::DATA_IN;
            ctx.remaining_bytes = blocks * 512;
            ctx.read_pending = true;
            return 0;
        }

        case ScsiCmd::WRITE_6: {
            if ((cbw.bmCBWFlags & 0x80) != 0 || !msc_disk_ready()) {
                set_sense(SenseKey::ILLEGAL_REQUEST, Asc::INVALID_CDB);
                return -1;
            }
            ctx.lba = ((static_cast<uint32_t>(cmd[1]) & 0x1F) << 16) |
                      (static_cast<uint32_t>(cmd[2]) << 8) |
                      static_cast<uint32_t>(cmd[3]);
            uint32_t blocks = (cmd[4] == 0) ? 256 : cmd[4];
            if (blocks == 0 && cbw.dCBWDataTransferLength > 0) {
                blocks = cbw.dCBWDataTransferLength / 512;
            }
            uint32_t blk_cnt = 0, blk_sz = 512;
            msc_disk_get_capacity(blk_cnt, blk_sz);
            if (ctx.lba + blocks > blk_cnt) {
                set_sense(SenseKey::ILLEGAL_REQUEST, Asc::ADDRESS_OUT_OF_RANGE);
                return -1;
            }
            ctx.state = BbbState::DATA_OUT;
            ctx.remaining_bytes = blocks * 512;
            uint32_t rx_len = USB_MIN(ctx.remaining_bytes, MSC_MEDIA_PACKET_SIZE);
            usbd_ep_recev(udev, MSC_OUT_EP, ctx.media_buffer, rx_len);
            return 0;
        }

        case ScsiCmd::INQUIRY: {
            // EVPD (Enable Vital Product Data) check
            if (cmd[1] & 0x01) {
                uint8_t page_code = cmd[2];
                if (page_code == 0x00) { // Supported VPD Pages
                    ctx.media_buffer[0] = 0x00; // Direct access block device
                    ctx.media_buffer[1] = 0x00; // Supported VPD Pages page
                    ctx.media_buffer[2] = 0x00; // Reserved
                    ctx.media_buffer[3] = 0x03; // Page length = 3
                    ctx.media_buffer[4] = 0x00; // Page 0x00 supported
                    ctx.media_buffer[5] = 0x80; // Page 0x80 supported
                    ctx.media_buffer[6] = 0x83; // Page 0x83 supported
                    ctx.data_len = USB_MIN(cbw.dCBWDataTransferLength, 7U);
                    return 0;
                } else if (page_code == 0x80) { // Unit Serial Number Page
                    ctx.media_buffer[0] = 0x00; // Direct access block device
                    ctx.media_buffer[1] = 0x80; // Page code
                    ctx.media_buffer[2] = 0x00; // Reserved
                    ctx.media_buffer[3] = 12;   // Page length
                    std::memcpy(&ctx.media_buffer[4], "LNMSC0000003", 12);
                    ctx.data_len = USB_MIN(cbw.dCBWDataTransferLength, 16U);
                    return 0;
                } else if (page_code == 0x83) { // Device Identification Page
                    ctx.media_buffer[0] = 0x00; // Direct access block device
                    ctx.media_buffer[1] = 0x83; // Page code
                    ctx.media_buffer[2] = 0x00; // Page length MSB
                    ctx.media_buffer[3] = 12;   // Page length LSB
                    ctx.media_buffer[4] = 0x02; // Code Set: ASCII
                    ctx.media_buffer[5] = 0x01; // Identifier type: T10 Vendor ID, Association: Logical unit
                    ctx.media_buffer[6] = 0x00; // Reserved
                    ctx.media_buffer[7] = 8;    // Identifier length
                    std::memcpy(&ctx.media_buffer[8], "Sipeed01", 8);
                    ctx.data_len = USB_MIN(cbw.dCBWDataTransferLength, 16U);
                    return 0;
                } else {
                    set_sense(SenseKey::ILLEGAL_REQUEST, Asc::INVALID_FIELD_IN_COMMAND);
                    return -1;
                }
            }
            uint16_t len = static_cast<uint16_t>(USB_MIN(cbw.dCBWDataTransferLength, sizeof(kInquiryData)));
            std::memcpy(ctx.media_buffer, kInquiryData, len);
            ctx.data_len = len;
            return 0;
        }

        case ScsiCmd::READ_CAPACITY_10: {
            uint32_t blk_cnt = 0, blk_sz = 512;
            if (!msc_disk_get_capacity(blk_cnt, blk_sz)) {
                set_sense(SenseKey::NOT_READY, Asc::MEDIUM_NOT_PRESENT);
                return -1;
            }
            uint32_t last_lba = blk_cnt - 1;
            ctx.media_buffer[0] = static_cast<uint8_t>(last_lba >> 24);
            ctx.media_buffer[1] = static_cast<uint8_t>(last_lba >> 16);
            ctx.media_buffer[2] = static_cast<uint8_t>(last_lba >> 8);
            ctx.media_buffer[3] = static_cast<uint8_t>(last_lba);
            ctx.media_buffer[4] = static_cast<uint8_t>(blk_sz >> 24);
            ctx.media_buffer[5] = static_cast<uint8_t>(blk_sz >> 16);
            ctx.media_buffer[6] = static_cast<uint8_t>(blk_sz >> 8);
            ctx.media_buffer[7] = static_cast<uint8_t>(blk_sz);
            ctx.data_len = 8;
            return 0;
        }

        case ScsiCmd::READ_FORMAT_CAPACITIES: {
            uint32_t blk_cnt = 0, blk_sz = 512;
            if (!msc_disk_get_capacity(blk_cnt, blk_sz)) {
                set_sense(SenseKey::NOT_READY, Asc::MEDIUM_NOT_PRESENT);
                return -1;
            }
            std::memset(ctx.media_buffer, 0, 12);
            ctx.media_buffer[3] = 0x08; // Capacity list length
            // Current / Maximum capacity descriptor
            ctx.media_buffer[4] = static_cast<uint8_t>(blk_cnt >> 24);
            ctx.media_buffer[5] = static_cast<uint8_t>(blk_cnt >> 16);
            ctx.media_buffer[6] = static_cast<uint8_t>(blk_cnt >> 8);
            ctx.media_buffer[7] = static_cast<uint8_t>(blk_cnt);
            ctx.media_buffer[8] = 0x02; // Formatted Media
            ctx.media_buffer[9] = static_cast<uint8_t>(blk_sz >> 16);
            ctx.media_buffer[10] = static_cast<uint8_t>(blk_sz >> 8);
            ctx.media_buffer[11] = static_cast<uint8_t>(blk_sz);
            ctx.data_len = 12;
            return 0;
        }

        case ScsiCmd::MODE_SENSE_6: {
            ctx.media_buffer[0] = 0x03; // Mode data length
            ctx.media_buffer[1] = 0x00; // Medium type: SBC
            ctx.media_buffer[2] = 0x00; // Device-specific param (bit 7 = 0: not write protected)
            ctx.media_buffer[3] = 0x00; // Block descriptor length
            ctx.data_len = 4;
            return 0;
        }

        case ScsiCmd::MODE_SENSE_10: {
            std::memset(ctx.media_buffer, 0, 8);
            ctx.media_buffer[1] = 0x06; // Mode data length
            ctx.data_len = 8;
            return 0;
        }

        case ScsiCmd::READ_10: {
            if ((cbw.bmCBWFlags & 0x80) == 0) {
                msc_trace_record(4, 0x28, 1, 0, cbw.bmCBWFlags, 0);
                set_sense(SenseKey::ILLEGAL_REQUEST, Asc::INVALID_CDB);
                return -1;
            }
            if (!msc_disk_ready()) {
                msc_trace_record(4, 0x28, 2, 0, 0, 0);
                set_sense(SenseKey::NOT_READY, Asc::MEDIUM_NOT_PRESENT);
                return -1;
            }

            ctx.lba = (static_cast<uint32_t>(cmd[2]) << 24) |
                      (static_cast<uint32_t>(cmd[3]) << 16) |
                      (static_cast<uint32_t>(cmd[4]) << 8)  |
                      static_cast<uint32_t>(cmd[5]);
            uint32_t blocks = (static_cast<uint32_t>(cmd[7]) << 8) | static_cast<uint32_t>(cmd[8]);
            if (blocks == 0 && cbw.dCBWDataTransferLength > 0) {
                blocks = cbw.dCBWDataTransferLength / 512;
            }
            if (blocks == 0) {
                ctx.data_len = 0;
                return 0;
            }

            uint32_t blk_cnt = 0, blk_sz = 512;
            msc_disk_get_capacity(blk_cnt, blk_sz);
            if (ctx.lba + blocks > blk_cnt) {
                msc_trace_record(4, 0x28, 3, 0, ctx.lba, blocks);
                set_sense(SenseKey::ILLEGAL_REQUEST, Asc::ADDRESS_OUT_OF_RANGE);
                return -1;
            }

            ctx.state = BbbState::DATA_IN;
            ctx.remaining_bytes = blocks * 512;

            if (cbw.dCBWDataTransferLength != ctx.remaining_bytes) {
                msc_trace_record(4, 0x28, 4, 0, cbw.dCBWDataTransferLength, ctx.remaining_bytes);
                set_sense(SenseKey::ILLEGAL_REQUEST, Asc::INVALID_CDB);
                return -1;
            }

            ctx.read_pending = true;
            return 0;
        }

        case ScsiCmd::WRITE_10: {
            if ((cbw.bmCBWFlags & 0x80) != 0) {
                msc_trace_record(4, 0x2A, 1, 0, cbw.bmCBWFlags, 0);
                set_sense(SenseKey::ILLEGAL_REQUEST, Asc::INVALID_CDB);
                return -1;
            }
            if (!msc_disk_ready()) {
                msc_trace_record(4, 0x2A, 2, 0, 0, 0);
                set_sense(SenseKey::NOT_READY, Asc::MEDIUM_NOT_PRESENT);
                return -1;
            }

            ctx.lba = (static_cast<uint32_t>(cmd[2]) << 24) |
                      (static_cast<uint32_t>(cmd[3]) << 16) |
                      (static_cast<uint32_t>(cmd[4]) << 8)  |
                      static_cast<uint32_t>(cmd[5]);
            uint32_t blocks = (static_cast<uint32_t>(cmd[7]) << 8) | static_cast<uint32_t>(cmd[8]);
            if (blocks == 0 && cbw.dCBWDataTransferLength > 0) {
                blocks = cbw.dCBWDataTransferLength / 512;
            }
            if (blocks == 0) {
                ctx.data_len = 0;
                return 0;
            }

            uint32_t blk_cnt = 0, blk_sz = 512;
            msc_disk_get_capacity(blk_cnt, blk_sz);
            if (ctx.lba + blocks > blk_cnt) {
                msc_trace_record(4, 0x2A, 3, 0, ctx.lba, blocks);
                set_sense(SenseKey::ILLEGAL_REQUEST, Asc::ADDRESS_OUT_OF_RANGE);
                return -1;
            }

            ctx.state = BbbState::DATA_OUT;
            ctx.remaining_bytes = blocks * 512;

            if (cbw.dCBWDataTransferLength != ctx.remaining_bytes) {
                msc_trace_record(4, 0x2A, 4, 0, cbw.dCBWDataTransferLength, ctx.remaining_bytes);
                set_sense(SenseKey::ILLEGAL_REQUEST, Asc::INVALID_CDB);
                return -1;
            }

            uint32_t rx_len = USB_MIN(ctx.remaining_bytes, MSC_MEDIA_PACKET_SIZE);
            usbd_ep_recev(udev, MSC_OUT_EP, ctx.media_buffer, rx_len);
            return 0;
        }

        case ScsiCmd::PREVENT_ALLOW_REMOVAL:
        case ScsiCmd::START_STOP_UNIT:
        case ScsiCmd::VERIFY_10:
        case ScsiCmd::SYNCHRONIZE_CACHE_10: {
            ctx.data_len = 0;
            return 0;
        }

        case ScsiCmd::SERVICE_ACTION_IN_16: { // 0x9E
            uint8_t action = cmd[1] & 0x1F;
            if (action == 0x10) { // READ_CAPACITY_16
                uint32_t blk_cnt = 0, blk_sz = 512;
                if (!msc_disk_get_capacity(blk_cnt, blk_sz)) {
                    set_sense(SenseKey::NOT_READY, Asc::MEDIUM_NOT_PRESENT);
                    return -1;
                }
                uint32_t last_lba = blk_cnt - 1;
                std::memset(ctx.media_buffer, 0, 32);
                ctx.media_buffer[4] = static_cast<uint8_t>(last_lba >> 24);
                ctx.media_buffer[5] = static_cast<uint8_t>(last_lba >> 16);
                ctx.media_buffer[6] = static_cast<uint8_t>(last_lba >> 8);
                ctx.media_buffer[7] = static_cast<uint8_t>(last_lba);
                ctx.media_buffer[8] = static_cast<uint8_t>(blk_sz >> 24);
                ctx.media_buffer[9] = static_cast<uint8_t>(blk_sz >> 16);
                ctx.media_buffer[10] = static_cast<uint8_t>(blk_sz >> 8);
                ctx.media_buffer[11] = static_cast<uint8_t>(blk_sz);
                ctx.data_len = USB_MIN(cbw.dCBWDataTransferLength, 32U);
                return 0;
            }
            set_sense(SenseKey::ILLEGAL_REQUEST, Asc::INVALID_FIELD_IN_COMMAND);
            return -1;
        }

        case ScsiCmd::SECURITY_PROTOCOL_IN: { // 0xA2
            if (cmd[1] == 0x00) { // Security Protocol Information (0x00)
                // Return 8 bytes: supported protocol list length = 0
                std::memset(ctx.media_buffer, 0, 8);
                ctx.data_len = USB_MIN(cbw.dCBWDataTransferLength, 8U);
                return 0;
            }
            set_sense(SenseKey::ILLEGAL_REQUEST, Asc::INVALID_FIELD_IN_COMMAND);
            return -1;
        }

        default: {
            msc_trace_record(4, cmd[0], 99, 0, cbw.dCBWDataTransferLength, 0);
            set_sense(SenseKey::ILLEGAL_REQUEST, Asc::INVALID_CDB);
            return -1;
        }
    }
}

static void cbw_decode(usb_core_driver *udev) {
    uint32_t rx_count = usbd_rxcount_get(udev, MSC_OUT_EP);
    auto &cbw = ctx.cbw();
    ctx.remaining_bytes = cbw.dCBWDataTransferLength;

    msc_trace_record(1, cbw.CBWCB[0], cbw.bCBWCBLength, static_cast<uint8_t>(rx_count), cbw.dCBWDataTransferLength, cbw.dCBWTag, cbw.CBWCB);

    if (rx_count != BBB_CBW_LENGTH ||
        cbw.dCBWSignature != BBB_CBW_SIGNATURE ||
        cbw.bCBWLUN != 0 ||
        cbw.bCBWCBLength < 1 ||
        cbw.bCBWCBLength > 16) {
        set_sense(SenseKey::ILLEGAL_REQUEST, Asc::INVALID_CDB);
        ctx.status = BbbStatus::ERROR;
        msc_trace_record(4, cbw.CBWCB[0], 0, 1, rx_count, cbw.dCBWSignature);
        abort_transfer(udev);
        return;
    }

    if (process_scsi(udev) < 0) {
        abort_transfer(udev);
    } else if (ctx.state != BbbState::DATA_IN && ctx.state != BbbState::DATA_OUT) {
        if (ctx.data_len > 0) {
            data_send(udev, ctx.media_buffer, ctx.data_len);
        } else {
            csw_send(udev, CswStatus::CMD_PASSED);
        }
    }
}

uint8_t init(usb_dev *udev, uint8_t config_index) {
    (void)config_index;
    auto *pcore = reinterpret_cast<usb_core_driver*>(udev);

    usbd_ep_setup(pcore, &msc_config_desc.msc_epin);
    usbd_ep_setup(pcore, &msc_config_desc.msc_epout);

    ctx.state = BbbState::IDLE;
    ctx.status = BbbStatus::NORMAL;
    ctx.read_pending = false;
    ctx.write_pending = false;

    usbd_fifo_flush(pcore, MSC_OUT_EP);
    usbd_fifo_flush(pcore, MSC_IN_EP);

    usbd_ep_recev(pcore, MSC_OUT_EP, ctx.cbw_buf, BBB_CBW_LENGTH);
    msc_trace_record(5, 0xAA, config_index, 0, 0, 0);
    return USBD_OK;
}

uint8_t deinit(usb_dev *udev, uint8_t config_index) {
    (void)config_index;
    auto *pcore = reinterpret_cast<usb_core_driver*>(udev);
    ctx.read_pending = false;
    ctx.write_pending = false;
    usbd_ep_clear(pcore, MSC_IN_EP);
    usbd_ep_clear(pcore, MSC_OUT_EP);
    return USBD_OK;
}

uint8_t req_handler(usb_dev *udev, usb_req *req) {
    auto *pcore = reinterpret_cast<usb_core_driver*>(udev);
    usb_transc *transc = &pcore->dev.transc_in[0];

    // Class request targeted to interface
    if ((req->bmRequestType & USB_RECPTYPE_MASK) == USB_RECPTYPE_ITF) {
        switch (req->bRequest) {
            case REQ_GET_MAX_LUN:
                ctx.max_lun = 0; // Single LUN (LUN 0)
                transc->xfer_buf = &ctx.max_lun;
                transc->remain_len = 1U;
                msc_trace_record(5, REQ_GET_MAX_LUN, 0, 0, 1, 0);
                return USBD_OK;

            case REQ_BBB_RESET:
                ctx.state = BbbState::IDLE;
                ctx.status = BbbStatus::RECOVERY;
                ctx.read_pending = false;
                ctx.write_pending = false;
                msc_trace_record(5, REQ_BBB_RESET, 0, 0, 0, 0);
                usbd_ep_recev(pcore, MSC_OUT_EP, ctx.cbw_buf, BBB_CBW_LENGTH);
                return USBD_OK;

            default:
                return USBD_FAIL;
        }
    }

    // Standard request targeted to endpoint (Clear Feature STALL)
    if ((req->bmRequestType & USB_RECPTYPE_MASK) == USB_RECPTYPE_EP) {
        if (req->bRequest == USB_CLEAR_FEATURE) {
            uint8_t ep = static_cast<uint8_t>(req->wIndex);
            if (ctx.status == BbbStatus::ERROR) {
                usbd_ep_stall(pcore, MSC_IN_EP);
                ctx.status = BbbStatus::NORMAL;
            } else if ((ep & 0x80) && ctx.status != BbbStatus::RECOVERY) {
                csw_send(pcore, CswStatus::CMD_FAILED);
            }
            return USBD_OK;
        }
    }

    return USBD_FAIL;
}

uint8_t data_in(usb_dev *udev, uint8_t ep_num) {
    auto *pcore = reinterpret_cast<usb_core_driver*>(udev);
    if (ep_num != (MSC_IN_EP & 0x7F)) return USBD_FAIL;

    switch (ctx.state) {
        case BbbState::DATA_IN:
            ctx.read_pending = true;
            break;

        case BbbState::SEND_DATA:
        case BbbState::LAST_DATA_IN:
            csw_send(pcore, CswStatus::CMD_PASSED);
            break;

        default:
            break;
    }
    return USBD_OK;
}

uint8_t data_out(usb_dev *udev, uint8_t ep_num) {
    auto *pcore = reinterpret_cast<usb_core_driver*>(udev);
    if (ep_num != (MSC_OUT_EP & 0x7F)) return USBD_FAIL;

    switch (ctx.state) {
        case BbbState::IDLE:
            cbw_decode(pcore);
            break;

        case BbbState::DATA_OUT:
            ctx.write_pending = true;
            break;

        default:
            break;
    }
    return USBD_OK;
}

void poll(usb_core_driver *udev) {
    if (ctx.read_pending) {
        ctx.read_pending = false;
        uint32_t len = USB_MIN(ctx.remaining_bytes, MSC_MEDIA_PACKET_SIZE);
        uint32_t blocks = len / 512;
        if (blocks == 0) blocks = 1;

        if (msc_disk_read(ctx.media_buffer, ctx.lba, blocks) != 0) {
            set_sense(SenseKey::HARDWARE_ERROR, Asc::UNRECOVERED_READ_ERROR);
            abort_transfer(udev);
            return;
        }

        ctx.lba += blocks;
        ctx.remaining_bytes -= len;

        if (ctx.remaining_bytes == 0) {
            ctx.state = BbbState::LAST_DATA_IN;
        }
        usbd_ep_send(udev, MSC_IN_EP, ctx.media_buffer, len);
    } else if (ctx.write_pending) {
        ctx.write_pending = false;
        uint32_t len = USB_MIN(ctx.remaining_bytes, MSC_MEDIA_PACKET_SIZE);
        uint32_t blocks = len / 512;
        if (blocks == 0) blocks = 1;

        if (msc_disk_write(ctx.media_buffer, ctx.lba, blocks) != 0) {
            set_sense(SenseKey::HARDWARE_ERROR, Asc::WRITE_FAULT);
            abort_transfer(udev);
            return;
        }

        ctx.lba += blocks;
        ctx.remaining_bytes -= len;

        if (ctx.remaining_bytes == 0) {
            csw_send(udev, CswStatus::CMD_PASSED);
        } else {
            uint32_t next_len = USB_MIN(ctx.remaining_bytes, MSC_MEDIA_PACKET_SIZE);
            usbd_ep_recev(udev, MSC_OUT_EP, ctx.media_buffer, next_len);
        }
    }
}

usb_class_core msc_class = {
    .command              = 0,
    .alter_set            = 0,
    .init                 = init,
    .deinit               = deinit,
    .req_proc             = req_handler,
    .set_intf             = nullptr,
    .ctlx_in              = nullptr,
    .ctlx_out             = nullptr,
    .data_in              = data_in,
    .data_out             = data_out,
    .SOF                  = nullptr,
    .incomplete_isoc_in   = nullptr,
    .incomplete_isoc_out  = nullptr
};

} // namespace msc
