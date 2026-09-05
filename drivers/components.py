components = {
    "drivers": {
        "c_sources": [],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [
            r"-Iinclude",
        ],
        "enabled": True,
    },
    "usb_driver": {
        "c_sources": [],
        "cpp_sources": [
            r"src/usb/drv_usb_core.cpp",
            r"src/usb/drv_usb_dev.cpp",
            r"src/usb/drv_usbd_int.cpp",
            r"src/usb/usbd_core.cpp",
            r"src/usb/usbd_enum.cpp",
            r"src/usb/usbd_transc.cpp",
        ],
        "asm_sources": [],
        "include_paths": [
            r"-Iinclude",
            r"-Iinclude/drivers/usb",
        ],
        "enabled": True,
    },
    "usb_cdc": {
        "c_sources": [],
        "cpp_sources": [
            r"src/usb/cdc_acm.cpp",
        ],
        "asm_sources": [],
        "include_paths": [
            r"-Iinclude",
            r"-Iinclude/drivers/usb",
        ],
        "enabled": True,
    },
}
