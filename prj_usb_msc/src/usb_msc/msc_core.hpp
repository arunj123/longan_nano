#pragma once

#include <cstdint>
#include <cstddef>
#include "drivers/usb/usb_core.hpp"
#include "drivers/usb/usbd_core.h"

inline constexpr uint32_t MSC_PACED_64B_XFER = 0U;

namespace msc {

struct MscTraceEntry {
    uint8_t  type; // 1 = CBW recvd, 2 = SCSI result, 3 = CSW sent, 4 = Abort, 5 = Class Req
    uint8_t  opcode;
    uint8_t  val8;
    uint8_t  status;
    uint32_t val32_1;
    uint32_t val32_2;
    uint8_t  cdb[10];
};

inline constexpr size_t MSC_TRACE_MAX = 64;
extern MscTraceEntry g_msc_trace[MSC_TRACE_MAX];
extern volatile uint8_t g_msc_trace_head;
extern volatile uint8_t g_msc_trace_tail;

void msc_trace_record(uint8_t type, uint8_t opcode, uint8_t val8, uint8_t status, uint32_t val32_1, uint32_t val32_2, const uint8_t *cdb = nullptr);

uint8_t init(usb_dev *udev, uint8_t config_index);
uint8_t deinit(usb_dev *udev, uint8_t config_index);
uint8_t req_handler(usb_dev *udev, usb_req *req);
uint8_t data_in(usb_dev *udev, uint8_t ep_num);
uint8_t data_out(usb_dev *udev, uint8_t ep_num);
void poll(usb_core_driver *udev);
bool is_idle();

extern usb_class_core msc_class;

} // namespace msc
