/*!
    \file    usb_types.h
    \brief   Centralized type definitions for the composite USB device

    \version 2025-02-10, firmware for GD32VF103
*/

#ifndef USB_TYPES_H
#define USB_TYPES_H

#include <cstdint>

// Ensure structs are packed correctly, matching hardware/protocol requirements
#pragma pack(1)

namespace usb {

// From usb_ch9_std.h
struct DescHeader {
    uint8_t bLength;
    uint8_t bDescriptorType;
};

struct UsbRequest {
    uint8_t  bmRequestType;
    uint8_t  bRequest;
    uint16_t wValue;
    uint16_t wIndex;
    uint16_t wLength;
};

// Standard USB Request Codes (bRequest)
enum class StdReq : uint8_t {
    GET_STATUS        = 0x00,
    CLEAR_FEATURE     = 0x01,
    SET_FEATURE       = 0x03,
    SET_ADDRESS       = 0x05,
    GET_DESCRIPTOR    = 0x06,
    SET_DESCRIPTOR    = 0x07,
    GET_CONFIGURATION = 0x08,
    SET_CONFIGURATION = 0x09,
    GET_INTERFACE     = 0x0A,
    SET_INTERFACE     = 0x0B,
    SYNCH_FRAME       = 0x0C
};

// USB Request Status for internal handling
enum class ReqStatus : uint8_t {
    REQ_SUPP,
    REQ_NOTSUPP
};

// String Descriptor Index
enum StrIdx : uint8_t {
    STR_IDX_LANGID  = 0x00U,
    STR_IDX_MFC     = 0x01U,
    STR_IDX_PRODUCT = 0x02U,
    STR_IDX_SERIAL  = 0x03U,
    STR_IDX_CONFIG  = 0x04U,
    STR_IDX_ITF     = 0x05U,
    STR_IDX_MAX     = 0xEFU
};

// Language ID
constexpr uint16_t ENG_LANGID = 0x0409U;

namespace hid {
    // From usb_hid.h
    constexpr uint8_t HID_CLASS = 0x03U;
    constexpr uint8_t DESC_TYPE_HID = 0x21U;
    constexpr uint8_t DESC_TYPE_REPORT = 0x22U;

    // HID Class-Specific Request Codes (bRequest)
    enum class HidReq : uint8_t {
        GET_REPORT   = 0x01,
        GET_IDLE     = 0x02,
        GET_PROTOCOL = 0x03,
        SET_REPORT   = 0x09,
        SET_IDLE     = 0x0A,
        SET_PROTOCOL = 0x0B
    };

    struct DescHid {
        DescHeader header;
        uint16_t bcdHID;
        uint8_t  bCountryCode;
        uint8_t  bNumDescriptors;
        uint8_t  bDescriptorType;
        uint16_t wDescriptorLength;
    };

    struct StandardHidHandler {
        uint32_t protocol;
        uint32_t idle_state;
        volatile uint8_t prev_transfer_complete;
    };

    struct CustomHidHandler {
        uint8_t data[2];
        uint8_t reportID;
        uint8_t idlestate;
        uint8_t protocol;
        volatile uint8_t prev_transfer_complete;
    };
} // namespace hid

namespace msc {
    // From usb_msc.h
    constexpr uint8_t MSC_CLASS = 0x08U;
    constexpr uint8_t MSC_SUBCLASS_SCSI = 0x06U;
    constexpr uint8_t MSC_PROTOCOL_BBB = 0x50U;
    constexpr uint8_t REQ_GET_MAX_LUN = 0xFEU;
    constexpr uint8_t REQ_BBB_RESET = 0xFFU;

    // From msc_bbb.h
    constexpr uint32_t BBB_CBW_SIGNATURE = 0x43425355U;
    constexpr uint32_t BBB_CSW_SIGNATURE = 0x53425355U;
    constexpr uint8_t BBB_CBW_LENGTH = 31U;
    constexpr uint8_t BBB_CSW_LENGTH = 13U;

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

    enum class CswStatus : uint8_t {
        CMD_PASSED = 0U,
        CMD_FAILED,
        PHASE_ERROR
    };

    enum class BbbState {
        BBB_IDLE = 0U,
        BBB_DATA_OUT,
        BBB_DATA_IN,
        BBB_LAST_DATA_IN,
        BBB_SEND_DATA
    };

    enum class BbbStatus {
        BBB_STATUS_NORMAL = 0U,
        BBB_STATUS_RECOVERY,
        BBB_STATUS_ERROR
    };

    // From msc_scsi.h
    namespace scsi {
        constexpr uint8_t SENSE_LIST_DEEPTH = 4U;

        enum class Command : uint8_t {
            TEST_UNIT_READY         = 0x00U,
            REQUEST_SENSE           = 0x03U,
            INQUIRY                 = 0x12U,
            MODE_SENSE_6            = 0x1AU,
            START_STOP_UNIT         = 0x1BU,
            ALLOW_MEDIUM_REMOVAL    = 0x1EU,
            READ_FORMAT_CAPACITIES  = 0x23U,
            READ_CAPACITY_10        = 0x25U,
            READ_10                 = 0x28U,
            WRITE_10                = 0x2AU,
            VERIFY_10               = 0x2FU,
            MODE_SENSE_10           = 0x5AU
        };

        enum class SenseKey : uint8_t {
            NO_SENSE            = 0x00,
            RECOVERED_ERROR     = 0x01,
            NOT_READY           = 0x02,
            MEDIUM_ERROR        = 0x03,
            HARDWARE_ERROR      = 0x04,
            ILLEGAL_REQUEST     = 0x05,
            UNIT_ATTENTION      = 0x06,
            DATA_PROTECT        = 0x07,
            BLANK_CHECK         = 0x08,
            VENDOR_SPECIFIC     = 0x09,
            COPY_ABORTED        = 0x0A,
            ABORTED_COMMAND     = 0x0B,
            VOLUME_OVERFLOW     = 0x0D,
            MISCOMPARE          = 0x0E
        };

        enum class Asc : uint8_t {
            INVALID_CDB                 = 0x20,
            INVALID_FIELD_IN_COMMAND    = 0x24,
            ADDRESS_OUT_OF_RANGE        = 0x21,
            MEDIUM_NOT_PRESENT          = 0x3A,
            WRITE_PROTECTED             = 0x27,
            UNRECOVERED_READ_ERROR      = 0x11,
            WRITE_FAULT                 = 0x03
        };

        struct SenseData {
            SenseKey key;
            uint8_t  ASC;
            uint8_t  ASCQ;
        };
    } // namespace scsi

    struct MscHandler {
        uint8_t bbb_data[MSC_MEDIA_PACKET_SIZE];
        uint8_t max_lun;
        BbbState bbb_state;
        BbbStatus bbb_status;
        uint32_t bbb_datalen;
        BbbCbw bbb_cbw;
        BbbCsw bbb_csw;
        uint8_t scsi_sense_head;
        uint8_t scsi_sense_tail;
        uint32_t scsi_blk_size[MEM_LUN_NUM];
        uint32_t scsi_blk_nbr[MEM_LUN_NUM];
        uint32_t scsi_blk_addr;
        uint32_t scsi_blk_len;
        scsi::SenseData scsi_sense[scsi::SENSE_LIST_DEEPTH];
    };

} // namespace msc

} // namespace usb

#pragma pack()

#endif // USB_TYPES_H