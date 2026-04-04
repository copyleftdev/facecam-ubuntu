#!/usr/bin/env python3
"""Generate seed corpus files from real Elgato Facecam data."""
import struct
import json
import os

BASE = os.path.dirname(os.path.abspath(__file__))

def write_seed(subdir, name, data):
    path = os.path.join(BASE, "corpus", subdir, name)
    with open(path, "wb") as f:
        f.write(data)

# --- v4l2_parse: QUERYCAP responses ---
def gen_querycap():
    # Real Facecam querycap response
    buf = bytearray(104)
    buf[0:9] = b"uvcvideo\x00"  # driver
    buf[16:42] = b"Elgato Facecam: Elgato F\x00"  # card
    buf[48:68] = b"usb-0000:00:14.0-2\x00"  # bus_info
    struct.pack_into("<I", buf, 80, 0x00060011)  # version 6.0.17
    struct.pack_into("<I", buf, 84, 0x84A00001)  # capabilities
    struct.pack_into("<I", buf, 88, 0x04A00001)  # device_caps
    write_seed("v4l2_parse", "real_facecam", bytes(buf))

    # Empty/zeroed
    write_seed("v4l2_parse", "zeroed", bytes(104))

    # Max values
    buf2 = bytearray(104)
    for i in range(104):
        buf2[i] = 0xFF
    write_seed("v4l2_parse", "all_ff", bytes(buf2))

    # No null terminators in strings
    buf3 = bytearray(104)
    for i in range(80):
        buf3[i] = 0x41  # 'A'
    write_seed("v4l2_parse", "no_nulls", bytes(buf3))

gen_querycap()

# --- v4l2_controls: QUERYCTRL responses ---
def gen_controls():
    def make_ctrl(ctrl_id, ctrl_type, name, min_v, max_v, step, default, flags):
        buf = bytearray(68)
        struct.pack_into("<I", buf, 0, ctrl_id)
        struct.pack_into("<I", buf, 4, ctrl_type)
        name_bytes = name.encode()[:31] + b"\x00"
        buf[8:8+len(name_bytes)] = name_bytes
        struct.pack_into("<i", buf, 40, min_v)
        struct.pack_into("<i", buf, 44, max_v)
        struct.pack_into("<i", buf, 48, step)
        struct.pack_into("<i", buf, 52, default)
        struct.pack_into("<I", buf, 56, flags)
        return bytes(buf)

    write_seed("v4l2_parse", "ctrl_brightness",
        make_ctrl(0x00980900, 1, "Brightness", 0, 255, 1, 128, 0))
    write_seed("v4l2_parse", "ctrl_contrast",
        make_ctrl(0x00980901, 1, "Contrast", 0, 10, 1, 3, 0))
    write_seed("v4l2_parse", "ctrl_menu",
        make_ctrl(0x00980918, 3, "Power Line Frequency", 0, 2, 1, 2, 0))
    write_seed("v4l2_parse", "ctrl_bool",
        make_ctrl(0x0098090c, 2, "White Balance Auto", 0, 1, 1, 1, 0))
    write_seed("v4l2_parse", "ctrl_disabled",
        make_ctrl(0x00980999, 1, "Disabled", 0, 100, 1, 50, 0x0001))
    # Negative ranges
    write_seed("v4l2_parse", "ctrl_negative",
        make_ctrl(0x00980903, 1, "Hue", -180, 180, 1, 0, 0))
    # Huge range menu (should not OOM)
    write_seed("v4l2_parse", "ctrl_huge_menu",
        make_ctrl(0x00990000, 3, "Huge Menu", 0, 999999, 1, 0, 0))

gen_controls()

# --- ipc_parse: DaemonCommand JSON ---
def gen_ipc():
    commands = [
        ("status", json.dumps("Status")),
        ("apply_profile", json.dumps({"ApplyProfile": {"name": "default"}})),
        ("set_control", json.dumps({"SetControl": {"name": "brightness", "value": 128}})),
        ("get_control", json.dumps({"GetControl": {"name": "contrast"}})),
        ("get_all", json.dumps("GetAllControls")),
        ("export_diag", json.dumps("ExportDiagnostics")),
        ("force_reset", json.dumps("ForceReset")),
        ("shutdown", json.dumps("Shutdown")),
        ("empty_obj", "{}"),
        ("empty_arr", "[]"),
        ("null", "null"),
        ("nested", json.dumps({"SetControl": {"name": "a" * 10000, "value": 2**53}})),
        ("unicode", json.dumps({"ApplyProfile": {"name": "\u0000\uffff\ud7ff"}})),
    ]
    for name, data in commands:
        write_seed("ipc_parse", name, data.encode())

gen_ipc()

# --- profile_parse: TOML profiles ---
def gen_profiles():
    seeds = {
        "valid_default": '''
name = "default"
description = "Test profile"

[video_mode]
width = 1920
height = 1080
fps = 30
format = "UYVY"

[controls]
brightness = 128
contrast = 3
''',
        "empty": '',
        "name_only": 'name = "x"\n',
        "huge_dims": '''
name = "huge"
[video_mode]
width = 4294967295
height = 4294967295
fps = 999
format = "XXXX"
''',
        "negative_ctrl": '''
name = "neg"
[controls]
brightness = -9999999
contrast = 9999999
''',
        "deep_nest": 'name = "n"\n' + '[controls]\n' + '\n'.join(
            f'key_{i} = {i}' for i in range(500)
        ),
        "bad_types": '''
name = 12345
description = true
[controls]
brightness = "not_a_number"
''',
    }
    for name, data in seeds.items():
        write_seed("profile_parse", name, data.encode())

gen_profiles()

print("Corpus generated.")
