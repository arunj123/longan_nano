# components.py
# This file defines the source and include paths for the Longan Nano GD32 project,
# organized into separate components for each major unit.
# All paths are now complete and explicit, with no wildcards.

# Note: The paths are relative to the 'gd32' directory.

components = {
    "riscv_drivers": {
        "c_sources": [],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-IFirmware/RISCV/drivers"],
        "enabled": True,
    },
    "syscall_stubs": {
        "c_sources": [],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [],
        "enabled": True,
    },
    "gd32_std_peripheral_lib": {
        "c_sources": [],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [
            r"-IFirmware/GD32VF103_standard_peripheral",
            r"-IFirmware/GD32VF103_standard_peripheral/Include"
        ],
        "enabled": True,
    },
    "usb_driver_core": {
        "c_sources": [
            r"Firmware/GD32VF103_usbfs_library/driver/Source/drv_usb_core.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-IFirmware/GD32VF103_usbfs_library/driver/Include"],
        "enabled": True,
    },
    "usb_driver_device": {
        "c_sources": [
            r"Firmware/GD32VF103_usbfs_library/driver/Source/drv_usb_dev.c",
            r"Firmware/GD32VF103_usbfs_library/driver/Source/drv_usbd_int.c",
        ],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-IFirmware/GD32VF103_usbfs_library/driver/Include"],
        "enabled": True,
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
        "enabled": True,
    },
    "usb_device_ustd": {
        "c_sources": [],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [
            r"-IFirmware/GD32VF103_usbfs_library/ustd/common",
            r"-IFirmware/GD32VF103_usbfs_library/ustd/class/cdc",
            r"-IFirmware/GD32VF103_usbfs_library/ustd/class/msc",
            r"-IFirmware/GD32VF103_usbfs_library/ustd/class/hid",
        ],
        "enabled": True,
    },
    "usb_class_cdc": {
        "c_sources": [r"Firmware/GD32VF103_usbfs_library/device/class/cdc/Source/cdc_acm_core.c"],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [
            r"-IFirmware/GD32VF103_usbfs_library/device/class/cdc/Include",
        ],
        "enabled": True,
    },
}

