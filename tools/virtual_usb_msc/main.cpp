#include <iostream>
#include <csignal>
#include <string>
#include "virtual_disk.hpp"
#include "usbip_server.hpp"

namespace {
    UsbipServer* g_server = nullptr;

    void signal_handler(int sig) {
        std::cout << "\n[MAIN] Caught signal " << sig << ", shutting down...\n";
        if (g_server) {
            g_server->stop();
        }
    }
}

int main(int argc, char* argv[]) {
    std::cout << std::unitbuf;
    std::cerr << std::unitbuf;

    std::string disk_path = "virtual_sd.img";
    uint32_t    size_mb   = 32;
    uint16_t    port      = usbip::DEFAULT_PORT;

    for (int i = 1; i < argc; ++i) {
        std::string arg = argv[i];
        if (arg == "--disk" && i + 1 < argc) {
            disk_path = argv[++i];
        } else if (arg == "--size" && i + 1 < argc) {
            size_mb = std::stoul(argv[++i]);
        } else if (arg == "--port" && i + 1 < argc) {
            port = static_cast<uint16_t>(std::stoul(argv[++i]));
        } else if (arg == "--help" || arg == "-h") {
            std::cout << "Usage: virtual_usb_msc [--disk <path>] [--size <MB>] [--port <port>]\n";
            return 0;
        }
    }

    std::cout << "========================================================\n"
              << "  Sipeed Longan Nano USB MSC Virtual Simulation Server\n"
              << "  C++23 SITL Simulation for Linux usb-storage / vhci-hcd\n"
              << "========================================================\n";

    VirtualDisk disk;
    uint32_t sector_count = (size_mb * 1024 * 1024) / 512;
    if (!disk.init(disk_path, sector_count, 512)) {
        std::cerr << "[MAIN] Failed to initialize virtual disk\n";
        return 1;
    }

    UsbipServer server(disk, port);
    g_server = &server;

    std::signal(SIGINT, signal_handler);
    std::signal(SIGTERM, signal_handler);

    if (!server.run()) {
        std::cerr << "[MAIN] Server run failed\n";
        return 1;
    }

    std::cout << "[MAIN] Server stopped cleanly.\n";
    return 0;
}
