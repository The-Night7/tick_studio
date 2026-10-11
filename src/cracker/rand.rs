#![allow(dead_code)]
const M: u64 = 0x5DEECE66D;
const A: u64 = 0xB;
const INV_A: u64 = 0xdfe05bcb1365;
const MASK: u64 = (1 << 48) - 1;

pub struct JavaRandom {
    pub state: u64,
}

impl JavaRandom {
    pub const fn with_seed(seed: u64) -> Self {
        Self {
            state: (seed ^ M) & MASK,
        }
    }

    pub const fn with_raw_seed(seed: u64) -> Self {
        Self { state: seed & MASK }
    }

    pub const fn set_seed(&mut self, seed: u64) {
        self.state = (seed ^ M) & MASK;
    }

    pub const fn advance(&mut self) {
        self.state = (self.state.wrapping_mul(M).wrapping_add(A)) & MASK;
    }

    pub const fn previous(&mut self) {
        self.state = ((self.state.wrapping_sub(A)).wrapping_mul(INV_A)) & MASK;
    }

    pub fn skip_n(&mut self, n: u64) {
        for _ in 0..n {
            self.advance();
        }
    }

    pub const fn next_bits(&mut self, bits: u32) -> i32 {
        self.advance();
        (self.state >> (48 - bits)) as i32
    }

    pub const fn next_long(&mut self) -> i64 {
        let high = self.next_int() as i64;
        let low = self.next_int() as i64;
        (high << 32) + low
    }

    pub const fn next_int(&mut self) -> i32 {
        self.next_bits(32)
    }

    pub const fn next_int_n(&mut self, n: i32) -> i32 {
        if n <= 0 {
            panic!("n must be positive");
        }
        if n & (n - 1) == 0 {
            return ((n as i64 * self.next_bits(31) as i64) >> 31) as i32;
        }
        let mut bits;
        let mut val;
        loop {
            bits = self.next_bits(31);
            val = bits % n;
            if bits - val + (n - 1) >= 0 {
                break;
            }
        }
        val
    }

    pub const fn next_boolean(&mut self) -> bool {
        self.next_bits(1) != 0
    }

    pub const fn next_float(&mut self) -> f32 {
        self.next_bits(24) as f32 / (1 << 24) as f32
    }

    pub const fn next_double(&mut self) -> f64 {
        let high = self.next_bits(26) as u64;
        let low = self.next_bits(27) as u64;
        ((high << 27 | low) as f64) / (1u64 << 53) as f64
    }
}

impl Default for JavaRandom {
    #[inline]
    fn default() -> Self {
        Self::with_seed(0)
    }
}
