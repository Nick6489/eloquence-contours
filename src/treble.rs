//! Literal port of hozosch/eloquence_64 v19.1 upsampler.c.
//! Source revision f60b6d55da253e4e977e7ea3b0a214ad9bce4a95.
//! Preserve the original operation order, gain, limiting and dither sequence.

#[derive(Debug)]
pub struct Treble {
    hist: [f32; 8],
    hist_idx: usize,
    slew: f32,
    svf: [f32; 2],
    notch1: [f32; 2],
    notch2: [f32; 2],
    rng: u32,
}

#[cfg(test)]
mod tests {
    use super::Treble;

    #[test]
    fn matches_shipped_v19_1_dll_before_and_after_reset() {
        // FNV-1a of little-endian PCM from the original GCC release DLL.
        let input: Vec<i16> = (0..8192)
            .map(|i| ((i * 7919 % 65536) - 32768) as i16)
            .collect();
        let mut port = Treble::default();
        for expected in [0xed720d6b98b8393f_u64, 0x5997ed477f5f2b41_u64] {
            port.reset();
            let mut hash = 14695981039346656037_u64;
            for chunk in input.chunks(137) {
                for sample in port.process(chunk) {
                    for byte in sample.to_le_bytes() {
                        hash = (hash ^ u64::from(byte)).wrapping_mul(1099511628211);
                    }
                }
            }
            assert_eq!(hash, expected);
        }
    }
}

impl Default for Treble {
    fn default() -> Self {
        Self {
            hist: [0.0; 8],
            hist_idx: 0,
            slew: 0.0,
            svf: [0.0; 2],
            notch1: [0.0; 2],
            notch2: [0.0; 2],
            rng: 0x12345678,
        }
    }
}

impl Treble {
    pub fn reset(&mut self) {
        // Upstream reset clears filters but deliberately leaves rng untouched.
        let rng = self.rng;
        *self = Self::default();
        self.rng = rng;
    }

    pub fn process(&mut self, input: &[i16]) -> Vec<i16> {
        let pi = std::f32::consts::PI;
        let fs = 22050.0_f32;
        let two_pi = 2.0 * pi;
        let g = (pi * 6600.0 / fs).tan();
        let a1 = 1.0 / (1.0 + g * (g + 1.0 / 3.0));
        let a2 = g * a1;
        let c1 = (two_pi * 7500.0 / fs).cos();
        let k1 = 1.0 / (1.0 + (two_pi * 7500.0 / fs).sin());
        let b11 = -2.0 * c1 * k1;
        let a21 = (1.0 - (two_pi * 7500.0 / fs).sin()) * k1;
        let c2 = (two_pi * 9000.0 / fs).cos();
        let k2 = 1.0 / (1.0 + (two_pi * 9000.0 / fs).sin());
        let b12 = -2.0 * c2 * k2;
        let a22 = (1.0 - (two_pi * 9000.0 / fs).sin()) * k2;
        let mut output = Vec::with_capacity(input.len() * 2);
        for &sample in input {
            self.hist[self.hist_idx] = f32::from(sample);
            let h = |offset: usize| self.hist[(self.hist_idx.wrapping_sub(offset)) & 7];
            let s0 = h(4);
            let s1 = h(7) * -0.015
                + h(6) * 0.055
                + h(5) * -0.155
                + h(4) * 0.615
                + h(3) * 0.615
                + h(2) * -0.155
                + h(1) * 0.055
                + h(0) * -0.015;
            self.hist_idx = (self.hist_idx + 1) & 7;
            for mut x in [s0, s1] {
                let diff = x - self.slew;
                let thr = 150.0 * (0.2 + 0.8 * (x.abs() / 32768.0));
                let mut ratio = diff.abs() / (thr + 1e-6);
                if ratio > 1.0 {
                    ratio = 1.0;
                }
                x = self.slew + diff * (0.4 + 0.6 * ratio);
                self.slew = x;
                let v1 = a1 * self.svf[0] + a2 * (x - self.svf[1]);
                let v2 = self.svf[1] + g * v1;
                self.svf[0] = 2.0 * v1 - self.svf[0];
                self.svf[1] = 2.0 * v2 - self.svf[1];
                let y = x + v1 * 3.5;
                let y1 = k1 * y + self.notch1[0];
                self.notch1[0] = b11 * y - b11 * y1 + self.notch1[1];
                self.notch1[1] = k1 * y - a21 * y1;
                let y2 = k2 * y1 + self.notch2[0];
                self.notch2[0] = b12 * y1 - b12 * y2 + self.notch2[1];
                self.notch2[1] = k2 * y1 - a22 * y2;
                self.rng = self.rng.wrapping_mul(1664525).wrapping_add(1013904223);
                let value = ((y2 + (self.rng as f32 * (1.0 / 4294967296.0)) * 0.5) * 0.80)
                    .clamp(-32760.0, 32760.0);
                output.push((value + if value > 0.0 { 0.5 } else { -0.5 }) as i16);
            }
        }
        output
    }
}
