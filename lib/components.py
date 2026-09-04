components = {
    "debug_uart0": {
        "c_sources": [],
        "cpp_sources": [r"debug_uart0/debug_write.cpp"],
        "asm_sources": [],
        "include_paths": [],
        "enabled": True,
    },
    "system": {
        "c_sources": [],
        "cpp_sources": [r"system/system_gd32vf103.cpp", r"system/init.cpp", r"system/handlers.cpp", r"system/systick.cpp", r"system/syscalls.cpp"],
        "asm_sources": [r"system/entry.S", r"system/start.S"],
        "include_paths": [r"-Isystem"],
        "enabled": True,
    },
    "gd32_lcd":{
        "c_sources": [],
        "cpp_sources": [r"gd32v_lcd/src/lcd.cpp"],
        "asm_sources": [],
        "include_paths": [r"-Igd32v_lcd/include"],
        "enabled": True,
    },
    "fatfs": {
        "c_sources": [r"fatfs/src/ff.c"],
        "cpp_sources": [r"fatfs/src/diskio.cpp"],
        "asm_sources": [],
        "include_paths": [r"-Ifatfs/include"],
        "enabled": True,
    },
}