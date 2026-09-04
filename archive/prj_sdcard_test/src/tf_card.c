#include "tf_card.h"
#include <stdio.h> // Include for printf

#define FCLK_SLOW() { SPI_CTL0(SPI1) = (SPI_CTL0(SPI1) & ~SPI_CTL0_PSC) | SPI_PSC_256; } /* Set SCLK = PCLK / 256 */
#define FCLK_FAST() { SPI_CTL0(SPI1) = (SPI_CTL0(SPI1) & ~SPI_CTL0_PSC) | SPI_PSC_2; }   /* Set SCLK = PCLK / 2 */

#define CS_HIGH() gpio_bit_set(GPIOB, GPIO_PIN_12)
#define CS_LOW()  gpio_bit_reset(GPIOB, GPIO_PIN_12)

#define FF_FS_READONLY 0 // Enable write support

/* MMC/SD command */
#define CMD0	(0)			/* GO_IDLE_STATE */
#define CMD1	(1)			/* SEND_OP_COND (MMC) */
#define	ACMD41	(0x80+41)	/* SEND_OP_COND (SDC) */
#define CMD8	(8)			/* SEND_IF_COND */
#define CMD9	(9)			/* SEND_CSD */
#define CMD12	(12)		/* STOP_TRANSMISSION */
#define CMD16	(16)		/* SET_BLOCKLEN */
#define CMD17	(17)		/* READ_SINGLE_BLOCK */
#define CMD18	(18)		/* READ_MULTIPLE_BLOCK */
#define	ACMD23	(0x80+23)	/* SET_WR_BLK_ERASE_COUNT (SDC) */
#define CMD24	(24)		/* WRITE_BLOCK */
#define CMD25	(25)		/* WRITE_MULTIPLE_BLOCK */
#define CMD55	(55)		/* APP_CMD */
#define CMD58	(58)		/* READ_OCR */

static volatile DSTATUS Stat = STA_NOINIT;
static volatile UINT delay_timer1, delay_timer2;
static BYTE CardType;

/*-----------------------------------------------------------------------*/
/* Platform dependent SPI control functions                              */
/*-----------------------------------------------------------------------*/

static void init_spi(void)
{
    spi_parameter_struct spi_init_struct;

    rcu_periph_clock_enable(RCU_GPIOB);
    rcu_periph_clock_enable(RCU_SPI1);
    rcu_periph_clock_enable(RCU_AF);

    gpio_init(GPIOB, GPIO_MODE_AF_PP, GPIO_OSPEED_50MHZ, GPIO_PIN_13 | GPIO_PIN_15);
    gpio_init(GPIOB, GPIO_MODE_IPU, GPIO_OSPEED_50MHZ, GPIO_PIN_14); // Use Input Pull-Up for MISO
    gpio_init(GPIOB, GPIO_MODE_OUT_PP, GPIO_OSPEED_50MHZ, GPIO_PIN_12);

    CS_HIGH();

    spi_i2s_deinit(SPI1);
    spi_struct_para_init(&spi_init_struct);
    spi_init_struct.trans_mode           = SPI_TRANSMODE_FULLDUPLEX;
    spi_init_struct.device_mode          = SPI_MASTER;
    spi_init_struct.frame_size           = SPI_FRAMESIZE_8BIT;
    spi_init_struct.clock_polarity_phase = SPI_CK_PL_HIGH_PH_2EDGE;
    spi_init_struct.nss                  = SPI_NSS_SOFT;
    spi_init_struct.prescale             = SPI_PSC_256; // Start with a slow clock
    spi_init_struct.endian               = SPI_ENDIAN_MSB;
    spi_init(SPI1, &spi_init_struct);

    spi_enable(SPI1);
}

static BYTE xchg_spi(BYTE dat)
{
	while(RESET == spi_i2s_flag_get(SPI1, SPI_FLAG_TBE));
    spi_i2s_data_transmit(SPI1, dat);
	while(RESET == spi_i2s_flag_get(SPI1, SPI_FLAG_RBNE));
    return (BYTE)spi_i2s_data_receive(SPI1);
}

static void rcvr_spi_multi(BYTE *buff, UINT btr)
{
    for (UINT i = 0; i < btr; i++) {
        buff[i] = xchg_spi(0xFF);
    }
}

static void xmit_spi_multi(const BYTE *buff, UINT btx)
{
    for (UINT i = 0; i < btx; i++) {
        xchg_spi(buff[i]);
    }
}

/*-----------------------------------------------------------------------*/
/* Wait for card ready                                                   */
/*-----------------------------------------------------------------------*/

static int wait_ready(UINT wt)
{
	delay_timer2 = wt;
	do {
		if (xchg_spi(0xFF) == 0xFF) return 1;
	} while (delay_timer2);
	return 0;
}

/*-----------------------------------------------------------------------*/
/* Deselect card and release SPI                                         */
/*-----------------------------------------------------------------------*/

static void deselect (void)
{
	CS_HIGH();
	xchg_spi(0xFF);
}

/*-----------------------------------------------------------------------*/
/* Select card and wait for ready                                        */
/*-----------------------------------------------------------------------*/

static int _select (void)
{
	CS_LOW();
	if (wait_ready(500)) return 1;
	deselect();
	return 0;
}

/*-----------------------------------------------------------------------*/
/* Receive a data packet from the card                                   */
/*-----------------------------------------------------------------------*/

static int rcvr_datablock (BYTE *buff, UINT btr)
{
	BYTE token;
	delay_timer1 = 200;
	do {
		token = xchg_spi(0xFF);
	} while ((token == 0xFF) && delay_timer1);

	if(token != 0xFE) return 0;

	rcvr_spi_multi(buff, btr);
	xchg_spi(0xFF); xchg_spi(0xFF); // Discard CRC

	return 1;
}

/*-----------------------------------------------------------------------*/
/* Send a data packet to the card                                        */
/*-----------------------------------------------------------------------*/
#if FF_FS_READONLY == 0
static int xmit_datablock (const BYTE *buff, BYTE token)
{
    BYTE resp;
    if (!wait_ready(500)) return 0;

    xchg_spi(token);
    if (token != 0xFD) { // Is it data token?
        xmit_spi_multi(buff, 512); // Transmit the 512 byte data block
        xchg_spi(0xFF); xchg_spi(0xFF); // Dummy CRC
        resp = xchg_spi(0xFF); // Receive data response
        if ((resp & 0x1F) != 0x05) return 0; // If not accepted, return error
    }
    return 1;
}
#endif

/*-----------------------------------------------------------------------*/
/* Send a command packet to the card                                     */
/*-----------------------------------------------------------------------*/

static BYTE send_cmd (BYTE cmd, DWORD arg)
{
	BYTE n, res;

    // *** LOGGING ADDED HERE ***
    // Print the command being sent. The (cmd & 0x7F) handles ACMD representation.
    printf("CMD_TX -> CMD%u, ARG=0x%08lX\n", (unsigned int)(cmd & 0x7F), (unsigned long)arg);
    
	if (cmd & 0x80) {
		cmd &= 0x7F;
		res = send_cmd(CMD55, 0);
		if (res > 1) return res;
	}

	if (cmd != CMD12) {
		deselect();
		if (!_select()) return 0xFF;
	}

	xchg_spi(0x40 | cmd);
	xchg_spi((BYTE)(arg >> 24));
	xchg_spi((BYTE)(arg >> 16));
	xchg_spi((BYTE)(arg >> 8));
	xchg_spi((BYTE)arg);

	n = 0x01;
	if (cmd == CMD0) n = 0x95;
	if (cmd == CMD8) n = 0x87;
	xchg_spi(n);

	if (cmd == CMD12) xchg_spi(0xFF);
	n = 10;
	do {
		res = xchg_spi(0xFF);
	} while ((res & 0x80) && --n);

    // *** LOGGING ADDED HERE ***
    printf("CMD_RX <- 0x%02X\n", (unsigned int)res);
	return res;
}

// NOTE: This timer is required for the timeout logic in the driver.
void TIMER2_IRQHandler(void)
{
    if (RESET != timer_flag_get(TIMER2, TIMER_FLAG_UP)){
        timer_flag_clear(TIMER2, TIMER_FLAG_UP);
        if (delay_timer1 > 0x00U) delay_timer1--;
        if (delay_timer2 > 0x00U) delay_timer2--;
    }
}

/*--------------------------------------------------------------------------
   Public Functions
---------------------------------------------------------------------------*/

DSTATUS disk_initialize (BYTE drv)
{
	BYTE n, cmd, ty, ocr[4];

	if (drv) return STA_NOINIT;
	init_spi();
	delay_1ms(10); // Provide short delay for power stabilization

    // The start.S file defines TIMER2_IRQHandler. Ensure ECLIC is set up.
    eclic_enable_interrupt(TIMER2_IRQn);
    eclic_set_irq_priority(TIMER2_IRQn, 2);

	if (Stat & STA_NODISK) return Stat;

	FCLK_SLOW();
    CS_HIGH(); // CS must be high during the init clocks
	for (n = 10; n; n--) xchg_spi(0xFF);	/* Send 80 dummy clocks */

	ty = 0;
	if (send_cmd(CMD0, 0) == 1) {
		delay_timer1 = 1000;
		if (send_cmd(CMD8, 0x1AA) == 1) {
			for (n = 0; n < 4; n++) ocr[n] = xchg_spi(0xFF);
			if (ocr[2] == 0x01 && ocr[3] == 0xAA) {
				while (delay_timer1 && send_cmd(ACMD41, 1UL << 30));
				if (delay_timer1 && send_cmd(CMD58, 0) == 0) {
					for (n = 0; n < 4; n++) ocr[n] = xchg_spi(0xFF);
					ty = (ocr[0] & 0x40) ? CT_SD2 | CT_BLOCK : CT_SD2;
				}
			}
		} else {
			if (send_cmd(ACMD41, 0) <= 1) 	{
				ty = CT_SD1; cmd = ACMD41;
			} else {
				ty = CT_MMC; cmd = CMD1;
			}
			while (delay_timer1 && send_cmd(cmd, 0));
			if (!delay_timer1 || send_cmd(CMD16, 512) != 0) ty = 0;
		}
	}
	CardType = ty;
	deselect();

	if (ty) {
		FCLK_FAST();
		Stat &= ~STA_NOINIT;
	} else {
		Stat = STA_NOINIT;
	}
	return Stat;
}

DSTATUS disk_status (BYTE drv)
{
	if (drv) return STA_NOINIT;
	return Stat;
}

DRESULT disk_read (BYTE drv, BYTE *buff, DWORD sector, UINT count)
{
	if (drv || !count) return RES_PARERR;
	if (Stat & STA_NOINIT) return RES_NOTRDY;

	if (!(CardType & CT_BLOCK)) sector *= 512;

	if (count == 1) {
		if ((send_cmd(CMD17, sector) == 0) && rcvr_datablock(buff, 512)) count = 0;
	} else {
		if (send_cmd(CMD18, sector) == 0) {
			do {
				if (!rcvr_datablock(buff, 512)) break;
				buff += 512;
			} while (--count);
			send_cmd(CMD12, 0);
		}
	}
	deselect();
	return count ? RES_ERROR : RES_OK;
}

#if FF_FS_READONLY == 0
DRESULT disk_write (BYTE drv, const BYTE *buff, DWORD sector, UINT count)
{
	if (drv || !count) return RES_PARERR;
	if (Stat & STA_NOINIT) return RES_NOTRDY;
	if (Stat & STA_PROTECT) return RES_WRPRT;

	if (!(CardType & CT_BLOCK)) sector *= 512;

	if (count == 1) {
		if ((send_cmd(CMD24, sector) == 0) && xmit_datablock(buff, 0xFE)) count = 0;
	} else {
		if (CardType & CT_SDC) send_cmd(ACMD23, count);
		if (send_cmd(CMD25, sector) == 0) {
			do {
				if (!xmit_datablock(buff, 0xFC)) break;
				buff += 512;
			} while (--count);
			if (!xmit_datablock(0, 0xFD)) count = 1;
		}
	}
	deselect();
	return count ? RES_ERROR : RES_OK;
}
#endif

DRESULT disk_ioctl (BYTE drv, BYTE cmd, void *buff)
{
	DRESULT res;
	BYTE n, csd[16];
	DWORD csize;

	if (drv) return RES_PARERR;
	if (Stat & STA_NOINIT) return RES_NOTRDY;
	res = RES_ERROR;

	switch (cmd) {
	case CTRL_SYNC:
		if (_select()) {
            deselect();
            res = RES_OK;
        }
		break;
	case GET_SECTOR_COUNT:
		if ((send_cmd(CMD9, 0) == 0) && rcvr_datablock(csd, 16)) {
			if ((csd[0] >> 6) == 1) {
				csize = csd[9] + ((WORD)csd[8] << 8) + ((DWORD)(csd[7] & 63) << 16) + 1;
				*(DWORD*)buff = csize << 10;
			} else {
				n = (csd[5] & 15) + ((csd[10] & 128) >> 7) + ((csd[9] & 3) << 1) + 2;
				csize = (csd[8] >> 6) + ((WORD)csd[7] << 2) + ((WORD)(csd[6] & 3) << 10) + 1;
				*(DWORD*)buff = csize << (n - 9);
			}
			res = RES_OK;
		}
		break;
	case GET_BLOCK_SIZE:
		*(DWORD*)buff = 512;
		res = RES_OK;
		break;
	default:
		res = RES_PARERR;
	}
	deselect();
	return res;
}

void disk_timerproc (void)
{
    // This function is intended to be called from a 1ms system timer.
    // The systick handler can be used for this.
    if (delay_timer1) delay_timer1--;
    if (delay_timer2) delay_timer2--;
}