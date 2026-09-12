#ifndef USBD_CONF_H
#define USBD_CONF_H

#include "usb_conf.h"

#define USBD_CFG_MAX_NUM                   1U
#define USBD_ITF_MAX_NUM                   1U

#define USB_STR_DESC_MAX_SIZE              64U
#define USB_STRING_COUNT                   4U

/* Mass Storage Class Endpoints & Settings */
#define MSC_INTERFACE                      0x00U

#define MSC_IN_EP                          EP_IN(1U)
#define MSC_OUT_EP                         EP_OUT(1U)
#define MSC_IN_PACKET                      64U
#define MSC_OUT_PACKET                     64U

#define MSC_MEDIA_PACKET_SIZE              512U  /* Media buffer: 512 bytes (1 sector, fits 100% in 768B TX1 FIFO) */
#define MEM_LUN_NUM                        1U    /* 1 LUN for onboard MicroSD slot */

#endif /* USBD_CONF_H */
