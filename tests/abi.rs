use eloquence_contours::audio::{AudioProcessor, PresenceContour};
use eloquence_contours::*;

#[test]
fn abi_matches_streaming_processor_and_reset() {
    let input: Vec<i16> = (0..4096).map(|i| (i * 7919) as i16).collect();
    for rate in [11025, 16000] {
        for (mode, selection) in [
            (0, PresenceContour::Disabled),
            (1, PresenceContour::Enabled),
            (2, PresenceContour::Presence),
            (3, PresenceContour::Treble),
        ] {
            if rate == 16000 && mode == 3 {
                continue;
            }
            unsafe {
                let handle = ec_contour_create(rate, mode);
                assert!(!handle.is_null());
                let mut expected = AudioProcessor::new(rate);
                expected.set_contour(selection);
                assert_eq!(
                    ec_contour_output_rate(handle),
                    if rate == 11025 && mode != 0 {
                        22050
                    } else {
                        rate
                    }
                );
                for chunk in input.chunks(137) {
                    let count = ec_contour_output_count(handle, chunk.len());
                    let mut out = vec![0; count];
                    assert_eq!(
                        ec_contour_process(
                            handle,
                            chunk.as_ptr(),
                            chunk.len(),
                            out.as_mut_ptr(),
                            count - 1
                        ),
                        -2
                    );
                    assert_eq!(
                        ec_contour_process(
                            handle,
                            chunk.as_ptr(),
                            chunk.len(),
                            out.as_mut_ptr(),
                            count
                        ),
                        0
                    );
                    assert_eq!(out, expected.process(chunk));
                }
                assert_eq!(ec_contour_reset(handle), 0);
                expected.reset();
                let mut out = vec![0; ec_contour_output_count(handle, input.len())];
                assert_eq!(
                    ec_contour_process(
                        handle,
                        input.as_ptr(),
                        input.len(),
                        out.as_mut_ptr(),
                        out.len()
                    ),
                    0
                );
                assert_eq!(out, expected.process(&input));
                ec_contour_destroy(handle);
            }
        }
    }
}

#[test]
fn invalid_arguments_and_empty_input() {
    unsafe {
        assert!(ec_contour_create(48000, 0).is_null());
        assert!(ec_contour_create(16000, 3).is_null());
        assert!(ec_contour_create(11025, 4).is_null());
        assert_eq!(ec_contour_output_count(std::ptr::null(), 0), usize::MAX);
        assert_eq!(ec_contour_reset(std::ptr::null_mut()), -1);
        let handle = ec_contour_create(11025, 1);
        assert_eq!(ec_contour_output_count(handle, usize::MAX), usize::MAX);
        assert_eq!(
            ec_contour_process(handle, std::ptr::null(), 0, std::ptr::null_mut(), 0),
            0
        );
        assert_eq!(
            ec_contour_process(handle, std::ptr::null(), 1, std::ptr::null_mut(), 2),
            -1
        );
        ec_contour_destroy(handle);
        ec_contour_destroy(std::ptr::null_mut());
    }
}
