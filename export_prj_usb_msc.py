#!/usr/bin/env python3
"""
Convenience launcher in repository root for tools/export_prj_usb_msc.py.
"""
import sys
import os

_ROOT = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(_ROOT, "tools"))

from export_prj_usb_msc import main

if __name__ == "__main__":
    main()
