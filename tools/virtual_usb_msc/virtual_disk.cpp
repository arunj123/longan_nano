#include "virtual_disk.hpp"
#include <iostream>
#include <filesystem>
#include <cstring>

VirtualDisk::~VirtualDisk() {
    if (file_.is_open()) {
        file_.flush();
        file_.close();
    }
}

bool VirtualDisk::init(const std::string& filepath, uint32_t sector_count, uint32_t sector_size) {
    filepath_ = filepath;
    sector_count_ = sector_count;
    sector_size_ = sector_size;

    const auto total_bytes = static_cast<std::streamoff>(sector_count_) * sector_size_;
    bool created_new = false;

    if (!std::filesystem::exists(filepath_) || std::filesystem::file_size(filepath_) < static_cast<uintmax_t>(total_bytes)) {
        std::cout << "[DISK] Creating virtual disk image: " << filepath_ 
                  << " (" << (total_bytes / (1024 * 1024)) << " MB)\n";
        file_.open(filepath_, std::ios::in | std::ios::out | std::ios::binary | std::ios::trunc);
        if (!file_.is_open()) {
            std::cerr << "[DISK] Failed to create: " << filepath_ << "\n";
            return false;
        }

        // Pre-allocate full size with zeros
        std::vector<uint8_t> zero_chunk(65536, 0);
        size_t written = 0;
        while (written < static_cast<size_t>(total_bytes)) {
            size_t to_write = std::min(zero_chunk.size(), static_cast<size_t>(total_bytes) - written);
            file_.write(reinterpret_cast<const char*>(zero_chunk.data()), to_write);
            written += to_write;
        }
        file_.flush();
        created_new = true;
    } else {
        std::cout << "[DISK] Opening existing disk image: " << filepath_ << "\n";
        file_.open(filepath_, std::ios::in | std::ios::out | std::ios::binary);
        if (!file_.is_open()) {
            std::cerr << "[DISK] Failed to open: " << filepath_ << "\n";
            return false;
        }
    }

    is_open_ = true;

    if (created_new) {
        format_default_mbr();
    }

    return true;
}

void VirtualDisk::format_default_mbr() {
    std::cout << "[DISK] Writing default MBR partition table to LBA 0...\n";
    std::vector<uint8_t> mbr(sector_size_, 0);

    // Bootstrap jump code dummy
    mbr[0] = 0xEB; mbr[1] = 0x3C; mbr[2] = 0x90;

    // Partition Entry 1 at offset 446 (0x1BE)
    uint8_t* p1 = &mbr[0x1BE];
    p1[0] = 0x80; // Active / Bootable
    p1[1] = 0x01; p1[2] = 0x01; p1[3] = 0x00; // CHS start
    p1[4] = 0x0C; // Type: FAT32 with LBA
    p1[5] = 0xFE; p1[6] = 0xFF; p1[7] = 0xFF; // CHS end

    // Starting LBA: 2048 (0x00000800)
    uint32_t start_lba = 2048;
    p1[8]  = static_cast<uint8_t>(start_lba & 0xFF);
    p1[9]  = static_cast<uint8_t>((start_lba >> 8) & 0xFF);
    p1[10] = static_cast<uint8_t>((start_lba >> 16) & 0xFF);
    p1[11] = static_cast<uint8_t>((start_lba >> 24) & 0xFF);

    // Number of sectors: sector_count_ - 2048
    uint32_t num_sectors = (sector_count_ > 2048) ? (sector_count_ - 2048) : 0;
    p1[12] = static_cast<uint8_t>(num_sectors & 0xFF);
    p1[13] = static_cast<uint8_t>((num_sectors >> 8) & 0xFF);
    p1[14] = static_cast<uint8_t>((num_sectors >> 16) & 0xFF);
    p1[15] = static_cast<uint8_t>((num_sectors >> 24) & 0xFF);

    // MBR Signature 0x55, 0xAA at bytes 510-511
    mbr[510] = 0x55;
    mbr[511] = 0xAA;

    write_sectors(0, 1, mbr);
}

bool VirtualDisk::read_sectors(uint32_t lba, uint32_t count, std::span<uint8_t> out) {
    if (!is_open_) return false;
    if (lba + count > sector_count_) {
        std::cerr << "[DISK] Read LBA out of range: " << lba << " + " << count << " > " << sector_count_ << "\n";
        return false;
    }
    const size_t bytes_to_read = static_cast<size_t>(count) * sector_size_;
    if (out.size() < bytes_to_read) {
        std::cerr << "[DISK] Read buffer too small: " << out.size() << " < " << bytes_to_read << "\n";
        return false;
    }

    const auto offset = static_cast<std::streamoff>(lba) * sector_size_;
    file_.seekg(offset, std::ios::beg);
    if (!file_) {
        file_.clear();
        file_.seekg(offset, std::ios::beg);
    }
    file_.read(reinterpret_cast<char*>(out.data()), bytes_to_read);
    return file_.good();
}

bool VirtualDisk::write_sectors(uint32_t lba, uint32_t count, std::span<const uint8_t> in) {
    if (!is_open_) return false;
    if (lba + count > sector_count_) {
        std::cerr << "[DISK] Write LBA out of range: " << lba << " + " << count << " > " << sector_count_ << "\n";
        return false;
    }
    const size_t bytes_to_write = static_cast<size_t>(count) * sector_size_;
    if (in.size() < bytes_to_write) {
        std::cerr << "[DISK] Write buffer too small: " << in.size() << " < " << bytes_to_write << "\n";
        return false;
    }

    const auto offset = static_cast<std::streamoff>(lba) * sector_size_;
    file_.seekp(offset, std::ios::beg);
    if (!file_) {
        file_.clear();
        file_.seekp(offset, std::ios::beg);
    }
    file_.write(reinterpret_cast<const char*>(in.data()), bytes_to_write);
    file_.flush();
    return file_.good();
}
