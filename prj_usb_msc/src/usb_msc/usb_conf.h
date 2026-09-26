#ifndef USB_CONF_H
#define USB_CONF_H

#include <cstdint>
#include <cstdlib>

#define USB_SOF_OUTPUT              1U
#define USB_LOW_POWER               0U

#define USE_USB_FS
#define USE_DEVICE_MODE

/*
 * Total available FIFO size for GD32VF103 is 1.25 KB (1280 bytes = 320 words).
 * RX FIFO: shared for all OUT endpoints (EP0 OUT, EP1 MSC OUT).
 * TX FIFOs: dedicated per IN endpoint.
 *
 * RX FIFO: 96 words (384 bytes = 6x 64B packets, ample for OUT data packets + status)
 * TX0 FIFO (EP0 IN): 32 words (128 bytes = 2x 64B EP0 MPS, double-buffered)
 * TX1 FIFO (EP1 MSC Bulk IN): 192 words (768 bytes = 512B sector + 256B headroom, eliminates FIFO wrap/drop)
 * TX2 FIFO: 0 words (Unused)
 * TX3 FIFO: 0 words (Unused)
 * Total: 96 + 32 + 192 = 320 words (100% exact hardware fit)
 */
#define RX_FIFO_FS_SIZE             96U
#define TX0_FIFO_FS_SIZE            32U
#define TX1_FIFO_FS_SIZE            192U
#define TX2_FIFO_FS_SIZE            0U
#define TX3_FIFO_FS_SIZE            0U

#endif /* USB_CONF_H */
