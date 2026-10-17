"""Golden PCM captured from hozosch's shipped v19.1 upsampler64.dll.

Release archive SHA256: 576ae599531a2a22c78d6776bce70f33fb7c8139d370e9b350c87497ed4aed3e
Source: f60b6d55da253e4e977e7ea3b0a214ad9bce4a95, upsampler.c.
The sequential reset calls intentionally preserve the original dither state.
"""
import hashlib
import random
import struct
import unittest
from pathlib import Path
from eloquence_contours import Contour

DLL = Path(__file__).resolve().parents[1] / 'target/x86_64-pc-windows-msvc/release/eloquence_contours.dll'


class TrebleTests(unittest.TestCase):
    def test_exact_shipped_c_output_including_reset_and_clipping(self):
        randomizer = random.Random(191)
        signals = [
            [0] * 4096,
            [20000] + [0] * 4095,
            [randomizer.randrange(-32768, 32768) for _ in range(32768)],
            [randomizer.randrange(-80, 81) for _ in range(8192)],
            [32767, -32768] * 4096,
        ]
        hashes = [
            '4fe7b59af6de3b665b67788cc2f99892ab827efae3a467342b3bb4e3bc8e5bfe',
            '033a4d3c9d25e3f0616e6f7435d7b100aec008ebc659b173318a5d4f58a0d22b',
            'dcad9b9d982842bd1d06f3bb634bb41278edba451b96d9a69de889b0a57652f3',
            'ff1adad9775da9c4b6e95931990fefdc85a08eb4eaa2f459ab11b1a981bf10df',
            '77e63bbbad4eb96bfa29b5a101f6e7c02e0c09ab591f8623b0d1769968d49f3d',
        ]
        with Contour(DLL, 11025, 3) as port:
            self.assertEqual(port.output_rate, 22050)
            for signal, expected_hash in zip(signals, hashes):
                port.reset()
                output = b''
                for offset in range(0, len(signal), 137):
                    chunk = signal[offset:offset + 137]
                    output += port.process(struct.pack(f'<{len(chunk)}h', *chunk))
                self.assertEqual(hashlib.sha256(output).hexdigest(), expected_hash)

    def test_independent_handles_and_callback_splits(self):
        pcm = struct.pack('<1024h', *((i * 7919 % 65536) - 32768 for i in range(1024)))
        with Contour(DLL, 11025, 3) as whole, Contour(DLL, 11025, 3) as split:
            self.assertEqual(whole.process(pcm), split.process(pcm[:274]) + split.process(pcm[274:]))
            whole.reset()
            split.reset()
            self.assertEqual(whole.process(pcm), split.process(pcm))
        with self.assertRaises(ValueError):
            Contour(DLL, 16000, 3)


if __name__ == '__main__':
    unittest.main()
