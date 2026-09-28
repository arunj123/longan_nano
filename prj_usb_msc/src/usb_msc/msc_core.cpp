#include "msc_core.hpp"
#include "msc_disk.hpp"
#include "usbd_descriptors.hpp"
#include "drivers/usb/drv_usbd_int.h"
#include "usb_conf.h"
#include "hal/time.hpp"
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
    SEND_DATA,
    SEND_ZLP,
    STATUS_PENDING,
    SEND_CSW   // CSW queued on IN EP; re-arm CBW OUT when this IN completes
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
    REPORT_LUNS            = 0xA0,
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
    alignas(4) uint8_t cbw_buf[64]; // 64 bytes — matches hardware OUT endpoint max packet size
    alignas(4) uint8_t csw_buf[16]; // Aligned to 4 bytes, padded to 16 bytes for 4-word FIFO write
    volatile BbbState state{BbbState::IDLE};
    volatile BbbStatus status{BbbStatus::NORMAL};
    uint8_t *p_data{nullptr};
    uint32_t data_len{0};
    uint32_t lba{0};
    uint32_t total_blocks{0};
    volatile uint32_t remaining_bytes{0};
    uint32_t csw_residue{0};   // BOT-correct residue = dCBWDataTransferLength - actual_transferred
    SenseKey sense_key{SenseKey::NO_SENSE};
    Asc sense_asc{Asc::NO_ASC};
    uint8_t max_lun{0};
    CswStatus csw_status{CswStatus::CMD_PASSED};
    hal::time::Instant data_done_time{};
    volatile bool csw_pending{false};   // True only when a stalled transfer owes a CSW to CLEAR_FEATURE
    volatile bool need_read{false};
    hal::time::Instant need_read_time{};
    volatile bool need_write{false};
    volatile uint16_t sub_offset{0};
    volatile uint16_t sub_remaining{0};

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
    if (next_head == g_msc_trace_tail) {
        g_msc_trace_tail = (g_msc_trace_tail + 1U) % MSC_TRACE_MAX;
    }
    auto &entry = g_msc_trace[g_msc_trace_head];
    entry = {type, opcode, val8, status, val32_1, val32_2, {}};
    if (cdb) {
        std::memcpy(entry.cdb, cdb, 10);
    }
    g_msc_trace_head = next_head;
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

static void msc_ep_send(usb_core_driver *udev, uint8_t *pbuf, uint32_t len) {
    usbd_ep_send(udev, MSC_IN_EP, pbuf, len);
}

static void csw_send(usb_core_driver *udev, CswStatus status) {
    auto *er_in1 = udev->regs.er_in[EP_ID(MSC_IN_EP)];
    usb_transc *transc = &udev->dev.transc_in[EP_ID(MSC_IN_EP)];
    ep1_debug_record(5, static_cast<uint32_t>(status), er_in1->DIEPLEN, er_in1->DIEPTFSTAT, transc->xfer_count, transc->xfer_len, er_in1->DIEPCTL);

    /* Defensive cleanup of any stale data-phase flags before arming the 13-byte CSW transfer */
    er_in1->DIEPINTF = DIEPINTF_TF | DIEPINTF_EPDIS | DIEPINTF_TXFUD;

    auto &csw = ctx.csw();
    auto &cbw = ctx.cbw();
    csw.dCSWSignature = BBB_CSW_SIGNATURE;
    csw.dCSWTag = cbw.dCBWTag;
    csw.dCSWDataResidue = ctx.csw_residue; // BOT-correct residue (set by command handler)
    csw.bCSWStatus = static_cast<uint8_t>(status);
    ctx.state = BbbState::SEND_CSW;
    ctx.csw_pending = false;

    msc_trace_record(3, cbw.CBWCB[0], static_cast<uint8_t>(status), 0, csw.dCSWDataResidue, 0);

    msc_ep_send(udev, ctx.csw_buf, BBB_CSW_LENGTH);
}

static void data_send(usb_core_driver *udev, uint8_t *pbuf, uint32_t len) {
    auto &cbw = ctx.cbw();
    len = USB_MIN(cbw.dCBWDataTransferLength, len);
    ctx.csw_residue = cbw.dCBWDataTransferLength - len; // residue = requested - actually sent
    ctx.remaining_bytes = 0; // All data for this command is dispatched; no subsequent sectors
    ctx.p_data = pbuf;
#if MSC_PACED_64B_XFER
    ctx.sub_offset = 0;
    ctx.sub_remaining = static_cast<uint16_t>(len);
    uint32_t chunk = USB_MIN(ctx.sub_remaining, 64U);
    ctx.sub_offset = static_cast<uint16_t>(ctx.sub_offset + chunk);
    ctx.sub_remaining = static_cast<uint16_t>(ctx.sub_remaining - chunk);
    if (ctx.csw_residue > 0 && (len > 0) && ((len % MSC_IN_PACKET) == 0)) {
        ctx.state = BbbState::SEND_ZLP;
    } else {
        ctx.state = BbbState::SEND_DATA;
    }
    msc_ep_send(udev, pbuf, chunk);
#else
    if (ctx.csw_residue > 0 && (len > 0) && ((len % MSC_IN_PACKET) == 0)) {
        ctx.state = BbbState::SEND_ZLP;
    } else {
        ctx.state = BbbState::SEND_DATA;
    }
    msc_ep_send(udev, pbuf, len);
#endif
}

static void abort_transfer(usb_core_driver *udev) {
    auto &cbw = ctx.cbw();
    ctx.need_read = false;
    ctx.need_write = false;
    msc_trace_record(4, cbw.CBWCB[0], static_cast<uint8_t>(ctx.sense_key), static_cast<uint8_t>(ctx.sense_asc), ctx.lba, ctx.remaining_bytes, cbw.CBWCB);
    ctx.csw_residue = ctx.remaining_bytes; // BOT-compliant: residue reflects untransferred bytes
    if (cbw.dCBWDataTransferLength == 0) {
        // No data phase: send CSW directly (no STALL, no CLEAR_FEATURE expected)
        csw_send(udev, CswStatus::CMD_FAILED);
        return;
    }
    if ((cbw.bmCBWFlags == 0) && (ctx.status == BbbStatus::NORMAL)) {
        usbd_ep_stall(udev, MSC_OUT_EP);
    }
    // Stall IN: host will CLEAR_FEATURE(0x81) and then expect a CMD_FAILED CSW
    usbd_ep_stall(udev, MSC_IN_EP);
    ctx.csw_pending = true;
    if (ctx.status == BbbStatus::ERROR) {
        usbd_ep_recev(udev, MSC_OUT_EP, ctx.cbw_buf, BBB_CBW_LENGTH);
    }
}

static void ep1_in_hard_reset(usb_core_driver *udev);

static int8_t msc_process_read(usb_core_driver *udev) {
    uint32_t len = USB_MIN(ctx.remaining_bytes, MSC_MEDIA_PACKET_SIZE);
    uint32_t blocks = (len + 511U) / 512U;

    msc_trace_record(2, 0x28, static_cast<uint8_t>(blocks), 0, ctx.lba, ctx.remaining_bytes);

    if (msc_disk_read(ctx.media_buffer, ctx.lba, blocks) != 0) {
        printf("[MSC] msc_disk_read FAILED LBA %lu\n", static_cast<unsigned long>(ctx.lba));
        ep1_in_hard_reset(udev);
        set_sense(SenseKey::HARDWARE_ERROR, Asc::UNRECOVERED_READ_ERROR);
        return -1;
    }

    ctx.lba += blocks;
    ctx.remaining_bytes -= len;
    ctx.state = BbbState::DATA_IN;

#if MSC_PACED_64B_XFER
    ctx.sub_offset = 0;
    ctx.sub_remaining = static_cast<uint16_t>(len);
    uint32_t chunk = USB_MIN(ctx.sub_remaining, 64U);
    ctx.sub_offset = static_cast<uint16_t>(ctx.sub_offset + chunk);
    ctx.sub_remaining = static_cast<uint16_t>(ctx.sub_remaining - chunk);
    msc_ep_send(udev, ctx.media_buffer, chunk);
#else
    msc_ep_send(udev, ctx.media_buffer, len);
#endif
    return 0;
}

static int8_t msc_process_write(usb_core_driver *udev) {
    uint32_t len = USB_MIN(ctx.remaining_bytes, MSC_MEDIA_PACKET_SIZE);
    uint32_t blocks = (len + 511U) / 512U;

    msc_trace_record(2, 0x2A, static_cast<uint8_t>(blocks), 0, ctx.lba, ctx.remaining_bytes);

    if (msc_disk_write(ctx.media_buffer, ctx.lba, blocks) != 0) {
        set_sense(SenseKey::HARDWARE_ERROR, Asc::WRITE_FAULT);
        return -1;
    }

    ctx.lba += blocks;
    ctx.remaining_bytes -= len;

    if (ctx.remaining_bytes == 0) {
        csw_send(udev, CswStatus::CMD_PASSED);
    } else {
        uint32_t next_len = USB_MIN(ctx.remaining_bytes, MSC_MEDIA_PACKET_SIZE);
        usbd_ep_recev(udev, MSC_OUT_EP, ctx.media_buffer, next_len);
    }
    return 0;
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
            uint32_t to_transfer_r6 = USB_MIN(cbw.dCBWDataTransferLength, blocks * 512U);
            ctx.remaining_bytes = to_transfer_r6;
            ctx.csw_residue = cbw.dCBWDataTransferLength - to_transfer_r6;

            ctx.need_read = true;
            ctx.need_read_time = hal::time::Instant::now();
            udev->regs.er_in[EP_ID(MSC_IN_EP)]->DIEPCTL |= DEPCTL_SNAK;
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
            uint32_t to_transfer_w6 = USB_MIN(cbw.dCBWDataTransferLength, blocks * 512U);
            ctx.remaining_bytes = to_transfer_w6;
            ctx.csw_residue = cbw.dCBWDataTransferLength - to_transfer_w6;
            uint32_t rx_len_w6 = USB_MIN(ctx.remaining_bytes, MSC_MEDIA_PACKET_SIZE);
            usbd_ep_recev(udev, MSC_OUT_EP, ctx.media_buffer, rx_len_w6);
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
                    ctx.media_buffer[3] = 0x03; // Page length = 3 entries
                    ctx.media_buffer[4] = 0x00; // Page 0x00 supported
                    ctx.media_buffer[5] = 0x80; // Page 0x80 (Unit Serial Number) supported
                    ctx.media_buffer[6] = 0x83; // Page 0x83 (Device Identification) supported
                    ctx.data_len = USB_MIN(cbw.dCBWDataTransferLength, 7U);
                    return 0;
                } else if (page_code == 0x80) { // Unit Serial Number Page
                    ctx.media_buffer[0] = 0x00; // Direct access block device
                    ctx.media_buffer[1] = 0x80; // Page code
                    ctx.media_buffer[2] = 0x00; // Reserved
                    ctx.media_buffer[3] = 12;   // Page length
                    std::memcpy(&ctx.media_buffer[4], "LNMSC0000076", 12);
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
                    // Any unsupported VPD page (e.g. 0xB0 Block Limits) must return ILLEGAL_REQUEST
                    set_sense(SenseKey::ILLEGAL_REQUEST, Asc::INVALID_FIELD_IN_COMMAND);
                    return -1;
                }
            }
            uint16_t len = static_cast<uint16_t>(USB_MIN(cbw.dCBWDataTransferLength, sizeof(kInquiryData)));
            ctx.p_data = const_cast<uint8_t*>(kInquiryData);
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
            ctx.data_len = USB_MIN(cbw.dCBWDataTransferLength, 8U);
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
            ctx.data_len = USB_MIN(cbw.dCBWDataTransferLength, 12U);
            return 0;
        }

        case ScsiCmd::MODE_SENSE_6: {
            // Standard universal 4-byte header for USB Mass Storage flash drives
            // Mode Data Length = 3 (3 bytes follow), direct-access SBC, no write protection, 0 block descriptors
            std::memset(ctx.media_buffer, 0, 4);
            ctx.media_buffer[0] = 0x03; // 3 bytes follow
            ctx.media_buffer[1] = 0x00; // Medium type: SBC direct access
            ctx.media_buffer[2] = 0x00; // Device-specific: not write-protected
            ctx.media_buffer[3] = 0x00; // Block descriptor length: 0
            ctx.data_len = USB_MIN(cbw.dCBWDataTransferLength, 4U);
            return 0;
        }

        case ScsiCmd::MODE_SENSE_10: {
            // Standard universal 8-byte header for USB Mass Storage flash drives
            std::memset(ctx.media_buffer, 0, 8);
            ctx.media_buffer[1] = 0x06; // Mode data length (16-bit: 6 bytes follow)
            ctx.data_len = USB_MIN(cbw.dCBWDataTransferLength, 8U);
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
            // Use min(CBW length, actual data) so CSW residue is correct (BOT Case 5/8)
            uint32_t to_transfer_r10 = USB_MIN(cbw.dCBWDataTransferLength, blocks * 512U);
            ctx.remaining_bytes = to_transfer_r10;
            ctx.csw_residue = cbw.dCBWDataTransferLength - to_transfer_r10;

            ctx.need_read = true;
            ctx.need_read_time = hal::time::Instant::now();
            udev->regs.er_in[EP_ID(MSC_IN_EP)]->DIEPCTL |= DEPCTL_SNAK;
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
            // Use min(CBW length, actual data) so CSW residue is correct (BOT Case 5/8)
            uint32_t to_transfer_w10 = USB_MIN(cbw.dCBWDataTransferLength, blocks * 512U);
            ctx.remaining_bytes = to_transfer_w10;
            ctx.csw_residue = cbw.dCBWDataTransferLength - to_transfer_w10;

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
            if ((cmd[1] & 0x1F) == 0x10) { // READ_CAPACITY_16
                uint32_t blk_cnt = 0, blk_sz = 512;
                if (!msc_disk_get_capacity(blk_cnt, blk_sz)) {
                    set_sense(SenseKey::NOT_READY, Asc::MEDIUM_NOT_PRESENT);
                    return -1;
                }
                uint64_t last_lba = blk_cnt - 1;
                std::memset(ctx.media_buffer, 0, 32);
                ctx.media_buffer[0] = static_cast<uint8_t>(last_lba >> 56);
                ctx.media_buffer[1] = static_cast<uint8_t>(last_lba >> 48);
                ctx.media_buffer[2] = static_cast<uint8_t>(last_lba >> 40);
                ctx.media_buffer[3] = static_cast<uint8_t>(last_lba >> 32);
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

        case ScsiCmd::REPORT_LUNS: { // 0xA0 — mandatory for Linux SCSI scan
            std::memset(ctx.media_buffer, 0, 16);
            ctx.media_buffer[3] = 0x08; // LUN list length = 8 (one LUN entry)
            // Bytes 8–15: LUN 0 (all zeros = peripheral device addressing)
            ctx.data_len = USB_MIN(cbw.dCBWDataTransferLength, 16U);
            return 0;
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
    ctx.csw_residue     = cbw.dCBWDataTransferLength; // default: nothing transferred yet

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

    // Valid CBW received — exit any RECOVERY / ERROR state so CLEAR_FEATURE works
    ctx.status = BbbStatus::NORMAL;
    ctx.csw_pending = false; // new command cycle; any previous CSW debt is cleared
    ctx.data_done_time = hal::time::Instant{}; // reset data phase timer for framing guard
    ctx.p_data = ctx.media_buffer;
    g_msc_stats.last_activity = hal::time::Instant::now();

    if (process_scsi(udev) < 0) {
        abort_transfer(udev);
    } else if (ctx.state != BbbState::DATA_IN && ctx.state != BbbState::DATA_OUT) {
        if (ctx.data_len > 0) {
            data_send(udev, ctx.p_data, ctx.data_len);
        } else {
            csw_send(udev, CswStatus::CMD_PASSED);
        }
    }
}

uint8_t init(usb_dev *udev, uint8_t config_index) {
    (void)config_index;
    auto *pcore = reinterpret_cast<usb_core_driver*>(udev);

    usbd_ep_setup(pcore, &msc_config_desc.msc_epout);
    usbd_ep_setup(pcore, &msc_config_desc.msc_epin);

    ctx = MscContext{};

    usbd_fifo_flush(pcore, MSC_OUT_EP);
    usbd_fifo_flush(pcore, MSC_IN_EP);

    /* Explicitly reset data toggle to DATA0 and assert NAK on both bulk endpoints per USB MSC BOT */
    pcore->regs.er_in[EP_ID(MSC_IN_EP)]->DIEPCTL |= DEPCTL_SD0PID | DEPCTL_SNAK;
    pcore->regs.er_out[EP_ID(MSC_OUT_EP)]->DOEPCTL |= DEPCTL_SD0PID | DEPCTL_SNAK;

    usbd_ep_recev(pcore, MSC_OUT_EP, ctx.cbw_buf, BBB_CBW_LENGTH);
    msc_trace_record(5, 0xAA, config_index, 0, 0, 0);
    return USBD_OK;
}

uint8_t deinit(usb_dev *udev, uint8_t config_index) {
    (void)config_index;
    auto *pcore = reinterpret_cast<usb_core_driver*>(udev);
    ctx = MscContext{};
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

            case REQ_BBB_RESET: {
                // BOT reset resets the MSC transport state.
                ctx = MscContext{};
                ctx.status = BbbStatus::RECOVERY;

                msc_trace_record(5, REQ_BBB_RESET, 0, 0, 0, 0);

                // Clear any HALT left by the failed BOT transfer.
                usbd_ep_stall_clear(pcore, MSC_IN_EP);
                usbd_ep_stall_clear(pcore, MSC_OUT_EP);

                // Flush stale endpoint FIFO contents.
                usbd_fifo_flush(pcore, MSC_IN_EP);
                usbd_fifo_flush(pcore, MSC_OUT_EP);

                // Reset endpoint data PID after BOT reset.
                pcore->regs.er_in[EP_ID(MSC_IN_EP)]->DIEPCTL |= DEPCTL_SD0PID | DEPCTL_SNAK;
                pcore->regs.er_out[EP_ID(MSC_OUT_EP)]->DOEPCTL |= DEPCTL_SD0PID | DEPCTL_SNAK;

                // Ready for the next CBW.
                usbd_ep_recev(pcore, MSC_OUT_EP, ctx.cbw_buf, BBB_CBW_LENGTH);

                return USBD_OK;
            }

            default:
                return USBD_FAIL;
        }
    }

    // Standard request targeted to endpoint (Clear Feature STALL)
    if ((req->bmRequestType & USB_RECPTYPE_MASK) == USB_RECPTYPE_EP) {
        if (req->bRequest == USB_CLEAR_FEATURE) {
            uint8_t ep = static_cast<uint8_t>(req->wIndex);
            /* Explicitly clear STALL and reset data toggle to DATA0 per USB 2.0 §9.4.5 & MSC BOT §4.4 */
            if (ep & 0x80) {
                auto *diepctl = &pcore->regs.er_in[EP_ID(ep)]->DIEPCTL;
                *diepctl = (*diepctl & ~(DEPCTL_STALL | DEPCTL_CNAK)) | DEPCTL_SD0PID | DEPCTL_SNAK;
            } else {
                auto *doepctl = &pcore->regs.er_out[EP_ID(ep)]->DOEPCTL;
                *doepctl = (*doepctl & ~(DEPCTL_STALL | DEPCTL_SNAK)) | DEPCTL_SD0PID | DEPCTL_CNAK;
                /* Critical recovery step: After clearing HALT on Bulk-OUT, re-arm
                 * the endpoint to receive the next Command Block Wrapper (CBW).
                 * Without this, the endpoint stays in SNAK forever and NAKs the next CBW. */
                if (EP_ID(ep) == EP_ID(MSC_OUT_EP)) {
                    usbd_ep_recev(pcore, MSC_OUT_EP, ctx.cbw_buf, BBB_CBW_LENGTH);
                }
            }

            if (ctx.status == BbbStatus::ERROR) {
                usbd_ep_stall(pcore, MSC_IN_EP);
            } else if ((ep & 0x80) && ctx.csw_pending) {
                // Only send CSW if a previous abort_transfer stalled IN and owes a response.
                // Sending CSW on an arbitrary CLEAR_FEATURE (e.g. post-enumeration) would
                // inject a phantom CMD_FAILED before any command, breaking BOT sync.
                ctx.csw_pending = false;
                csw_send(pcore, CswStatus::CMD_FAILED);
            }
            ctx.status = BbbStatus::NORMAL;
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
#if MSC_PACED_64B_XFER
            if (ctx.sub_remaining > 0) {
                uint32_t chunk = USB_MIN(ctx.sub_remaining, 64U);
                uint8_t *pbuf = ctx.media_buffer + ctx.sub_offset;
                ctx.sub_offset = static_cast<uint16_t>(ctx.sub_offset + chunk);
                ctx.sub_remaining = static_cast<uint16_t>(ctx.sub_remaining - chunk);
                msc_ep_send(pcore, pbuf, chunk);
            } else if (ctx.remaining_bytes > 0) {
                // Next sector needed from SD card (handled asynchronously in poll())
                ctx.need_read = true;
                ctx.need_read_time = hal::time::Instant::now();
            } else {
                ctx.csw_status = CswStatus::CMD_PASSED;
                ctx.data_done_time = hal::time::Instant::now();
                ctx.state = BbbState::STATUS_PENDING;
            }
#else
            if (ctx.remaining_bytes > 0) {
                // Next sector needed from SD card (handled asynchronously in poll())
                ctx.need_read = true;
                ctx.need_read_time = hal::time::Instant::now();
                pcore->regs.er_in[EP_ID(MSC_IN_EP)]->DIEPCTL |= DEPCTL_SNAK;
            } else {
                ctx.csw_status = CswStatus::CMD_PASSED;
                ctx.data_done_time = hal::time::Instant::now();
                ctx.state = BbbState::STATUS_PENDING;
            }
#endif
            break;

        case BbbState::LAST_DATA_IN:
        case BbbState::SEND_DATA:
#if MSC_PACED_64B_XFER
            if (ctx.sub_remaining > 0) {
                uint32_t chunk = USB_MIN(ctx.sub_remaining, 64U);
                uint8_t *pbuf = ctx.p_data + ctx.sub_offset;
                ctx.sub_offset = static_cast<uint16_t>(ctx.sub_offset + chunk);
                ctx.sub_remaining = static_cast<uint16_t>(ctx.sub_remaining - chunk);
                msc_ep_send(pcore, pbuf, chunk);
                break;
            }
#endif
            ctx.csw_status = CswStatus::CMD_PASSED;
            ctx.data_done_time = hal::time::Instant::now();
            ctx.state = BbbState::STATUS_PENDING;
            break;

        case BbbState::SEND_ZLP:
            ctx.state = BbbState::SEND_DATA;
            msc_ep_send(pcore, ctx.media_buffer, 0);
            break;

        case BbbState::SEND_CSW:
            // CSW transmission has completed on the wire.
            ctx.state = BbbState::IDLE;
            // Prime the OUT endpoint to receive the next CBW from the host.
            usbd_ep_recev(pcore, MSC_OUT_EP, ctx.cbw_buf, BBB_CBW_LENGTH);
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
            // Data received from host — handle SD write asynchronously in poll()
            ctx.need_write = true;
            break;

        default:
            break;
    }
    return USBD_OK;
}

static void ep1_in_hard_reset(usb_core_driver *udev) {
    auto *er_in1 = udev->regs.er_in[EP_ID(MSC_IN_EP)];
    if (er_in1->DIEPCTL & DEPCTL_EPEN) {
        er_in1->DIEPCTL |= DEPCTL_EPD | DEPCTL_SNAK;
    }
    udev->regs.dr->DIEPFEINTEN &= ~(1U << EP_ID(MSC_IN_EP));
    er_in1->DIEPLEN = 0U;
    er_in1->DIEPINTF = DIEPINTF_TF | DIEPINTF_EPDIS | DIEPINTF_TXFUD;
    usbd_fifo_flush(udev, MSC_IN_EP);

    /* Preserve the architectural endpoint configuration bits.
     * Writing 0U to DIEPCTL would destroy TXFNUM (FIFO assignment),
     * EPTYPE (Bulk), MPL (64), and EPACT (active), permanently
     * bricking the IN endpoint until a hardware reset.
     *
     * EP1 is a Bulk IN endpoint on FIFO 1 with MPL = 64:
     *   DIEPCTL = 0x00488040  (EPACT | Bulk<<18 | 1<<22 | 64)
     */
    constexpr uint32_t kEp1BaseCfg =
        DEPCTL_EPACT |
        (static_cast<uint32_t>(USB_EPTYPE_BULK) << 18) |
        (1U << 22) |                    /* TXFNUM = 1 */
        MSC_IN_PACKET;                  /* MPL = 64 */

    er_in1->DIEPCTL = kEp1BaseCfg | DEPCTL_SD0PID | DEPCTL_SNAK;
}

void poll(usb_core_driver *udev) {
    const auto kIoWatchdog = hal::time::Duration::from_ms(5000);

    if (ctx.need_read) {
        ctx.need_read = false;
        if (msc_process_read(udev) < 0) {
            abort_transfer(udev);
        }
    } else if (ctx.need_write) {
        ctx.need_write = false;
        if (msc_process_write(udev) < 0) {
            abort_transfer(udev);
        }
    } else if (ctx.state == BbbState::STATUS_PENDING) {
        auto *er_in1 = udev->regs.er_in[EP_ID(MSC_IN_EP)];
        usb_transc *transc = &udev->dev.transc_in[EP_ID(MSC_IN_EP)];

        bool hw_done     = ((er_in1->DIEPLEN & (DEPLEN_TLEN | DEPLEN_PCNT)) == 0);
        bool fifo_empty  = ((er_in1->DIEPTFSTAT & DIEPTFSTAT_IEPTFS) >= TX1_FIFO_FS_SIZE);
        bool ep_disabled = !(er_in1->DIEPCTL & DEPCTL_EPEN);
        bool sw_done     = (transc->xfer_count == transc->xfer_len);
        bool no_pending  = ((er_in1->DIEPINTF & DIEPINTF_TF) == 0);

        /* CSW framing guard: Enforce >= 2.5 ms (2.5 USB frames) between data ACK
         * and CSW transmission. This ensures the host xHCI controller fully retires
         * the Data URB before Endpoint 1 presents the 13-byte CSW, preventing the Data
         * URB from absorbing the CSW as a short-packet premature completion. */
        static const hal::time::Duration kCswFramingGuard = hal::time::Duration::from_us(2500);
        bool guard_elapsed = (ctx.data_done_time.ticks != 0) &&
                             (hal::time::Instant::now() - ctx.data_done_time >= kCswFramingGuard);

        if (hw_done && fifo_empty && ep_disabled && sw_done && no_pending && guard_elapsed) {
            csw_send(udev, ctx.csw_status);
        } else if (ctx.data_done_time.ticks != 0 && hal::time::Instant::now() - ctx.data_done_time >= kIoWatchdog) {
            ep1_in_hard_reset(udev);
            set_sense(SenseKey::HARDWARE_ERROR, Asc::UNRECOVERED_READ_ERROR);
            abort_transfer(udev);
        }
    } else if (ctx.state == BbbState::DATA_IN) {
        if (ctx.need_read_time.ticks != 0 && hal::time::Instant::now() - ctx.need_read_time >= kIoWatchdog) {
            ep1_in_hard_reset(udev);
            set_sense(SenseKey::HARDWARE_ERROR, Asc::UNRECOVERED_READ_ERROR);
            abort_transfer(udev);
        }
    }
}

bool is_idle() {
    return (ctx.state == BbbState::IDLE) && !ctx.need_read && !ctx.need_write;
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
