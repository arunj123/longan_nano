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
 * RX FIFO: 128 words (512 bytes = exactly 1 sector, ample space for OUT data packets + status)
 * TX0 FIFO (EP0 IN): 64 words (256 bytes = 4x 64-byte EP0 MPS)
 * TX1 FIFO (EP1 MSC Bulk IN): 128 words (512 bytes = exactly 1 sector, eliminates memory aliasing)
 * TX2 FIFO: 0 words (Unused)
 * TX3 FIFO: 0 words (Unused)
 * Total: 128 + 64 + 128 = 320 words (100% exact hardware fit)
 */
#define RX_FIFO_FS_SIZE             128U
#define TX0_FIFO_FS_SIZE            64U
#define TX1_FIFO_FS_SIZE            128U
#define TX2_FIFO_FS_SIZE            0U
#define TX3_FIFO_FS_SIZE            0U

#endif /* USB_CONF_H */
