#pragma once

#include <cstdint>
#include <cstddef>
#include <span>
#include <string>
#include <fstream>
#include <vector>

class VirtualDisk {
public:
    VirtualDisk() = default;
    ~VirtualDisk();

    VirtualDisk(const VirtualDisk&) = delete;
    VirtualDisk& operator=(const VirtualDisk&) = delete;

    /// @brief Opens or creates a backing file image
    bool init(const std::string& filepath, uint32_t sector_count = 65536, uint32_t sector_size = 512);

    /// @brief Reads sectors from the virtual disk into the output buffer
    bool read_sectors(uint32_t lba, uint32_t count, std::span<uint8_t> out);

    /// @brief Writes sectors to the virtual disk from the input buffer
    bool write_sectors(uint32_t lba, uint32_t count, std::span<const uint8_t> in);

    [[nodiscard]] uint32_t sector_count() const noexcept { return sector_count_; }
    [[nodiscard]] uint32_t sector_size() const noexcept { return sector_size_; }
    [[nodiscard]] bool is_ready() const noexcept { return is_open_; }

private:
    std::fstream file_;
    std::string  filepath_;
    uint32_t     sector_count_{0};
    uint32_t     sector_size_{512};
    bool         is_open_{false};

    void format_default_mbr();
};
