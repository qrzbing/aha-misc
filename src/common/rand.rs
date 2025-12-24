//! Some Random tools
//!

use libafl_bolts::rands::Rand;

/// Trait RandExt
pub trait RandExt: Rand {
    /// Generate random bytes
    fn generate_bytes(&mut self, len: usize) -> Vec<u8> {
        let mut data = vec![0u8; len];
        self.fill_bytes(&mut data);
        data
    }

    /// Fill bytes
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let mut chunks = dest.chunks_exact_mut(8);
        for chunk in chunks.by_ref() {
            let r = self.next();
            chunk.copy_from_slice(&r.to_le_bytes());
        }
        let remainder = chunks.into_remainder();
        if !remainder.is_empty() {
            let r = self.next();
            remainder.copy_from_slice(&r.to_le_bytes()[..remainder.len()]);
        }
    }
}

/// Implement RandExt for all Rand
impl<T: Rand + ?Sized> RandExt for T {}
