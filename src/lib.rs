//! Mono signed 16-bit PCM processing, independent of ECI, NVDA and IPC.
pub mod audio;
use audio::{AudioProcessor, PresenceContour};

pub struct Contour {
    processor: AudioProcessor,
    rate: u32,
    selection: u32,
}

fn selection(value: u32) -> Option<PresenceContour> {
    match value {
        0 => Some(PresenceContour::Disabled),
        1 => Some(PresenceContour::Enabled),
        2 => Some(PresenceContour::Presence),
        _ => None,
    }
}

#[no_mangle]
pub extern "C" fn ec_contour_abi_version() -> u32 {
    1
}

#[no_mangle]
pub extern "C" fn ec_contour_create(rate: u32, mode: u32) -> *mut Contour {
    let Some(mode_value) = selection(mode) else {
        return std::ptr::null_mut();
    };
    if !matches!(rate, 11_025 | 16_000) {
        return std::ptr::null_mut();
    }
    let mut processor = AudioProcessor::new(rate);
    processor.set_contour(mode_value);
    Box::into_raw(Box::new(Contour {
        processor,
        rate,
        selection: mode,
    }))
}

/// All handle calls require a live handle and external serialization.
/// Destroy exactly once; NULL is allowed.
#[no_mangle]
pub unsafe extern "C" fn ec_contour_destroy(handle: *mut Contour) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

#[no_mangle]
pub unsafe extern "C" fn ec_contour_reset(handle: *mut Contour) -> i32 {
    let Some(handle) = handle.as_mut() else {
        return -1;
    };
    handle.processor.reset();
    0
}

#[no_mangle]
pub unsafe extern "C" fn ec_contour_output_rate(handle: *const Contour) -> u32 {
    let Some(handle) = handle.as_ref() else {
        return 0;
    };
    if handle.rate == 11_025 && handle.selection != 0 {
        22_050
    } else {
        handle.rate
    }
}

/// Returns required output samples, or SIZE_MAX on invalid count/handle.
#[no_mangle]
pub unsafe extern "C" fn ec_contour_output_count(handle: *const Contour, count: usize) -> usize {
    let Some(handle) = handle.as_ref() else {
        return usize::MAX;
    };
    let factor = if handle.rate == 11_025 && handle.selection != 0 {
        2
    } else {
        1
    };
    count
        .checked_mul(factor)
        .filter(|n| *n <= isize::MAX as usize / 2)
        .unwrap_or(usize::MAX)
}

/// 0 success, -1 invalid argument, -2 insufficient capacity (no state change).
/// Input/output regions must be valid, aligned and non-overlapping.
#[no_mangle]
pub unsafe extern "C" fn ec_contour_process(
    handle: *mut Contour,
    input: *const i16,
    count: usize,
    output: *mut i16,
    capacity: usize,
) -> i32 {
    let required = ec_contour_output_count(handle, count);
    if required == usize::MAX {
        return -1;
    }
    if capacity < required {
        return -2;
    }
    if count == 0 {
        return 0;
    }
    if input.is_null() || output.is_null() {
        return -1;
    }
    let handle = &mut *handle;
    let samples = std::slice::from_raw_parts(input, count);
    let processed = handle.processor.process(samples);
    std::ptr::copy_nonoverlapping(processed.as_ptr(), output, required);
    0
}
