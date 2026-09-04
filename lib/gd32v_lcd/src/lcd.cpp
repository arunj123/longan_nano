#include "lcd.h"
#include "hal/rcu.hpp"
#include "hal/spi.hpp"
#include "hal/dma.hpp"
#include "hal/gpio.hpp"
#include "hal/time.hpp"
#include "hal/eclic.hpp"
#include "bsp/board.hpp"

extern "C" {
#include "gd32vf103.h"
}

// ------------------------------------------------------------------------
// Hardware aliases
// ------------------------------------------------------------------------
using Spi = hal::spi::Spi0;
using DmaTx = hal::dma::Dma0Channel2; // Fixed hardware map: SPI0_TX
using DmaRx = hal::dma::Dma0Channel1; // Fixed hardware map: SPI0_RX

using PinCs   = bsp::board::LcdCs;   // PB2
using PinDc   = bsp::board::LcdDc;   // PB0
using PinRst  = bsp::board::LcdRst;  // PB1
using PinSck  = bsp::board::LcdSck;  // PA5
using PinMosi = bsp::board::LcdMosi; // PA7

// ------------------------------------------------------------------------

enum class WaitStatus : uint8_t {
    None     = 0,
    ReadU24  = 1,
    WriteU24 = 2,
};

static WaitStatus g_waitStatus = WaitStatus::None;
static uint32_t   g_fbAddress  = 0;
static int        g_fbEnabled  = 0;
static uint32_t   g_dma_const_value = 0;

// ------------------------------------------------------------------------
// Internal helpers
// ------------------------------------------------------------------------

static inline void lcd_mode_cmd() noexcept {
    PinDc::reset();
}

static inline void lcd_mode_data() noexcept {
    PinDc::set();
}

static inline void lcd_cs_enable() noexcept {
    PinCs::reset();
}

static inline void lcd_cs_disable() noexcept {
    PinCs::set();
}

static void dma_send_u8(const void* src, uint32_t count) {
    Spi::wait_idle();
    lcd_mode_data();
    Spi::set_8bit();
    DmaTx::configure_tx(Spi::data_register_address(), reinterpret_cast<uintptr_t>(src), count, hal::dma::Width::Bits8, true);
    DmaTx::enable();
}

static void dma_send_u16(const void* src, uint32_t count) {
    Spi::wait_idle();
    lcd_mode_data();
    Spi::set_16bit();
    DmaTx::configure_tx(Spi::data_register_address(), reinterpret_cast<uintptr_t>(src), count, hal::dma::Width::Bits16, true);
    DmaTx::enable();
}

[[maybe_unused]] static void dma_send_const_u8(uint8_t data, uint32_t count) {
    Spi::wait_idle();
    g_dma_const_value = data;
    lcd_mode_data();
    Spi::set_8bit();
    DmaTx::configure_tx(Spi::data_register_address(), reinterpret_cast<uintptr_t>(&g_dma_const_value), count, hal::dma::Width::Bits8, false);
    DmaTx::enable();
}

static void dma_send_const_u16(uint16_t data, uint32_t count) {
    Spi::wait_idle();
    g_dma_const_value = data;
    lcd_mode_data();
    Spi::set_16bit();
    DmaTx::configure_tx(Spi::data_register_address(), reinterpret_cast<uintptr_t>(&g_dma_const_value), count, hal::dma::Width::Bits16, false);
    DmaTx::enable();
}

static void lcd_reg(uint8_t x) {
    Spi::wait_idle();
    Spi::set_8bit();
    lcd_mode_cmd();
    Spi::send_8(x);
}

static void lcd_u8(uint8_t x) {
    Spi::wait_idle();
    Spi::set_8bit();
    lcd_mode_data();
    Spi::send_8(x);
}

static inline void lcd_u8c(uint8_t x) {
    Spi::send_8(x);
}

static void lcd_u16(uint16_t x) {
    Spi::wait_idle();
    Spi::set_16bit();
    lcd_mode_data();
    Spi::send_16(x);
}

static inline void lcd_u16c(uint16_t x) {
    Spi::send_16(x);
}

static void lcd_set_addr(int x, int y, int w, int h) {
    lcd_reg(0x2a); // CASET
    lcd_u16(static_cast<uint16_t>(x + 1));
    lcd_u16c(static_cast<uint16_t>(x + w));
    lcd_reg(0x2b); // RASET
    lcd_u16(static_cast<uint16_t>(y + 26));
    lcd_u16c(static_cast<uint16_t>(y + h + 25));
    lcd_reg(0x2c); // RAMWR
}

// ------------------------------------------------------------------------
// Public API implementation (extern "C")
// ------------------------------------------------------------------------

extern "C" {

void lcd_init(void) {
    // 1. Enable peripheral clocks via modern hal::rcu
    hal::rcu::enable(hal::rcu::Peripheral::GpioA);
    hal::rcu::enable(hal::rcu::Peripheral::GpioB);
    hal::rcu::enable(hal::rcu::Peripheral::Afio);
    hal::rcu::enable(hal::rcu::Peripheral::Dma0);
    hal::rcu::enable(hal::rcu::Peripheral::Spi0);

    // 2. Initialize GPIO pins
    PinSck::init(hal::gpio::Mode::AlternatePushPull, hal::gpio::Speed::Speed50MHz);
    PinMosi::init(hal::gpio::Mode::AlternatePushPull, hal::gpio::Speed::Speed50MHz);
    PinDc::init(hal::gpio::Mode::OutputPushPull, hal::gpio::Speed::Speed50MHz);
    PinRst::init(hal::gpio::Mode::OutputPushPull, hal::gpio::Speed::Speed50MHz);
    PinCs::init(hal::gpio::Mode::OutputPushPull, hal::gpio::Speed::Speed50MHz);

    // Initial pin states: DC=0, RST=0, CS=1
    lcd_mode_cmd();
    PinRst::reset();
    lcd_cs_disable();

    // Hardware reset pulse
    hal::time::delay_ms(1);
    PinRst::set();
    hal::time::delay_ms(5);

    // 3. Reset and configure SPI0 and DMA
    Spi::disable();
    DmaRx::disable();
    DmaTx::disable();

    // Setup SPI0 as Master, 8-bit, CPOL=0, CPHA=0, Prescaler /8 (~12-13.5 MHz)
    Spi::init_master(hal::spi::Prescaler::Div8,
                     hal::spi::ClockPolarity::Low,
                     hal::spi::ClockPhase::Edge1,
                     hal::spi::FrameFormat::Bits8);
    Spi::enable_dma_tx();

    // Enable LCD controller CS
    lcd_cs_enable();

    // 4. Send ST7735 initialization sequence
    static const uint8_t init_sequence[] = {
        0x21, 0xff,                                                         // Inversion ON
        0xb1, 0x05, 0x3a, 0x3a, 0xff,                                     // Frame Rate (Normal)
        0xb2, 0x05, 0x3a, 0x3a, 0xff,                                     // Frame Rate (Idle)
        0xb3, 0x05, 0x3a, 0x3a, 0x05, 0x3a, 0x3a, 0xff,                   // Frame Rate (Partial)
        0xb4, 0x03, 0xff,                                                 // Display Inversion Control
        0xc0, 0x62, 0x02, 0x04, 0xff,                                     // Power Control 1
        0xc1, 0xc0, 0xff,                                                 // Power Control 2
        0xc2, 0x0d, 0x00, 0xff,                                           // Power Control 3
        0xc3, 0x8d, 0x6a, 0xff,                                           // Power Control 4
        0xc4, 0x8d, 0xee, 0xff,                                           // Power Control 5
        0xc5, 0x0e, 0xff,                                                 // VCOM Control 1
        0xe0, 0x10, 0x0e, 0x02, 0x03, 0x0e, 0x07, 0x02, 0x07, 0x0a, 0x12, 0x27, 0x37, 0x00, 0x0d, 0x0e, 0x10, 0xff, // Gamma '+'
        0xe1, 0x10, 0x0e, 0x03, 0x03, 0x0f, 0x06, 0x02, 0x08, 0x0a, 0x13, 0x26, 0x36, 0x00, 0x0d, 0x0e, 0x10, 0xff, // Gamma '-'
        0x3a, 0x55, 0xff,                                                 // Interface Pixel Format: 16-bit/pixel (RGB565)
        0x36, 0x78, 0xff,                                                 // Memory Data Access Control (Orientation)
        0x29, 0xff,                                                       // Display ON
        0x11, 0xff,                                                       // Sleep OUT
        0xff
    };

    for (const uint8_t* p = init_sequence; *p != 0xff; ) {
        lcd_reg(*p++);
        if (*p == 0xff) {
            p++;
            continue;
        }
        Spi::wait_idle();
        lcd_mode_data();
        while (*p != 0xff) {
            lcd_u8c(*p++);
        }
        p++; // skip terminal 0xff
    }

    // 5. Clear display to black
    lcd_clear(0);

    g_waitStatus = WaitStatus::None;
    g_fbAddress  = 0;
    g_fbEnabled  = 0;
}

void lcd_clear(uint16_t color) {
    if (g_fbEnabled) return;

    lcd_wait();
    lcd_set_addr(0, 0, LCD_WIDTH, LCD_HEIGHT);
    dma_send_const_u16(color, LCD_WIDTH * LCD_HEIGHT);
}

void lcd_setpixel(int x, int y, unsigned short int color) {
    if (g_fbEnabled) return;

    lcd_wait();
    lcd_set_addr(x, y, 1, 1);
    lcd_u8(static_cast<uint8_t>(color >> 8));
    lcd_u8c(static_cast<uint8_t>(color & 0xFF));
}

void lcd_fill_rect(int x, int y, int w, int h, uint16_t color) {
    if (g_fbEnabled || w <= 0 || h <= 0) return;

    lcd_wait();
    lcd_set_addr(x, y, w, h);
    dma_send_const_u16(color, static_cast<uint32_t>(w * h));
}

void lcd_rect(int x, int y, int w, int h, uint16_t color) {
    if (g_fbEnabled || w <= 0 || h <= 0) return;

    lcd_wait();
    // Top border
    lcd_fill_rect(x, y, w, 1, color);
    // Bottom border
    if (h > 1) {
        lcd_fill_rect(x, y + h - 1, w, 1, color);
    }
    // Left border
    if (h > 2) {
        lcd_fill_rect(x, y + 1, 1, h - 2, color);
        // Right border
        if (w > 1) {
            lcd_fill_rect(x + w - 1, y + 1, 1, h - 2, color);
        }
    }
}

void lcd_write_u16(int x, int y, int w, int h, const void* buffer) {
    if (g_fbEnabled || w <= 0 || h <= 0 || !buffer) return;

    lcd_wait();
    lcd_set_addr(x, y, w, h);
    dma_send_u16(buffer, static_cast<uint32_t>(w * h));
}

void lcd_write_u24(int x, int y, int w, int h, const void* buffer) {
    if (g_fbEnabled || w <= 0 || h <= 0 || !buffer) return;

    lcd_wait();
    lcd_reg(0x3a);  // COLMOD
    lcd_u8(0x66);   // RGB666 (transferred as 3 x 8b)
    lcd_set_addr(x, y, w, h);
    dma_send_u8(buffer, static_cast<uint32_t>(w * h * 3));
    g_waitStatus = WaitStatus::WriteU24;
}

void lcd_read_u24(int x, int y, int w, int h, void* buffer) {
    if (g_fbEnabled || w <= 0 || h <= 0 || !buffer) return;

    lcd_wait();

    // Send receive commands
    lcd_set_addr(x, y, w, h);
    lcd_reg(0x3a);  // COLMOD
    lcd_u8(0x66);   // RGB666 (transferred as 3 x 8b)
    lcd_reg(0x2e);  // RAMRD
    lcd_u8(0x00);   // Flush dummy first byte sent by display

    // Reconfigure SPI and DMA for receiving
    Spi::wait_idle();
    Spi::disable();
    lcd_mode_data();
    (void)Spi::read_raw(); // Clear RBNE

    // Configure SPI0 for bidirectional receive
    Spi::RegCTL0::write(Spi::CTL0_MSTMOD | Spi::CTL0_SWNSS | Spi::CTL0_SWNSSEN |
                        (static_cast<uint32_t>(hal::spi::Prescaler::Div8) << 3) |
                        Spi::CTL0_CKPL | Spi::CTL0_CKPH | Spi::CTL0_RO);
    Spi::enable_dma_rx();

    DmaRx::configure_rx(Spi::data_register_address(), reinterpret_cast<uintptr_t>(buffer), static_cast<uint32_t>(w * h * 3), hal::dma::Width::Bits8, true);
    DmaRx::enable();
    Spi::enable();
    g_waitStatus = WaitStatus::ReadU24;
}

int lcd_is_dma_busy(void) {
    return DmaTx::is_busy() || (Spi::RegSTAT::read() & Spi::STAT_TRANS);
}

void lcd_wait(void) {
    if (g_fbEnabled) return;

    // Wait for pending transmit DMA (DMA0_CH2) and SPI idle
    DmaTx::wait_complete();
    Spi::wait_idle();

    if (g_waitStatus == WaitStatus::None) return;

    if (g_waitStatus == WaitStatus::ReadU24) {
        DmaRx::wait_complete();
        DmaRx::disable();
        Spi::disable();
        lcd_cs_disable();

        // Restore normal SPI transmit mode
        Spi::init_master(hal::spi::Prescaler::Div8,
                         hal::spi::ClockPolarity::Low,
                         hal::spi::ClockPhase::Edge1,
                         hal::spi::FrameFormat::Bits8);
        Spi::enable_dma_tx();
        lcd_cs_enable();

        // Return ST7735 to RGB565 mode
        lcd_reg(0x3a);
        lcd_u8(0x55);

        g_waitStatus = WaitStatus::None;
        return;
    }

    if (g_waitStatus == WaitStatus::WriteU24) {
        Spi::wait_idle();
        lcd_reg(0x3a);
        lcd_u8(0x55);

        g_waitStatus = WaitStatus::None;
        return;
    }
}

// ------------------------------------------------------------------------
// Framebuffer functions
// ------------------------------------------------------------------------

void DMA0_Channel2_IRQHandler(void) {
    DmaTx::clear_all_flags();

    if (g_fbEnabled) {
        lcd_set_addr(0, 0, LCD_WIDTH, LCD_HEIGHT);
        dma_send_u16(reinterpret_cast<const void*>(g_fbAddress), LCD_FRAMEBUFFER_PIXELS);
    } else {
        DmaTx::disable_interrupt();
    }
}

void lcd_fb_setaddr(const void* buffer) {
    if (g_fbEnabled) return;
    g_fbAddress = reinterpret_cast<uint32_t>(buffer);
}

void lcd_fb_enable(void) {
    if (g_fbEnabled || !g_fbAddress) return;

    lcd_wait();
    Spi::wait_idle();
    g_fbEnabled = 1;

    hal::eclic::Eclic::enable_global_interrupts();
    hal::eclic::Eclic::enable(DMA0_Channel2_IRQn);

    DmaTx::disable();
    DmaTx::clear_all_flags();
    DmaTx::enable_interrupt();

    lcd_set_addr(0, 0, LCD_WIDTH, LCD_HEIGHT);
    dma_send_u16(reinterpret_cast<const void*>(g_fbAddress), LCD_FRAMEBUFFER_PIXELS);
}

void lcd_fb_disable(void) {
    if (!g_fbEnabled) return;

    g_fbEnabled = 0;
    while (DmaTx::RegCTL::read() & DmaTx::CTL_FTFIE);
}

} // extern "C"
