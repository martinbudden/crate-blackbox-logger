#![cfg(feature = "huffman")]
use crate::huffman_table::HUFFMAN_TABLE;

#[derive(Debug, Default, PartialEq)]
pub struct HuffmanWriter<'a> {
    output: &'a mut [u8],
    byte_idx: usize,
    bit_buffer: u64, // Accumulates bits before flushing to bytes
    bit_count: u8,
}

impl<'a> HuffmanWriter<'a> {
    pub fn new(output: &'a mut [u8]) -> Self {
        Self { output, byte_idx: 0, bit_buffer: 0, bit_count: 0 }
    }

    /// Pushes a predefined code into the stream.
    pub fn write_code(&mut self, code: u32, len: u8) -> Result<(), ()> {
        // Append bits to our 64-bit buffer
        self.bit_buffer |= u64::from(code) << self.bit_count;
        self.bit_count += len;

        // Flush whole bytes out to the array
        while self.bit_count >= 8 {
            if self.byte_idx >= self.output.len() {
                return Err(()); // Output buffer overflowed
            }
            self.output[self.byte_idx] = (self.bit_buffer & 0xFF) as u8;
            self.byte_idx += 1;
            self.bit_buffer >>= 8;
            self.bit_count -= 8;
        }
        Ok(())
    }

    /// Pad any leftover bits with zeros up to the nearest byte boundary.
    pub fn flush(mut self) -> Result<usize, ()> {
        if self.bit_count > 0 {
            if self.byte_idx >= self.output.len() {
                return Err(());
            }
            self.output[self.byte_idx] = (self.bit_buffer & 0xFF) as u8;
            self.byte_idx += 1;
        }
        Ok(self.byte_idx) // Returns total bytes written
    }

    /// O(1) compression routine per byte.
    pub fn compress(mut self, input: &[u8]) -> Result<usize, ()> {
        for &byte in input {
            let huffman_code = HUFFMAN_TABLE[byte as usize];
            #[allow(clippy::cast_possible_truncation)]
            self.write_code(u32::from(huffman_code.code), huffman_code.len as u8)?;
        }
        self.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_partial<T: Sized + Send + Sync + Unpin + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_partial::<HuffmanWriter>();
    }
}
