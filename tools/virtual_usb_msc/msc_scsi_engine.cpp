#include "msc_scsi_engine.hpp"
#include <iostream>
#include <iomanip>
#include <cstring>
#include <algorithm>

namespace msc {

alignas(4) static constexpr uint8_t kInquiryData[36] = {
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

MscScsiEngine::MscScsiEngine(VirtualDisk& disk) : disk_(disk) {
    reset();
}

void MscScsiEngine::reset() {
    state_ = State::IDLE;
    std::memset(&cbw_, 0, sizeof(cbw_));
    std::memset(&csw_, 0, sizeof(csw_));
    sense_key_ = SenseKey::NO_SENSE;
    sense_asc_ = Asc::NO_ASC;
    in_data_buffer_.clear();
    in_data_offset_ = 0;
    write_lba_ = 0;
    write_remaining_bytes_ = 0;
    stall_in_ = false;
    stall_out_ = false;
}

void MscScsiEngine::clear_stall(uint8_t ep) {
    if (ep & 0x80) {
        stall_in_ = false;
    } else {
        stall_out_ = false;
    }
}

bool MscScsiEngine::on_cbw(std::span<const uint8_t> cbw_data) {
    if (cbw_data.size() != BBB_CBW_LENGTH) {
        std::cerr << "[MSC] Invalid CBW size: " << cbw_data.size() << " != 31\n";
        return false;
    }

    std::memcpy(&cbw_, cbw_data.data(), sizeof(cbw_));

    if (cbw_.dCBWSignature != BBB_CBW_SIGNATURE) {
        std::cerr << "[MSC] Invalid CBW signature: 0x" << std::hex << cbw_.dCBWSignature << std::dec << "\n";
        stall_in_ = true;
        stall_out_ = true;
        return false;
    }

    if (cbw_.bCBWLUN != 0 || cbw_.bCBWCBLength < 1 || cbw_.bCBWCBLength > 16) {
        std::cerr << "[MSC] Unsupported LUN (" << (int)cbw_.bCBWLUN << ") or length (" << (int)cbw_.bCBWCBLength << ")\n";
        set_sense(SenseKey::ILLEGAL_REQUEST, Asc::INVALID_CDB);
        prepare_csw(CswStatus::CMD_FAILED, cbw_.dCBWDataTransferLength);
        stall_in_ = true;
        return true;
    }

    in_data_buffer_.clear();
    in_data_offset_ = 0;
    stall_in_ = false;
    stall_out_ = false;

    process_scsi();
    return true;
}

void MscScsiEngine::process_scsi() {
    const uint8_t* cmd = cbw_.CBWCB;
    const auto opcode = static_cast<ScsiCmd>(cmd[0]);

    switch (opcode) {
        case ScsiCmd::TEST_UNIT_READY: {
            if (!disk_.is_ready()) {
                set_sense(SenseKey::NOT_READY, Asc::MEDIUM_NOT_PRESENT);
                prepare_csw(CswStatus::CMD_FAILED, cbw_.dCBWDataTransferLength);
                return;
            }
            prepare_csw(CswStatus::CMD_PASSED, 0);
            break;
        }

        case ScsiCmd::INQUIRY: {
            const size_t len = std::min<size_t>(cbw_.dCBWDataTransferLength, sizeof(kInquiryData));
            in_data_buffer_.assign(kInquiryData, kInquiryData + len);
            state_ = State::DATA_IN;
            prepare_csw(CswStatus::CMD_PASSED, cbw_.dCBWDataTransferLength - len);
            break;
        }

        case ScsiCmd::REQUEST_SENSE: {
            std::array<uint8_t, 18> sense_buf{};
            sense_buf[0]  = 0x70; // Current error
            sense_buf[2]  = static_cast<uint8_t>(sense_key_);
            sense_buf[7]  = 10;   // Additional length
            sense_buf[12] = static_cast<uint8_t>(sense_asc_);

            // Clear sense after reporting
            sense_key_ = SenseKey::NO_SENSE;
            sense_asc_ = Asc::NO_ASC;

            const size_t len = std::min<size_t>(cbw_.dCBWDataTransferLength, sense_buf.size());
            in_data_buffer_.assign(sense_buf.begin(), sense_buf.begin() + len);
            state_ = State::DATA_IN;
            prepare_csw(CswStatus::CMD_PASSED, cbw_.dCBWDataTransferLength - len);
            break;
        }

        case ScsiCmd::READ_CAPACITY_10: {
            if (!disk_.is_ready()) {
                set_sense(SenseKey::NOT_READY, Asc::MEDIUM_NOT_PRESENT);
                prepare_csw(CswStatus::CMD_FAILED, cbw_.dCBWDataTransferLength);
                stall_in_ = true;
                return;
            }
            const uint32_t last_lba = disk_.sector_count() - 1;
            const uint32_t block_sz = disk_.sector_size();
            std::array<uint8_t, 8> cap{};
            cap[0] = static_cast<uint8_t>((last_lba >> 24) & 0xFF);
            cap[1] = static_cast<uint8_t>((last_lba >> 16) & 0xFF);
            cap[2] = static_cast<uint8_t>((last_lba >> 8) & 0xFF);
            cap[3] = static_cast<uint8_t>(last_lba & 0xFF);
            cap[4] = static_cast<uint8_t>((block_sz >> 24) & 0xFF);
            cap[5] = static_cast<uint8_t>((block_sz >> 16) & 0xFF);
            cap[6] = static_cast<uint8_t>((block_sz >> 8) & 0xFF);
            cap[7] = static_cast<uint8_t>(block_sz & 0xFF);

            const size_t len = std::min<size_t>(cbw_.dCBWDataTransferLength, cap.size());
            in_data_buffer_.assign(cap.begin(), cap.begin() + len);
            state_ = State::DATA_IN;
            prepare_csw(CswStatus::CMD_PASSED, cbw_.dCBWDataTransferLength - len);
            break;
        }

        case ScsiCmd::SERVICE_ACTION_IN_16: { // 0x9E
            if ((cmd[1] & 0x1F) == 0x10) { // READ_CAPACITY_16
                if (!disk_.is_ready()) {
                    set_sense(SenseKey::NOT_READY, Asc::MEDIUM_NOT_PRESENT);
                    prepare_csw(CswStatus::CMD_FAILED, cbw_.dCBWDataTransferLength);
                    stall_in_ = true;
                    return;
                }
                const uint64_t last_lba = disk_.sector_count() - 1;
                const uint32_t block_sz = disk_.sector_size();
                std::array<uint8_t, 32> cap16{};
                for (int i = 0; i < 8; ++i) {
                    cap16[i] = static_cast<uint8_t>((last_lba >> (56 - i * 8)) & 0xFF);
                }
                cap16[8]  = static_cast<uint8_t>((block_sz >> 24) & 0xFF);
                cap16[9]  = static_cast<uint8_t>((block_sz >> 16) & 0xFF);
                cap16[10] = static_cast<uint8_t>((block_sz >> 8) & 0xFF);
                cap16[11] = static_cast<uint8_t>(block_sz & 0xFF);

                const size_t len = std::min<size_t>(cbw_.dCBWDataTransferLength, cap16.size());
                in_data_buffer_.assign(cap16.begin(), cap16.begin() + len);
                state_ = State::DATA_IN;
                prepare_csw(CswStatus::CMD_PASSED, cbw_.dCBWDataTransferLength - len);
                return;
            }
            set_sense(SenseKey::ILLEGAL_REQUEST, Asc::INVALID_FIELD_IN_COMMAND);
            prepare_csw(CswStatus::CMD_FAILED, cbw_.dCBWDataTransferLength);
            stall_in_ = true;
            break;
        }

        case ScsiCmd::READ_FORMAT_CAPACITIES: {
            if (!disk_.is_ready()) {
                set_sense(SenseKey::NOT_READY, Asc::MEDIUM_NOT_PRESENT);
                prepare_csw(CswStatus::CMD_FAILED, cbw_.dCBWDataTransferLength);
                stall_in_ = true;
                return;
            }
            const uint32_t blocks   = disk_.sector_count();
            const uint32_t block_sz = disk_.sector_size();
            std::array<uint8_t, 12> fmt_cap{};
            fmt_cap[3] = 0x08; // Capacity list length = 8
            fmt_cap[4] = static_cast<uint8_t>((blocks >> 24) & 0xFF);
            fmt_cap[5] = static_cast<uint8_t>((blocks >> 16) & 0xFF);
            fmt_cap[6] = static_cast<uint8_t>((blocks >> 8) & 0xFF);
            fmt_cap[7] = static_cast<uint8_t>(blocks & 0xFF);
            fmt_cap[8] = 0x02; // Formatted Media
            fmt_cap[9] = static_cast<uint8_t>((block_sz >> 16) & 0xFF);
            fmt_cap[10]= static_cast<uint8_t>((block_sz >> 8) & 0xFF);
            fmt_cap[11]= static_cast<uint8_t>(block_sz & 0xFF);

            const size_t len = std::min<size_t>(cbw_.dCBWDataTransferLength, fmt_cap.size());
            in_data_buffer_.assign(fmt_cap.begin(), fmt_cap.begin() + len);
            state_ = State::DATA_IN;
            prepare_csw(CswStatus::CMD_PASSED, cbw_.dCBWDataTransferLength - len);
            break;
        }

        case ScsiCmd::MODE_SENSE_6: {
            std::array<uint8_t, 4> mode{};
            mode[0] = 3;    // Mode data length
            mode[1] = 0x00; // Medium type (SBC)
            mode[2] = 0x00; // Device specific (Write Protect = 0)
            mode[3] = 0x00; // Block descriptor length

            const size_t len = std::min<size_t>(cbw_.dCBWDataTransferLength, mode.size());
            in_data_buffer_.assign(mode.begin(), mode.begin() + len);
            state_ = State::DATA_IN;
            prepare_csw(CswStatus::CMD_PASSED, cbw_.dCBWDataTransferLength - len);
            break;
        }

        case ScsiCmd::MODE_SENSE_10: {
            std::array<uint8_t, 8> mode10{};
            mode10[1] = 6;    // Mode data length
            mode10[2] = 0x00; // Medium type
            mode10[3] = 0x00; // Write Protect = 0

            const size_t len = std::min<size_t>(cbw_.dCBWDataTransferLength, mode10.size());
            in_data_buffer_.assign(mode10.begin(), mode10.begin() + len);
            state_ = State::DATA_IN;
            prepare_csw(CswStatus::CMD_PASSED, cbw_.dCBWDataTransferLength - len);
            break;
        }

        case ScsiCmd::REPORT_LUNS: { // 0xA0
            std::array<uint8_t, 16> luns{};
            luns[3] = 0x08; // LUN list length = 8 (one LUN)
            // Bytes 8-15: LUN 0 = all zeros
            const size_t len = std::min<size_t>(cbw_.dCBWDataTransferLength, luns.size());
            in_data_buffer_.assign(luns.begin(), luns.begin() + len);
            state_ = State::DATA_IN;
            prepare_csw(CswStatus::CMD_PASSED, cbw_.dCBWDataTransferLength - len);
            break;
        }

        case ScsiCmd::SECURITY_PROTOCOL_IN: { // 0xA2
            std::array<uint8_t, 8> sec{};
            // Length = 0
            const size_t len = std::min<size_t>(cbw_.dCBWDataTransferLength, sec.size());
            in_data_buffer_.assign(sec.begin(), sec.begin() + len);
            state_ = State::DATA_IN;
            prepare_csw(CswStatus::CMD_PASSED, cbw_.dCBWDataTransferLength - len);
            break;
        }

        case ScsiCmd::READ_10:
        case ScsiCmd::READ_6: {
            if (!disk_.is_ready()) {
                set_sense(SenseKey::NOT_READY, Asc::MEDIUM_NOT_PRESENT);
                prepare_csw(CswStatus::CMD_FAILED, cbw_.dCBWDataTransferLength);
                stall_in_ = true;
                return;
            }

            uint32_t lba = 0;
            uint32_t blocks = 0;
            if (opcode == ScsiCmd::READ_10) {
                lba = (static_cast<uint32_t>(cmd[2]) << 24) |
                      (static_cast<uint32_t>(cmd[3]) << 16) |
                      (static_cast<uint32_t>(cmd[4]) << 8)  |
                      static_cast<uint32_t>(cmd[5]);
                blocks = (static_cast<uint32_t>(cmd[7]) << 8) | static_cast<uint32_t>(cmd[8]);
                if (blocks == 0 && cbw_.dCBWDataTransferLength > 0) {
                    blocks = cbw_.dCBWDataTransferLength / disk_.sector_size();
                }
            } else {
                lba = ((static_cast<uint32_t>(cmd[1]) & 0x1F) << 16) |
                      (static_cast<uint32_t>(cmd[2]) << 8) |
                      static_cast<uint32_t>(cmd[3]);
                blocks = (cmd[4] == 0) ? 256 : cmd[4];
            }

            if (blocks == 0) {
                prepare_csw(CswStatus::CMD_PASSED, 0);
                return;
            }

            if (lba + blocks > disk_.sector_count()) {
                std::cerr << "[SCSI] READ past capacity: LBA " << lba << " + " << blocks << " > " << disk_.sector_count() << "\n";
                set_sense(SenseKey::ILLEGAL_REQUEST, Asc::ADDRESS_OUT_OF_RANGE);
                prepare_csw(CswStatus::CMD_FAILED, cbw_.dCBWDataTransferLength);
                stall_in_ = true;
                return;
            }

            const size_t total_bytes = static_cast<size_t>(blocks) * disk_.sector_size();
            in_data_buffer_.resize(total_bytes);
            if (!disk_.read_sectors(lba, blocks, in_data_buffer_)) {
                std::cerr << "[SCSI] Disk read error LBA " << lba << "\n";
                set_sense(SenseKey::HARDWARE_ERROR, Asc::UNRECOVERED_READ_ERROR);
                prepare_csw(CswStatus::CMD_FAILED, cbw_.dCBWDataTransferLength);
                stall_in_ = true;
                return;
            }

            state_ = State::DATA_IN;
            prepare_csw(CswStatus::CMD_PASSED, cbw_.dCBWDataTransferLength - total_bytes);
            break;
        }

        case ScsiCmd::WRITE_10:
        case ScsiCmd::WRITE_6: {
            if (!disk_.is_ready()) {
                set_sense(SenseKey::NOT_READY, Asc::MEDIUM_NOT_PRESENT);
                prepare_csw(CswStatus::CMD_FAILED, cbw_.dCBWDataTransferLength);
                stall_out_ = true;
                return;
            }

            uint32_t lba = 0;
            uint32_t blocks = 0;
            if (opcode == ScsiCmd::WRITE_10) {
                lba = (static_cast<uint32_t>(cmd[2]) << 24) |
                      (static_cast<uint32_t>(cmd[3]) << 16) |
                      (static_cast<uint32_t>(cmd[4]) << 8)  |
                      static_cast<uint32_t>(cmd[5]);
                blocks = (static_cast<uint32_t>(cmd[7]) << 8) | static_cast<uint32_t>(cmd[8]);
                if (blocks == 0 && cbw_.dCBWDataTransferLength > 0) {
                    blocks = cbw_.dCBWDataTransferLength / disk_.sector_size();
                }
            } else {
                lba = ((static_cast<uint32_t>(cmd[1]) & 0x1F) << 16) |
                      (static_cast<uint32_t>(cmd[2]) << 8) |
                      static_cast<uint32_t>(cmd[3]);
                blocks = (cmd[4] == 0) ? 256 : cmd[4];
            }

            if (blocks == 0) {
                prepare_csw(CswStatus::CMD_PASSED, 0);
                return;
            }

            if (lba + blocks > disk_.sector_count()) {
                std::cerr << "[SCSI] WRITE past capacity: LBA " << lba << " + " << blocks << " > " << disk_.sector_count() << "\n";
                set_sense(SenseKey::ILLEGAL_REQUEST, Asc::ADDRESS_OUT_OF_RANGE);
                prepare_csw(CswStatus::CMD_FAILED, cbw_.dCBWDataTransferLength);
                stall_out_ = true;
                return;
            }

            write_lba_ = lba;
            write_remaining_bytes_ = blocks * disk_.sector_size();
            state_ = State::DATA_OUT;
            break;
        }

        case ScsiCmd::PREVENT_ALLOW_REMOVAL:
        case ScsiCmd::START_STOP_UNIT:
        case ScsiCmd::VERIFY_10:
        case ScsiCmd::SYNCHRONIZE_CACHE_10: {
            prepare_csw(CswStatus::CMD_PASSED, 0);
            break;
        }

        default: {
            std::cerr << "[SCSI] Unsupported opcode: 0x" << std::hex << (int)cmd[0] << std::dec << "\n";
            set_sense(SenseKey::ILLEGAL_REQUEST, Asc::INVALID_CDB);
            prepare_csw(CswStatus::CMD_FAILED, cbw_.dCBWDataTransferLength);
            if (cbw_.dCBWDataTransferLength > 0) {
                if (cbw_.bmCBWFlags & 0x80) {
                    stall_in_ = true;
                } else {
                    stall_out_ = true;
                }
            }
            break;
        }
    }
}

bool MscScsiEngine::on_data_out(std::span<const uint8_t> data) {
    if (state_ != State::DATA_OUT) {
        std::cerr << "[MSC] on_data_out called in unexpected state\n";
        return false;
    }

    const uint32_t sector_sz = disk_.sector_size();
    const uint32_t blocks = (data.size() + sector_sz - 1) / sector_sz;

    if (!disk_.write_sectors(write_lba_, blocks, data)) {
        std::cerr << "[SCSI] Disk write error at LBA " << write_lba_ << "\n";
        set_sense(SenseKey::HARDWARE_ERROR, Asc::WRITE_FAULT);
        prepare_csw(CswStatus::CMD_FAILED, write_remaining_bytes_);
        stall_out_ = true;
        return false;
    }

    write_lba_ += blocks;
    if (data.size() >= write_remaining_bytes_) {
        write_remaining_bytes_ = 0;
        prepare_csw(CswStatus::CMD_PASSED, 0);
    } else {
        write_remaining_bytes_ -= data.size();
    }
    return true;
}

std::span<const uint8_t> MscScsiEngine::get_in_data(size_t max_len) {
    if (in_data_offset_ >= in_data_buffer_.size()) {
        return {};
    }
    const size_t available = in_data_buffer_.size() - in_data_offset_;
    const size_t chunk = std::min(available, max_len);
    return std::span<const uint8_t>(in_data_buffer_.data() + in_data_offset_, chunk);
}

void MscScsiEngine::consume_in_data(size_t bytes_sent) {
    in_data_offset_ += bytes_sent;
    if (in_data_offset_ >= in_data_buffer_.size()) {
        state_ = State::CSW_PENDING;
    }
}

std::span<const uint8_t> MscScsiEngine::get_csw() {
    state_ = State::IDLE;
    return std::span<const uint8_t>(reinterpret_cast<const uint8_t*>(&csw_), BBB_CSW_LENGTH);
}

void MscScsiEngine::prepare_csw(CswStatus status, uint32_t residue) {
    csw_.dCSWSignature   = BBB_CSW_SIGNATURE;
    csw_.dCSWTag         = cbw_.dCBWTag;
    csw_.dCSWDataResidue = residue;
    csw_.bCSWStatus      = static_cast<uint8_t>(status);

    if (in_data_buffer_.empty()) {
        state_ = State::CSW_PENDING;
    }
}

} // namespace msc
