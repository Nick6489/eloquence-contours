# Changes

## 1.1.0

- Add mode 3, Presence treble, for 11025 Hz input and 22050 Hz output.
- Port hozosch/eloquence_64 v19.1 `upsampler.c` without substituting an EQ fit.
- Preserve the original reset behavior, including continuing the dither sequence.
- Verify output against the original release DLL and include response measurements.
- Preserve ABI version 1 and all existing contour modes and coefficients.

## 1.0.0

- Extract existing Raw, Presence and Smooth processing behind a standalone C ABI,
  Rust API and Python adapter.
