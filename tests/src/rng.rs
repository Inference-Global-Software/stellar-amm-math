//! SplitMix64: a small deterministic generator, so that every run of the
//! suite draws the same inputs and a failure reproduces from its seed.

pub struct Rng(u64);

impl Rng {
    #[must_use]
    pub fn new(seed: u64) -> Rng {
        Rng(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// A value in `0..bound`.
    ///
    /// # Panics
    ///
    /// When `bound` is 0.
    pub fn below(&mut self, bound: u64) -> u64 {
        assert!(bound > 0, "an empty range");
        self.next_u64() % bound
    }

    /// A value in `low..=high`.
    ///
    /// # Panics
    ///
    /// When `high` is below `low`.
    pub fn range_u32(&mut self, low: u32, high: u32) -> u32 {
        let span = u64::from(high - low) + 1;
        low + u32::try_from(self.below(span)).expect("below a u32 span")
    }

    /// One of `items`.
    ///
    /// # Panics
    ///
    /// When `items` is empty.
    pub fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        let index = usize::try_from(self.below(items.len() as u64)).expect("an index");
        items[index]
    }

    /// A `u32` from a mix that reaches every scale: uniform, log-uniform (a
    /// random bit length), one of `edges` nudged by up to two, or small.
    ///
    /// # Panics
    ///
    /// When `edges` is empty.
    pub fn mixed_u32(&mut self, edges: &[u32]) -> u32 {
        match self.below(4) {
            0 => self.next_u32(),
            1 => {
                let bits = self.below(33);
                if bits == 0 { 0 } else { self.next_u32() >> (32 - bits) }
            }
            2 => {
                let nudged = i64::from(self.pick(edges)) + i64::from(self.range_u32(0, 4)) - 2;
                u32::try_from(nudged.clamp(0, i64::from(u32::MAX))).expect("clamped to u32")
            }
            _ => self.range_u32(0, 99_999),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Rng;

    /// The first outputs of the reference implementation for seed 0.
    #[test]
    fn matches_the_reference_splitmix64() {
        let mut rng = Rng::new(0);
        assert_eq!(rng.next_u64(), 0xe220_a839_7b1d_cdaf);
        assert_eq!(rng.next_u64(), 0x6e78_9e6a_a1b9_65f4);
        assert_eq!(rng.next_u64(), 0x06c4_5d18_8009_454f);
    }
}
