#pragma once

#include <cstdint>
#include <span>
#include <string_view>
#include <utility>
#include "fatfs/ff.h"

namespace drivers::fatfs {

[[nodiscard]] inline const char* result_to_string(FRESULT res) noexcept {
    switch (res) {
        case FR_OK:                  return "FR_OK: Succeeded";
        case FR_DISK_ERR:            return "FR_DISK_ERR: Low-level disk I/O error";
        case FR_INT_ERR:             return "FR_INT_ERR: Assertion failed";
        case FR_NOT_READY:           return "FR_NOT_READY: Physical drive cannot work";
        case FR_NO_FILE:             return "FR_NO_FILE: Could not find the file";
        case FR_NO_PATH:             return "FR_NO_PATH: Could not find the path";
        case FR_INVALID_NAME:        return "FR_INVALID_NAME: Path name format is invalid";
        case FR_DENIED:              return "FR_DENIED: Access denied or directory full";
        case FR_EXIST:               return "FR_EXIST: Access denied due to prohibited access";
        case FR_INVALID_OBJECT:      return "FR_INVALID_OBJECT: File/directory object is invalid";
        case FR_WRITE_PROTECTED:     return "FR_WRITE_PROTECTED: Physical drive is write protected";
        case FR_INVALID_DRIVE:       return "FR_INVALID_DRIVE: Logical drive number is invalid";
        case FR_NOT_ENABLED:         return "FR_NOT_ENABLED: Volume has no work area";
        case FR_NO_FILESYSTEM:       return "FR_NO_FILESYSTEM: Valid FAT volume not found";
        case FR_MKFS_ABORTED:        return "FR_MKFS_ABORTED: f_mkfs aborted";
        case FR_TIMEOUT:             return "FR_TIMEOUT: File lock timeout";
        case FR_LOCKED:              return "FR_LOCKED: Operation rejected by file sharing protection";
        case FR_NOT_ENOUGH_CORE:     return "FR_NOT_ENOUGH_CORE: LFN working buffer cannot be allocated";
        case FR_TOO_MANY_OPEN_FILES: return "FR_TOO_MANY_OPEN_FILES: Number of open files > FF_FS_LOCK";
        case FR_INVALID_PARAMETER:   return "FR_INVALID_PARAMETER: Given parameter is invalid";
        default:                     return "Unknown FatFs Error";
    }
}

/**
 * @brief Modern RAII wrapper around FatFs FIL object.
 */
class File {
public:
    File() noexcept : is_open_(false) {}

    ~File() noexcept {
        close();
    }

    File(const File&) = delete;
    File& operator=(const File&) = delete;

    File(File&& other) noexcept : fil_(other.fil_), is_open_(other.is_open_) {
        other.is_open_ = false;
    }

    File& operator=(File&& other) noexcept {
        if (this != &other) {
            close();
            fil_ = other.fil_;
            is_open_ = other.is_open_;
            other.is_open_ = false;
        }
        return *this;
    }

    FRESULT open(const char* path, BYTE mode) noexcept {
        close();
        FRESULT res = f_open(&fil_, path, mode);
        if (res == FR_OK) {
            is_open_ = true;
        }
        return res;
    }

    FRESULT close() noexcept {
        if (is_open_) {
            is_open_ = false;
            return f_close(&fil_);
        }
        return FR_OK;
    }

    FRESULT write(std::span<const uint8_t> data, UINT& bytes_written) noexcept {
        if (!is_open_) return FR_INVALID_OBJECT;
        return f_write(&fil_, data.data(), static_cast<UINT>(data.size()), &bytes_written);
    }

    FRESULT write(std::string_view text, UINT& bytes_written) noexcept {
        return write(std::span<const uint8_t>{reinterpret_cast<const uint8_t*>(text.data()), text.size()}, bytes_written);
    }

    FRESULT read(std::span<uint8_t> buffer, UINT& bytes_read) noexcept {
        if (!is_open_) return FR_INVALID_OBJECT;
        return f_read(&fil_, buffer.data(), static_cast<UINT>(buffer.size()), &bytes_read);
    }

    FRESULT sync() noexcept {
        if (!is_open_) return FR_INVALID_OBJECT;
        return f_sync(&fil_);
    }

    [[nodiscard]] bool is_open() const noexcept { return is_open_; }
    [[nodiscard]] uint32_t size() const noexcept { return is_open_ ? static_cast<uint32_t>(f_size(&fil_)) : 0; }
    [[nodiscard]] FIL* raw_fil() noexcept { return &fil_; }

private:
    FIL fil_{};
    bool is_open_{false};
};

/**
 * @brief FAT Volume Manager.
 */
class FileSystem {
public:
    static inline FileSystem& instance() noexcept {
        static FileSystem fs_instance;
        return fs_instance;
    }

    FRESULT mount(const char* path = "0:", bool immediate = true) noexcept {
        if (mounted_) {
            unmount(path);
        }
        // opt=1 mounts the volume immediately (checks FAT boot sector)
        FRESULT res = f_mount(&fs_, path, immediate ? 1 : 0);
        if (res == FR_OK) {
            mounted_ = true;
        }
        return res;
    }

    FRESULT unmount(const char* path = "0:") noexcept {
        if (mounted_) {
            mounted_ = false;
            return f_mount(nullptr, path, 0);
        }
        return FR_OK;
    }

    [[nodiscard]] bool is_mounted() const noexcept { return mounted_; }

    FRESULT write_file(const char* path, std::string_view content) noexcept {
        File f;
        FRESULT res = f.open(path, FA_CREATE_ALWAYS | FA_WRITE);
        if (res != FR_OK) return res;

        UINT written = 0;
        res = f.write(content, written);
        if (res != FR_OK) return res;

        if (written != content.size()) return FR_DISK_ERR;
        return f.close();
    }

    FRESULT read_file(const char* path, std::span<char> out_buffer, UINT& bytes_read) noexcept {
        bytes_read = 0;
        File f;
        FRESULT res = f.open(path, FA_READ);
        if (res != FR_OK) return res;

        res = f.read(std::span<uint8_t>{reinterpret_cast<uint8_t*>(out_buffer.data()), out_buffer.size() - 1}, bytes_read);
        if (res != FR_OK) return res;

        out_buffer[bytes_read] = '\0'; // Null-terminate string buffer
        return f.close();
    }

    FRESULT remove(const char* path) noexcept {
        return f_unlink(path);
    }

    bool exists(const char* path) noexcept {
        FILINFO fno;
        return f_stat(path, &fno) == FR_OK;
    }

private:
    FileSystem() = default;
    FATFS fs_{};
    bool mounted_{false};
};

} // namespace drivers::fatfs
