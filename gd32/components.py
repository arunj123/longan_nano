# components.py
# This file defines the source and include paths for the Longan Nano GD32 project,
# organized into separate components for each major unit.
# All paths are now complete and explicit, with no wildcards.

# Note: The paths are relative to the 'gd32' directory.

components = {
    "riscv_drivers": {
        "c_sources": [r"Firmware/RISCV/drivers/n200_func.c"],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-IFirmware/RISCV/drivers"],
    },
    "syscall_stubs": {
        "c_sources": [
            r"Firmware/RISCV/stubs/sbrk.c", r"Firmware/RISCV/stubs/lseek.c",
            r"Firmware/RISCV/stubs/fstat.c", r"Firmware/RISCV/stubs/close.c",
            r"Firmware/RISCV/stubs/_exit.c", r"Firmware/RISCV/stubs/write_hex.c",
            r"Firmware/RISCV/stubs/isatty.c", r"Firmware/RISCV/stubs/read.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-IFirmware/RISCV/stubs"],
    },
    "gd32_std_peripheral_lib": {
        "c_sources": [
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_adc.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_bkp.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_can.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_crc.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_dac.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_dbg.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_dma.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_eclic.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_exmc.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_exti.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_fmc.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_fwdgt.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_gpio.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_i2c.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_pmu.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_rcu.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_rtc.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_spi.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_timer.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_usart.c",
            r"Firmware/GD32VF103_standard_peripheral/Source/gd32vf103_wwdgt.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [
            r"-IFirmware/GD32VF103_standard_peripheral",
            r"-IFirmware/GD32VF103_standard_peripheral/Include"
        ],
    },
    "usb_driver_core": {
        "c_sources": [
            r"Firmware/GD32VF103_usbfs_library/driver/Source/drv_usb_core.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-IFirmware/GD32VF103_usbfs_library/driver/Include"],
    },
    "usb_driver_host": {
        "c_sources": [
            r"Firmware/GD32VF103_usbfs_library/driver/Source/drv_usb_host.c",
            r"Firmware/GD32VF103_usbfs_library/driver/Source/drv_usbh_int.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-IFirmware/GD32VF103_usbfs_library/driver/Include"],
    },
    "usb_driver_device": {
        "c_sources": [
            r"Firmware/GD32VF103_usbfs_library/driver/Source/drv_usb_dev.c",
            r"Firmware/GD32VF103_usbfs_library/driver/Source/drv_usbd_int.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-IFirmware/GD32VF103_usbfs_library/driver/Include"],
    },
    "usb_device_core": {
        "c_sources": [
            r"Firmware/GD32VF103_usbfs_library/device/core/Source/usbd_core.c",
            r"Firmware/GD32VF103_usbfs_library/device/core/Source/usbd_enum.c",
            r"Firmware/GD32VF103_usbfs_library/device/core/Source/usbd_transc.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-IFirmware/GD32VF103_usbfs_library/device/core/Include"],
    },
    "usb_device_class_audio": {
        "c_sources": [
            r"Firmware/GD32VF103_usbfs_library/device/class/audio/Source/audio_core.c",
            r"Firmware/GD32VF103_usbfs_library/device/class/audio/Source/audio_out_itf.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-IFirmware/GD32VF103_usbfs_library/device/class/audio/Include"],
    },
    "usb_device_ustd": {
        "c_sources": [],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [
            r"-IFirmware/GD32VF103_usbfs_library/ustd/common",
            r"-IFirmware/GD32VF103_usbfs_library/ustd/class/cdc",
            r"-IFirmware/GD32VF103_usbfs_library/ustd/class/msc",
            r"-IFirmware/GD32VF103_usbfs_library/ustd/class/hid",],
    },
    "usb_class_cdc": {
        "c_sources": [r"Firmware/GD32VF103_usbfs_library/device/class/cdc/Source/cdc_acm_core.c"],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [
            r"-IFirmware/GD32VF103_usbfs_library/device/class/cdc/Include",
        ],
    },
    "usb_device_class_dfu": {
        "c_sources": [
            r"Firmware/GD32VF103_usbfs_library/device/class/dfu/Source/dfu_core.c",
            r"Firmware/GD32VF103_usbfs_library/device/class/dfu/Source/dfu_mem.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-IFirmware/GD32VF103_usbfs_library/device/class/dfu/Include"],
    },
    "usb_device_class_hid": {
        "c_sources": [
            r"Firmware/GD32VF103_usbfs_library/device/class/hid/Source/custom_hid_core.c",
            r"Firmware/GD32VF103_usbfs_library/device/class/hid/Source/standard_hid_core.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-IFirmware/GD32VF103_usbfs_library/device/class/hid/Include",],
    },
    "usb_device_class_iap": {
        "c_sources": [r"Firmware/GD32VF103_usbfs_library/device/class/iap/Source/usb_iap_core.c"],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-IFirmware/GD32VF103_usbfs_library/device/class/iap/Include"],
    },
    "usb_device_class_msc": {
        "c_sources": [
            r"Firmware/GD32VF103_usbfs_library/device/class/msc/Source/usbd_msc_bbb.c",
            r"Firmware/GD32VF103_usbfs_library/device/class/msc/Source/usbd_msc_core.c",
            r"Firmware/GD32VF103_usbfs_library/device/class/msc/Source/usbd_msc_scsi.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-IFirmware/GD32VF103_usbfs_library/device/class/msc/Include",],
    },
    "usb_device_class_printer": {
        "c_sources": [r"Firmware/GD32VF103_usbfs_library/device/class/printer/Source/printer_core.c"],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-IFirmware/GD32VF103_usbfs_library/device/class/printer/Include"],
    },
    "usb_host_core": {
        "c_sources": [
            r"Firmware/GD32VF103_usbfs_library/host/core/Source/usbh_core.c",
            r"Firmware/GD32VF103_usbfs_library/host/core/Source/usbh_enum.c",
            r"Firmware/GD32VF103_usbfs_library/host/core/Source/usbh_pipe.c",
            r"Firmware/GD32VF103_usbfs_library/host/core/Source/usbh_transc.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-IFirmware/GD32VF103_usbfs_library/host/core/Include"],
    },
    "usb_host_class_hid": {
        "c_sources": [
            r"Firmware/GD32VF103_usbfs_library/host/class/hid/Source/usbh_hid_core.c",
            r"Firmware/GD32VF103_usbfs_library/host/class/hid/Source/usbh_standard_hid.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-IFirmware/GD32VF103_usbfs_library/host/class/hid/Include"],
    },
    "usb_host_class_msc": {
        "c_sources": [
            r"Firmware/GD32VF103_usbfs_library/host/class/msc/Source/usbh_msc_bbb.c",
            r"Firmware/GD32VF103_usbfs_library/host/class/msc/Source/usbh_msc_core.c",
            r"Firmware/GD32VF103_usbfs_library/host/class/msc/Source/usbh_msc_fatfs.c",
            r"Firmware/GD32VF103_usbfs_library/host/class/msc/Source/usbh_msc_scsi.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-IFirmware/GD32VF103_usbfs_library/host/class/msc/Include"],
    },
}
