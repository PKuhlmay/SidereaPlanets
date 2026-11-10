"""Writes two 48x32 single-component baseline JPEGs (frame1.jpg, frame2.jpg) into the directory given as
argument. ffmpeg's MJPEG encoder cannot produce grayscale JPEG, so the gray fixture is made here and then
muxed with `ffmpeg -framerate 5 -i frame%d.jpg -c copy mjpg_gray.avi` (see generate.sh).
Standard luminance tables from JPEG Annex K, quantisation halved. No dependencies."""

import math
import struct
import sys

W, H = 48, 32

QUANT = [
    16, 11, 10, 16, 24, 40, 51, 61, 12, 12, 14, 19, 26, 58, 60, 55,
    14, 13, 16, 24, 40, 57, 69, 56, 14, 17, 22, 29, 51, 87, 80, 62,
    18, 22, 37, 56, 68, 109, 103, 77, 24, 35, 55, 64, 81, 104, 113, 92,
    49, 64, 78, 87, 103, 121, 120, 101, 72, 92, 95, 98, 112, 100, 103, 99,
]
QUANT = [max(1, q // 2) for q in QUANT]

ZIGZAG = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20, 13, 6, 7,
    14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59, 52, 45, 38, 31, 39,
    46, 53, 60, 61, 54, 47, 55, 62, 63,
]

DC_COUNTS = [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0]
DC_SYMBOLS = list(range(12))
AC_COUNTS = [0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 0x7D]
AC_SYMBOLS = [
    0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07, 0x22,
    0x71, 0x14, 0x32, 0x81, 0x91, 0xA1, 0x08, 0x23, 0x42, 0xB1, 0xC1, 0x15, 0x52, 0xD1, 0xF0, 0x24, 0x33,
    0x62, 0x72, 0x82, 0x09, 0x0A, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x34,
    0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x53, 0x54, 0x55,
    0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x73, 0x74, 0x75, 0x76,
    0x77, 0x78, 0x79, 0x7A, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x92, 0x93, 0x94, 0x95, 0x96,
    0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5,
    0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3, 0xD4,
    0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF1,
    0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA,
]


def codes(counts, symbols):
    table, code, k = {}, 0, 0
    for length in range(1, 17):
        for _ in range(counts[length - 1]):
            table[symbols[k]] = (code, length)
            code += 1
            k += 1
        code <<= 1
    return table


DC, AC = codes(DC_COUNTS, DC_SYMBOLS), codes(AC_COUNTS, AC_SYMBOLS)


def category(v):
    return abs(v).bit_length()


def pixel(frame, x, y):
    v = 128 + 90 * math.sin((x + 5 * frame) / 6.0) * math.cos(y / 4.0) + 20 * math.sin((x + y) / 3.0)
    return max(0, min(255, int(round(v))))


def dct(block):
    out = [0.0] * 64
    for v in range(8):
        for u in range(8):
            s = 0.0
            for y in range(8):
                for x in range(8):
                    s += (block[y * 8 + x] - 128) * math.cos((2 * x + 1) * u * math.pi / 16) * math.cos(
                        (2 * y + 1) * v * math.pi / 16
                    )
            cu = math.sqrt(0.5) if u == 0 else 1.0
            cv = math.sqrt(0.5) if v == 0 else 1.0
            out[v * 8 + u] = 0.25 * cu * cv * s
    return out


def encode(frame):
    bits = []

    def put(code, length):
        bits.extend((code >> i) & 1 for i in range(length - 1, -1, -1))

    def put_value(v, size):
        if size:
            put(v if v >= 0 else v + (1 << size) - 1, size)

    prev_dc = 0
    for by in range(0, H, 8):
        for bx in range(0, W, 8):
            block = [pixel(frame, bx + x, by + y) for y in range(8) for x in range(8)]
            coef = [int(round(c / q)) for c, q in zip(dct(block), QUANT)]
            zz = [coef[i] for i in ZIGZAG]
            diff = zz[0] - prev_dc
            prev_dc = zz[0]
            size = category(diff)
            put(*DC[size])
            put_value(diff, size)
            run = 0
            for c in zz[1:]:
                if c == 0:
                    run += 1
                    continue
                while run > 15:
                    put(*AC[0xF0])
                    run -= 16
                size = category(c)
                put(*AC[(run << 4) | size])
                put_value(c, size)
                run = 0
            if run:
                put(*AC[0x00])
    while len(bits) % 8:
        bits.append(1)
    data = bytearray()
    for i in range(0, len(bits), 8):
        byte = int("".join(map(str, bits[i : i + 8])), 2)
        data.append(byte)
        if byte == 0xFF:
            data.append(0)

    def seg(marker, body):
        return bytes([0xFF, marker]) + struct.pack(">H", len(body) + 2) + body

    out = b"\xff\xd8"
    out += seg(0xDB, bytes([0]) + bytes(QUANT[i] for i in ZIGZAG))
    out += seg(0xC0, struct.pack(">BHHB", 8, H, W, 1) + bytes([1, 0x11, 0]))
    out += seg(0xC4, bytes([0x00]) + bytes(DC_COUNTS) + bytes(DC_SYMBOLS))
    out += seg(0xC4, bytes([0x10]) + bytes(AC_COUNTS) + bytes(AC_SYMBOLS))
    out += seg(0xDA, bytes([1, 1, 0x00, 0, 63, 0]))
    return out + bytes(data) + b"\xff\xd9"


for n in (1, 2):
    with open(f"{sys.argv[1]}/frame{n}.jpg", "wb") as f:
        f.write(encode(n))
