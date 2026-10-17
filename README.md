# Eloquence contours

A standalone, GPL-2.0-only PCM processor. No ECI, NVDA, OpenEVV, host executable,
Python runtime, device access or IPC is linked into the DLL. Applications supply
mono signed little-endian 16-bit PCM and play the returned PCM themselves.
The existing Beta 2 Presence and Beta 4 Smooth implementations were moved
verbatim, including interpolation, gains, clipping and filter history.

## Build and use

Run `build.cmd` from the repository root in a Visual Studio developer environment with Rust
and its i686/x86_64 MSVC targets installed. Outputs:

- `target/i686-pc-windows-msvc/release/eloquence_contours.dll`: 32-bit callers.
- `target/x86_64-pc-windows-msvc/release/eloquence_contours.dll`: 64-bit callers.

Paths above are relative to this directory. Match the DLL to the calling
process, independently of the engine's architecture. A 64-bit NVDA processing
audio from a 32-bit engine uses the **64-bit** contour DLL. Hostless OpenEVV
running in 64-bit NVDA also uses that same 64-bit DLL.

Use `include/eloquence_contours.h` for C/C++ (`cdecl`, ABI version 1), or copy
`python/eloquence_contours.py` alongside a Python consumer. No pip dependencies.
Use an absolute path to load the DLL. The Rust host uses this crate directly
through its path dependency, so its normal build needs no extra runtime DLL.

| Input Hz | Mode | Output Hz | Meaning |
| --- | --- | --- | --- |
| 11025 | 0 | 11025 | Raw, exact PCM passthrough |
| 11025 | 1 or 2 | 22050 | Existing Classic Presence |
| 16000 | 0 | 16000 | Raw, exact PCM passthrough |
| 16000 | 1 | 16000 | Existing native Smooth |
| 16000 | 2 | 16000 | Existing native Presence |

Other rates and modes are rejected. OpenEVV advertises additional rates;
select 11025 or 16000 for these established contours. At other rates the
consumer must offer Raw passthrough outside the module or disable the contour
choice. Do not reinterpret 44100-Hz PCM as 16000-Hz PCM. Supporting new contour
rates requires a separate sound-design decision; this extraction changes no EQ.

```python
from eloquence_contours import Contour

# Same code for PCM from in-process OpenEVV and a legacy engine.
with Contour(r"C:\my-addon\eloquence_contours.dll", 16000, 2) as contour:
    playback_rate = contour.output_rate
    processed_pcm = contour.process(engine_pcm)
    # Feed processed_pcm to a mono PCM16 player opened at playback_rate.
```

Keep one handle for a continuous stream. Do not recreate/reset it at every
callback or speech index: that breaks filter history and Say All continuity.
Reset after cancellation before accepting the next generation of audio.
Recreate on input-rate or mode changes. Serialize calls per handle, destroy
exactly once, and keep the DLL loaded until all its handles are closed.
The Python adapter provides locking and explicit/context-manager cleanup.

## License and origin

Licensed under GPL-2.0-only; see [LICENSE](LICENSE). The DSP was extracted from
[Nick6489/Eloquence64RS](https://github.com/Nick6489/Eloquence64RS), based on
add-on revision `3b5b66f5af498395273aa8319c86dbc05caf8195` and its local contour
extraction. The original Beta 2 and Beta 4 DSP source is preserved unchanged.
This repository contains no proprietary Eloquence engine or voice data.

## Integration with fastfinge/eloquence_64 and OpenEVV

The reviewed `master` uses `DirectEngineClient` for in-process OpenEVV and
`EloquenceHostClient` for the legacy engine. Both feed `AudioPipeline` and its
`AudioWorker` in `addon/synthDrivers/_eloquence.py`:
[shared pipeline source](https://github.com/fastfinge/eloquence_64/blob/master/addon/synthDrivers/_eloquence.py).
Integrate **once in that playback worker**, after stale-generation rejection
and before feeding PCM to WavePlayer. Do not put the DLL solely in the legacy
host, and do not process the same audio twice.

The integration must preserve the following ordering:

1. Treat incoming `RateChange.rate` as the **engine input rate**. Flush the
   pending outgoing chunk, recreate the contour at that input rate, and use
   `contour.output_rate` when calling `_switch_rate`/opening WavePlayer.
   Keep input-rate tracking separate from the player's output-rate tracking.
2. After discarding stale chunks, reset the contour when the accepted chunk's
   generation changes. Process its nonempty `data` once and retain its original
   `(index, is_final, seq)`. Never turn an empty index/final event into PCM.
3. Keep the existing pending-chunk and `onDone` handling. Speech indexes still
   attach to the preceding real audio and fire after playback; resampling
   changes sample counts, not the associated speech index.
4. Queue contour-setting changes alongside rate changes so outgoing queued
   PCM finishes with its original contour/rate. Preserve history between
   fragments, including fragments produced by different compatible backends.
5. Close the contour after the worker has exited. For unsupported OpenEVV
   rates, use Raw bypass or select a supported engine rate before enabling EQ.

This is the integration contract for the external add-on, not a claim that its
remote repository has been modified or tested inside NVDA. The originating
Rust add-on consumes the shared processor through its Rust dependency. This
standalone repository can be built and tested without the add-on checkout.

## Validation

Run `cargo test`.
After building, run `python python/test_contours.py` with system
64-bit Python. Tests cover exact legacy DSP output, callback splitting, reset,
Raw passthrough, rates, C buffer sizing and error paths, and the real 64-bit DLL
through ctypes. Tests do not substitute for listening and cancellation/Say All
checks inside NVDA with the external add-on.
