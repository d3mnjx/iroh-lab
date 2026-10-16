"""Generate simple code-drawn PWA PNG icons without imaging dependencies."""
import math
import struct
import zlib
from pathlib import Path


def png(size, path):
    rows = []
    for y in range(size):
        row = bytearray()
        for x in range(size):
            # Keep all foreground marks inside the maskable safe circle.
            px, py = (x + 0.5) * 128 / size, (y + 0.5) * 128 / size
            color = (19, 47, 50)
            ax, ay, bx, by = 41, 79, 87, 49
            t = max(0, min(1, ((px-ax)*(bx-ax)+(py-ay)*(by-ay))/((bx-ax)**2+(by-ay)**2)))
            if math.hypot(px-(ax+t*(bx-ax)), py-(ay+t*(by-ay))) < 3.5:
                color = (193, 220, 139)
            if math.hypot(px-ax, py-ay) < 13:
                color = (244, 244, 238)
            if math.hypot(px-bx, py-by) < 13:
                color = (193, 220, 139)
            row.extend(color)
        rows.append(b'\0' + row)
    def chunk(kind, body):
        return struct.pack('>I', len(body)) + kind + body + struct.pack('>I', zlib.crc32(kind+body))
    data = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', size, size, 8, 2, 0, 0, 0)) + chunk(b'IDAT', zlib.compress(b''.join(rows), 9)) + chunk(b'IEND', b'')
    path.write_bytes(data)

root = Path(__file__).resolve().parents[1] / 'web' / 'icons'
for size in (192, 512):
    png(size, root / f'icon-{size}.png')
png(512, root / 'maskable-512.png')
