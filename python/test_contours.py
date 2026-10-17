"""Real-DLL smoke tests. Run with system 64-bit Python after build.cmd."""
import ctypes
import struct
import unittest
from pathlib import Path

from eloquence_contours import Contour

DLL = Path(__file__).resolve().parents[1] / "target/x86_64-pc-windows-msvc/release/eloquence_contours.dll"


class DllTests(unittest.TestCase):
    def test_pcm_streams(self):
        pcm = struct.pack("<256h", *(i * 251 - 32000 for i in range(256)))
        for rate in (11025, 16000):
            for mode in (0, 1, 2):
                with Contour(DLL, rate, mode) as contour:
                    output = contour.process(pcm)
                    self.assertEqual(contour.output_rate, 22050 if rate == 11025 and mode else rate)
                    self.assertEqual(len(output), len(pcm) * (2 if rate == 11025 and mode else 1))
                    self.assertEqual(contour.process(b""), b"")
                    contour.reset()
                    split = contour.process(pcm[:126]) + contour.process(pcm[126:])
                    self.assertEqual(output, split)
                    contour.reset()
                    self.assertEqual(output, contour.process(pcm))
                    if mode == 0:
                        self.assertEqual(output, pcm)
                    with self.assertRaises(ValueError):
                        contour.process(b"x")
                contour.close()
                with self.assertRaises(RuntimeError):
                    contour.process(pcm)

    def test_architecture_and_invalid_configuration(self):
        self.assertEqual(ctypes.sizeof(ctypes.c_void_p), 8)
        with self.assertRaises(ValueError):
            Contour(DLL, 44100, 1)


if __name__ == "__main__":
    unittest.main()
