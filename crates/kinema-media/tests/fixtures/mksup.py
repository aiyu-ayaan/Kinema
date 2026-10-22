"""Writes a minimal PGS (.sup) file with rendered text cues, for testing OCR only."""
import struct, sys
from PIL import Image, ImageDraw, ImageFont

W, H = 1280, 720
FONT = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 44)
CUES = [(1.0, 3.0, "Hello there friend"), (4.0, 6.5, "Second line of text")]


def seg(pts, kind, data):
    t = int(pts * 90000)
    return b"PG" + struct.pack(">IIBH", t, t, kind, len(data)) + data


def rle(img):
    w, h = img.size
    px = img.load()
    out = bytearray()
    for y in range(h):
        x = 0
        while x < w:
            c = px[x, y]
            n = 1
            while x + n < w and px[x + n, y] == c and n < 16383:
                n += 1
            if c == 0:
                if n < 64:
                    out += bytes([0, n])
                else:
                    out += bytes([0, 0x40 | (n >> 8), n & 0xFF])
            else:
                if n < 3:
                    out += bytes([c]) * n
                elif n < 64:
                    out += bytes([0, 0x80 | n, c])
                else:
                    out += bytes([0, 0xC0 | (n >> 8), n & 0xFF, c])
            x += n
        out += b"\x00\x00"
    return bytes(out)


def main(path):
    buf = bytearray()
    for i, (start, end, text) in enumerate(CUES):
        img = Image.new("P", (900, 80), 0)
        ImageDraw.Draw(img).text((10, 10), text, fill=1, font=FONT)
        ow, oh = img.size
        x, y = (W - ow) // 2, H - oh - 40
        pcs = struct.pack(">HHBHBBBB", W, H, 0x10, i * 2, 0x80, 0, 0, 1) + struct.pack(">HBBHH", 0, 0, 0, x, y)
        wds = struct.pack(">BBHHHH", 1, 0, x, y, ow, oh)
        pds = struct.pack(">BB", 0, 0) + bytes([0, 16, 128, 128, 0]) + bytes([1, 235, 128, 128, 255])
        data = rle(img)
        ods = struct.pack(">HBB", 0, 0, 0xC0) + (len(data) + 4).to_bytes(3, "big") + struct.pack(">HH", ow, oh) + data
        buf += seg(start, 0x16, pcs) + seg(start, 0x17, wds) + seg(start, 0x14, pds) + seg(start, 0x15, ods) + seg(start, 0x80, b"")
        clear = struct.pack(">HHBHBBBB", W, H, 0x10, i * 2 + 1, 0x00, 0, 0, 0)
        buf += seg(end, 0x16, clear) + seg(end, 0x17, wds) + seg(end, 0x80, b"")
    open(path, "wb").write(buf)


main(sys.argv[1])
