components = {
    "sdcard": {
        "c_sources": [],
        "cpp_sources": [r"sdcard/sd_card.cpp", r"sdcard/sd_test.cpp", r"sdcard/sd_spi_hal.cpp"],
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
    "system": {
        "c_sources": [r"system/system_gd32vf103.c", r"system/init.c",
                       r"system/handlers.c", r"system/systick.c",],
        "cpp_sources": [],
        "asm_sources": [r"system/entry.S", r"system/start.S"],
        "include_paths": [r"-Isystem"],
        "enabled": True,
    },
    "gd32_lcd":{
        "c_sources": [r"gd32v_lcd/src/lcd.c"],
        "cpp_sources": [],
        "asm_sources": [],
        "include_paths": [r"-Igd32v_lcd/include"],
        "enabled": True,
    },
}