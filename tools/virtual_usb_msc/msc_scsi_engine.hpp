#pragma once

#include <cstdint>
#include <cstddef>
#include <span>
#include <vector>
#include <array>
#include <string>
#include "virtual_disk.hpp"

namespace msc {

// Bulk-Only Transport (BOT) Signatures & Constants
inline constexpr uint32_t BBB_CBW_SIGNATURE = 0x43425355U; // "USBC"
inline constexpr uint32_t BBB_CSW_SIGNATURE = 0x53425355U; // "USBS"
inline constexpr size_t   BBB_CBW_LENGTH    = 31U;
inline constexpr size_t   BBB_CSW_LENGTH    = 13U;

enum class CswStatus : uint8_t {
    CMD_PASSED  = 0x00,
    CMD_FAILED  = 0x01,
    PHASE_ERROR = 0x02
};

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
    REPORT_LUNS            = 0xA0,
    SECURITY_PROTOCOL_IN   = 0xA2
};

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

class MscScsiEngine {
public:
    explicit MscScsiEngine(VirtualDisk& disk);

    void reset();

    /// @brief Receives and parses a 31-byte Command Block Wrapper (CBW)
    /// @return true if CBW is syntactically valid; false if transport error
    bool on_cbw(std::span<const uint8_t> cbw_data);

    /// @brief Called when host sends data for WRITE_6 / WRITE_10
    bool on_data_out(std::span<const uint8_t> data);

    /// @brief Retrieves data payload to return on Bulk-IN (e.g. Inquiry, Read sectors)
    /// @param max_len Maximum bytes host requested in URB
    /// @return Span of data ready to transmit, or empty if no data phase
    [[nodiscard]] std::span<const uint8_t> get_in_data(size_t max_len);

    /// @brief Consumes bytes transferred on Bulk-IN
    void consume_in_data(size_t bytes_sent);

    /// @brief Returns the 13-byte CSW packet
    [[nodiscard]] std::span<const uint8_t> get_csw();

    [[nodiscard]] bool has_pending_in_data() const noexcept {
        return in_data_offset_ < in_data_buffer_.size();
    }
    [[nodiscard]] bool is_waiting_data_out() const noexcept {
        return state_ == State::DATA_OUT;
    }
    [[nodiscard]] bool is_waiting_csw() const noexcept {
        return state_ == State::CSW_PENDING;
    }
    [[nodiscard]] bool is_stall_in() const noexcept {
        return stall_in_;
    }
    [[nodiscard]] bool is_stall_out() const noexcept {
        return stall_out_;
    }
    void clear_stall(uint8_t ep);

private:
    enum class State {
        IDLE,
        DATA_IN,
        DATA_OUT,
        CSW_PENDING
    };

    VirtualDisk& disk_;
    State        state_{State::IDLE};
    BbbCbw       cbw_{};
    BbbCsw       csw_{};

    SenseKey     sense_key_{SenseKey::NO_SENSE};
    Asc          sense_asc_{Asc::NO_ASC};

    std::vector<uint8_t> in_data_buffer_;
    size_t               in_data_offset_{0};

    uint32_t     write_lba_{0};
    uint32_t     write_remaining_bytes_{0};

    bool         stall_in_{false};
    bool         stall_out_{false};

    void set_sense(SenseKey key, Asc asc) noexcept {
        sense_key_ = key;
        sense_asc_ = asc;
    }

    void process_scsi();
    void prepare_csw(CswStatus status, uint32_t residue);
};

} // namespace msc
