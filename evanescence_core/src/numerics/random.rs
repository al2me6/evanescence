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
        // SAFETY: `u64` -> `[u32; 2]` is safe, as they are pure data types of the same size.
        let mut rand_u32 = unsafe { mem::transmute::<u64, [u32; 2]>(self.gen_u64()) }[0];
        Self::random_f32_bits_from_u32(&mut rand_u32);
        let ret = f32::from_bits(rand_u32);
        ret - 1.
    }
}
