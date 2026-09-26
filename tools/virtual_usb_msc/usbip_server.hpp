#pragma once

#include <cstdint>
#include <string>
#include <memory>
#include <atomic>
#include <span>
#include "usbip_protocol.hpp"
#include "virtual_disk.hpp"
#include "msc_scsi_engine.hpp"

class UsbipServer {
public:
    explicit UsbipServer(VirtualDisk& disk, uint16_t port = usbip::DEFAULT_PORT);
    ~UsbipServer();

    UsbipServer(const UsbipServer&) = delete;
    UsbipServer& operator=(const UsbipServer&) = delete;

    /// @brief Starts listening and running the server loop
    bool run();

    /// @brief Signals server to stop
    void stop() noexcept;

private:
    VirtualDisk&        disk_;
    msc::MscScsiEngine  engine_;
    uint16_t            port_;
    int                 server_fd_{-1};
    std::atomic<bool>   running_{false};

    bool handle_client(int client_fd);
    bool handle_devlist(int client_fd, const usbip::OpHeader& req_hdr);
    bool handle_import(int client_fd, const usbip::OpHeader& req_hdr);
    bool handle_urb_loop(int client_fd);

    void handle_ep0_control(int client_fd, const usbip::UsbipPacketHeader& cmd_hdr);
    void handle_ep1_bulk_out(int client_fd, const usbip::UsbipPacketHeader& cmd_hdr);
    void handle_ep1_bulk_in(int client_fd, const usbip::UsbipPacketHeader& cmd_hdr);

    bool send_ret_submit(int client_fd, const usbip::UsbipPacketHeader& cmd_hdr, int32_t status, std::span<const uint8_t> data);
};
