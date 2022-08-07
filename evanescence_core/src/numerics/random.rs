use std::mem;

/// RNG based on the wyrand algorithm; see
/// <https://github.com/lemire/testingRNG/blob/bfd776ba13b837bc1680de08e5de389a7f44f10d/source/wyrand.h>.
pub struct WyRand(u64);

impl WyRand {
    pub fn new_seed(seed: u64) -> Self {
        Self(seed)
    }

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut buf = [0_u8; 8];
        getrandom::getrandom(&mut buf).expect("failed to initialize from system entropy");
        Self(u64::from_ne_bytes(buf))
    }
}

impl WyRand {
    #[allow(clippy::cast_lossless, clippy::cast_possible_truncation)]
    #[inline]
    pub fn gen_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x_a076_1d64_78bd_642f);
        let t = (self.0 as u128).wrapping_mul((self.0 ^ 0x_e703_7ed1_a0b4_28db) as u128);
        (t.wrapping_shr(64) ^ t) as u64
    }

    /// Convert the bits of `x`, a uniformly random `u32` into those of a uniformly random `f32`
    /// between `[1, 2)`.
    /// See <http://www.math.sci.hiroshima-u.ac.jp/m-mat/MT/ARTICLES/dSFMT.pdf>.
    #[inline]
    fn random_f32_bits_from_u32(x: &mut u32) {
        *x >>= 9;
        *x |= 0x_3f80_0000;
    }

    #[inline]
    pub fn gen_f32x2(&mut self) -> [f32; 2] {
        // SAFETY: `u64` -> `[u32; 2]` -> `[f32; 2]` are both safe, as they are pure data types
        // of the same size.
        let mut rand_u32s: [u32; 2] = unsafe { mem::transmute(self.gen_u64()) };
        Self::random_f32_bits_from_u32(&mut rand_u32s[0]);
        Self::random_f32_bits_from_u32(&mut rand_u32s[1]);
        let mut ret: [f32; 2] = unsafe { mem::transmute(rand_u32s) };
        ret[0] -= 1.;
        ret[1] -= 1.;
        ret
    }

    #[inline]
    pub fn gen_f32(&mut self) -> f32 {
        #[allow(clippy::cast_possible_truncation)]
        let mut rand_u32 = self.gen_u64() as u32;
        Self::random_f32_bits_from_u32(&mut rand_u32);
        let ret = f32::from_bits(rand_u32);
        ret - 1.
    }
}

#[cfg(test)]
mod tests {
    use std::iter;

    use itertools::Itertools;

    use super::WyRand;
    use crate::numerics::statistics::kolmogorov_smirnov::test_uniformly_distributed_on;

    #[test]
    fn wyrand_f32_uniformity() {
        let mut p_fails = 0;
        for _ in 0..20 {
            let rng = &mut WyRand::new();
            let mut samples = iter::repeat_with(|| rng.gen_f32x2())
                .take(25_000)
                .flatten()
                .collect_vec();
            samples.extend(iter::repeat_with(|| rng.gen_f32()).take(50_000));
            samples.sort_by(f32::total_cmp);
            assert!(samples[0] >= 0.0 && *samples.last().unwrap() <= 1.0);
            let (ks, p) = test_uniformly_distributed_on(&samples, 0.0..=1.0);
            println!("ks = {ks}, p = {p}");
            assert!(ks < 0.0075);
            if p < 0.05 {
                p_fails += 1;
            }
        }
        println!("p failed count: {p_fails}");
        assert!(p_fails <= 4);
    }
}
