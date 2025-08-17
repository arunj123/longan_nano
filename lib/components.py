components = {
    "sdcard": {
        "c_sources": [],
        "cpp_sources": [r"sdcard/sd_card.cpp", r"sdcard/sd_test.cpp"],
        "asm_sources": [],
        "include_paths": [r"-Isdcard"],
        "enabled": True,
    },
    "debug_uart0": {
        "c_sources": [],
        "cpp_sources": [],
        "asm_sources": [r"debug_uart0/debug_write.cpp"],
        "include_paths": [],
        "enabled": True,
    },
}