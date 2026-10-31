"""Write owned synthetic fixtures only. This is asset preparation, not a test."""
import struct
import zlib
from pathlib import Path

root = Path(__file__).parent

def chunk(kind, data):
    return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))

def png(width, height, rows):
    return b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0)) + chunk(b'IDAT', zlib.compress(rows)) + chunk(b'IEND', b'')

# Two RGB scanlines, each with a PNG filter byte, generated from constants.
pixels = b'\0' + bytes([17, 91, 203, 18, 92, 202]) + b'\0' + bytes([19, 93, 201, 20, 94, 200])
(root / 'synthetic.png').write_bytes(png(2, 2, pixels))
# Structurally complete compressed RGB image: 6200 * 6200 * 3 = 115320000
# pixel bytes, above the admitted 100 MiB decoder/live allocation bound.
# Stream its zero rows while preparing the asset, avoiding a huge author buffer.
compressor = zlib.compressobj()
row = b'\0' * (1 + 6200 * 3)
compressed = b''.join(compressor.compress(row) for _ in range(6200)) + compressor.flush()
bounded = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', 6200, 6200, 8, 2, 0, 0, 0)) + chunk(b'IDAT', compressed) + chunk(b'IEND', b'')
(root / 'oversized-dimensions.png').write_bytes(bounded)
