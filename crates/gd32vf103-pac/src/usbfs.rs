use crate::VolatileCell;

pub const USBFS_BASE: usize = 0x5000_0000;

#[repr(C)]
pub struct UsbfsGlobalRegs {
    pub gotgcs: VolatileCell<u32>,              // 0x000 OTG Control and Status
    pub gotgintf: VolatileCell<u32>,            // 0x004 OTG Interrupt Flag
    pub gahbcs: VolatileCell<u32>,              // 0x008 Core AHB Configuration
    pub gusbcs: VolatileCell<u32>,              // 0x00C Core USB Configuration
    pub grstctl: VolatileCell<u32>,             // 0x010 Core Reset Control
    pub gintf: VolatileCell<u32>,               // 0x014 Core Interrupt Flag
    pub ginten: VolatileCell<u32>,              // 0x018 Core Interrupt Mask/Enable
    pub grstatr: VolatileCell<u32>,             // 0x01C Receive Status Debug Read
    pub grstatp: VolatileCell<u32>,             // 0x020 Receive Status Pop
    pub grflen: VolatileCell<u32>,              // 0x024 Receive FIFO Size
    pub diep0tflen_hnptflen: VolatileCell<u32>, // 0x028 EP0 / Non-Periodic TX FIFO Size
    pub hnptfqstat: VolatileCell<u32>,          // 0x02C Non-Periodic TX FIFO Status
    pub _reserved0: [u32; 2],                  // 0x030 - 0x034
    pub gccfg: VolatileCell<u32>,               // 0x038 Global Core Configuration
    pub cid: VolatileCell<u32>,                 // 0x03C Core ID
    pub _reserved1: [u32; 48],                 // 0x040 - 0x0FF
    pub hptflen: VolatileCell<u32>,             // 0x100 Host Periodic TX FIFO Size
    pub dieptflen: [VolatileCell<u32>; 15],     // 0x104 - 0x13C Device IN EP TX FIFO Sizes (index 0 = EP1)
}

#[repr(C)]
pub struct UsbfsDeviceRegs {
    pub dcfg: VolatileCell<u32>,                // 0x00 (0x800) Device Configuration
    pub dctl: VolatileCell<u32>,                // 0x04 (0x804) Device Control
    pub dstat: VolatileCell<u32>,               // 0x08 (0x808) Device Status
    pub _reserved0: u32,                       // 0x0C
    pub diepinten: VolatileCell<u32>,           // 0x10 (0x810) Device IN EP Common Interrupt Mask
    pub doepinten: VolatileCell<u32>,           // 0x14 (0x814) Device OUT EP Common Interrupt Mask
    pub daepint: VolatileCell<u32>,             // 0x18 (0x818) Device All Endpoints Interrupt
    pub daepinten: VolatileCell<u32>,           // 0x1C (0x81C) Device All Endpoints Interrupt Mask
    pub _reserved1: [u32; 2],                  // 0x20 - 0x24
    pub dvbusdt: VolatileCell<u32>,             // 0x28 (0x828) Device VBUS Discharge Time
    pub dvbuspt: VolatileCell<u32>,             // 0x2C (0x82C) Device VBUS Pulsing Time
    pub dthrctl: VolatileCell<u32>,             // 0x30 (0x830) Device Threshold Control
    pub diepfeinten: VolatileCell<u32>,         // 0x34 (0x834) Device IN EP FIFO Empty Interrupt Mask
    pub dep1int: VolatileCell<u32>,             // 0x38 (0x838) Device Dedicated EP1 Interrupt
    pub dep1inten: VolatileCell<u32>,           // 0x3C (0x83C) Device Dedicated EP1 Interrupt Mask
    pub _reserved2: u32,                       // 0x40
    pub diep1inten: VolatileCell<u32>,          // 0x44 (0x844) Device IN EP1 Interrupt Mask
    pub _reserved3: [u32; 15],                 // 0x48 - 0x80
    pub doep1inten: VolatileCell<u32>,          // 0x84 (0x884) Device OUT EP1 Interrupt Mask
}

#[repr(C)]
pub struct UsbfsInEpRegs {
    pub diepctl: VolatileCell<u32>,             // +0x00 IN EP Control
    pub _reserved0: u32,                       // +0x04
    pub diepintf: VolatileCell<u32>,            // +0x08 IN EP Interrupt Flag
    pub _reserved1: u32,                       // +0x0C
    pub dieplen: VolatileCell<u32>,             // +0x10 IN EP Transfer Length
    pub _reserved2: u32,                       // +0x14
    pub dieptfstat: VolatileCell<u32>,          // +0x18 IN EP TX FIFO Space Remaining (words)
    pub _reserved3: u32,                       // +0x1C
}

#[repr(C)]
pub struct UsbfsOutEpRegs {
    pub doepctl: VolatileCell<u32>,             // +0x00 OUT EP Control
    pub _reserved0: u32,                       // +0x04
    pub doepintf: VolatileCell<u32>,            // +0x08 OUT EP Interrupt Flag
    pub _reserved1: u32,                       // +0x0C
    pub doeplen: VolatileCell<u32>,             // +0x10 OUT EP Transfer Length
    pub _reserved2: u32,                       // +0x14
    pub _reserved3: [u32; 2],                  // +0x18 - +0x1C
}

// Bit definitions
pub mod gahbcs {
    pub const GINTEN: u32 = 1 << 0;
}

pub mod gusbcs {
    pub const EMBPHY: u32 = 1 << 6;
    pub const UTT_MASK: u32 = 0xF << 10;
    pub const FHM: u32 = 1 << 29;
    pub const FDM: u32 = 1 << 30;
}

pub mod grstctl {
    pub const CSRST: u32 = 1 << 0;
    pub const RXFF: u32 = 1 << 4;
    pub const TXFF: u32 = 1 << 5;
}

pub mod gccfg {
    pub const PWRON: u32 = 1 << 16;
    pub const VBUSACEN: u32 = 1 << 18;
    pub const VBUSBCEN: u32 = 1 << 19;
    pub const SOFOEN: u32 = 1 << 20;
    pub const VBUSIG: u32 = 1 << 21;
}

pub mod gintf {
    pub const COPM: u32 = 1 << 0;
    pub const SOF: u32 = 1 << 3;
    pub const RXFNEIF: u32 = 1 << 4;
    pub const SP: u32 = 1 << 11;
    pub const RST: u32 = 1 << 12;
    pub const ENUMFIF: u32 = 1 << 13;
    pub const IEPIF: u32 = 1 << 18;
    pub const OEPIF: u32 = 1 << 19;
    pub const ISOINCIF: u32 = 1 << 20;
    pub const ISOONCIF: u32 = 1 << 21;
    pub const WKUPIF: u32 = 1 << 31;
}

pub mod ginten {
    pub const SOFIE: u32 = 1 << 3;
    pub const RXFNEIE: u32 = 1 << 4;
    pub const SPIE: u32 = 1 << 11;
    pub const RSTIE: u32 = 1 << 12;
    pub const ENUMFIE: u32 = 1 << 13;
    pub const IEPIE: u32 = 1 << 18;
    pub const OEPIE: u32 = 1 << 19;
    pub const ISOINCIE: u32 = 1 << 20;
    pub const ISOONCIE: u32 = 1 << 21;
    pub const WKUPIE: u32 = 1 << 31;
}

pub mod grstat {
    pub const EPNUM_MASK: u32 = 0xF << 0;
    pub const BCOUNT_MASK: u32 = 0x7FF << 4;
    pub const RPCKST_MASK: u32 = 0xF << 17;

    pub const RSTAT_GOUT_NAK: u32 = 1;
    pub const RSTAT_DATA_UPDT: u32 = 2;
    pub const RSTAT_XFER_COMP: u32 = 3;
    pub const RSTAT_SETUP_COMP: u32 = 4;
    pub const RSTAT_SETUP_UPDT: u32 = 6;
}

pub mod dcfg {
    pub const DS_MASK: u32 = 0x3 << 0;
    pub const DS_FULL: u32 = 0x3 << 0;
    pub const DAR_MASK: u32 = 0x7F << 4;
    pub const EOPFT_MASK: u32 = 0x3 << 11;
    pub const EOPFT_80: u32 = 0x0 << 11;
}

pub mod dctl {
    pub const RWKUP: u32 = 1 << 0;
    pub const SDIS: u32 = 1 << 1;
    pub const CGINAK: u32 = 1 << 8;
}

pub mod dstat {
    pub const SPST: u32 = 1 << 0;
    pub const ES_MASK: u32 = 0x3 << 1;
}

pub mod diepinten {
    pub const TFEN: u32 = 1 << 0;
}

pub mod doepinten {
    pub const TFEN: u32 = 1 << 0;
    pub const STPFEN: u32 = 1 << 3;
}

pub mod depctl {
    pub const MPL_MASK: u32 = 0x7FF << 0;
    pub const EPACT: u32 = 1 << 15;
    pub const NAKS: u32 = 1 << 17;
    pub const EPTYPE_MASK: u32 = 0x3 << 18;
    pub const EPTYPE_CTRL: u32 = 0x0 << 18;
    pub const EPTYPE_ISOC: u32 = 0x1 << 18;
    pub const EPTYPE_BULK: u32 = 0x2 << 18;
    pub const EPTYPE_INTR: u32 = 0x3 << 18;
    pub const STALL: u32 = 1 << 21;
    pub const TXFNUM_MASK: u32 = 0xF << 22;
    pub const CNAK: u32 = 1 << 26;
    pub const SNAK: u32 = 1 << 27;
    pub const SD0PID: u32 = 1 << 28;
    pub const SD1PID: u32 = 1 << 29;
    pub const EPD: u32 = 1 << 30;
    pub const EPEN: u32 = 1 << 31;
}

pub mod diepintf {
    pub const TF: u32 = 1 << 0;
    pub const EPDIS: u32 = 1 << 1;
    pub const TXFUD: u32 = 1 << 4;
    pub const TXFE: u32 = 1 << 7;
}

pub mod doepintf {
    pub const TF: u32 = 1 << 0;
    pub const EPDIS: u32 = 1 << 1;
    pub const STPF: u32 = 1 << 3;
}

pub mod deplen {
    pub const TLEN_MASK: u32 = 0x7_FFFF;
    pub const PCNT_MASK: u32 = 0x3FF << 19;
}

pub struct Usbfs {
    _private: (),
}

unsafe impl Send for Usbfs {}
unsafe impl Sync for Usbfs {}

impl Usbfs {
    #[inline(always)]
    pub unsafe fn steal() -> Self {
        Self { _private: () }
    }

    #[inline(always)]
    pub fn global(&self) -> &UsbfsGlobalRegs {
        unsafe { &*(USBFS_BASE as *const UsbfsGlobalRegs) }
    }

    #[inline(always)]
    pub fn device(&self) -> &UsbfsDeviceRegs {
        unsafe { &*((USBFS_BASE + 0x0800) as *const UsbfsDeviceRegs) }
    }

    #[inline(always)]
    pub fn in_ep(&self, ep: usize) -> &UsbfsInEpRegs {
        debug_assert!(ep < 4);
        unsafe { &*((USBFS_BASE + 0x0900 + ep * 0x20) as *const UsbfsInEpRegs) }
    }

    #[inline(always)]
    pub fn out_ep(&self, ep: usize) -> &UsbfsOutEpRegs {
        debug_assert!(ep < 4);
        unsafe { &*((USBFS_BASE + 0x0B00 + ep * 0x20) as *const UsbfsOutEpRegs) }
    }

    #[inline(always)]
    pub fn pwrclkctl(&self) -> &VolatileCell<u32> {
        unsafe { &*((USBFS_BASE + 0x0E00) as *const VolatileCell<u32>) }
    }

    #[inline(always)]
    pub fn fifo(&self, ep: usize) -> *mut u32 {
        debug_assert!(ep < 4);
        (USBFS_BASE + 0x1000 + ep * 0x1000) as *mut u32
    }
}
