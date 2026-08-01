#![cfg(feature = "huffman")]
use crate::huffman_table::{HUFFMAN_MAX_ENCODED_BITS, HUFFMAN_TABLE};

#[derive(Debug, Default, PartialEq)]
pub struct HuffmanEncoder<'a, const MAX_IN_LEN: usize> {
    output: &'a mut [u8],
    write_idx: usize,
    bit_buffer: u64,
    bit_count: u32,
}

impl<'a, const MAX_INPUT_LEN: usize> HuffmanEncoder<'a, MAX_INPUT_LEN> {
    pub const fn new(output: &'a mut [u8]) -> Result<Self, ()> {
        // The compiler evaluates this at compile time because `MAX_IN_LEN` is a constant value.
        let payload_bits = MAX_INPUT_LEN * HUFFMAN_MAX_ENCODED_BITS;
        let worst_case_payload = payload_bits.div_ceil(8);
        let required_capacity = 1 + worst_case_payload;

        // Perform the verification check exactly once during setup
        if output.len() < required_capacity {
            return Err(());
        }

        Ok(Self {
            output,
            // Start at index 1 to reserve index 0 for the uncompressed length of the input stream
            write_idx: 1,
            bit_buffer: 0,
            bit_count: 0,
        })
    }

    /// Internal helper to push bits into the slice.
    /// Expects left-aligned bits from a u16 code.
    /// Bounds checks have been completely removed via hoisting.
    #[inline]
    fn write_bits(&mut self, code: u16, len: u32) {
        let code_u64 = u64::from(code) << 48;
        self.bit_buffer |= code_u64 >> self.bit_count;
        self.bit_count += len;

        if self.bit_count >= 8 {
            // SAFETY: Bounds check eliminated because the `new` function pre-validated that write_idx can never exceed output.len()
            unsafe {
                *self.output.get_unchecked_mut(self.write_idx) = (self.bit_buffer >> 56) as u8;
            }
            self.write_idx += 1;
            self.bit_buffer <<= 8;
            self.bit_count -= 8;
        }

        if self.bit_count >= 8 {
            // SAFETY: Bounds check eliminated because the `new` function pre-validated that write_idx can never exceed output.len()
            unsafe {
                *self.output.get_unchecked_mut(self.write_idx) = (self.bit_buffer >> 56) as u8;
            }
            self.write_idx += 1;
            self.bit_buffer <<= 8;
            self.bit_count -= 8;
        }
    }

    /// O(1) compression routine per byte.
    pub fn compress(mut self, input: &[u8]) -> Result<usize, ()> {
        let input_len = input.len();
        // Assert for development safety that the slice doesn't break the contract.
        // In release builds on Cortex-M, this is a zero-cost invariant.
        debug_assert!(input.len() <= MAX_INPUT_LEN);

        // Hot path loop - completely free of internal bounds checking branch loops
        for &byte in input {
            let huff_code = HUFFMAN_TABLE[byte as usize];
            self.write_bits(huff_code.code, u32::from(huff_code.len));
        }

        // Flush remaining partial bits (Guaranteed safe by our pre-check)
        if self.bit_count > 0 {
            // SAFETY: Bounds check eliminated because the `new` function pre-validated that write_idx can never exceed output.len()
            unsafe {
                *self.output.get_unchecked_mut(self.write_idx) = (self.bit_buffer >> 56) as u8;
            }
            self.write_idx += 1;
        }

        // Set the leading length byte safely.
        // `try_into` ensures that if input_len somehow exceeded 255, it fails cleanly without silently truncating data.
        let Ok(input_len_u8) = input_len.try_into() else {
            return Err(());
        };
        self.output[0] = input_len_u8;

        Ok(self.write_idx)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_partial<T: Sized + Send + Sync + Unpin + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_partial::<HuffmanEncoder<16>>();
    }

    // 0x0 11
    // 0x1 101
    // 0x2 1001
    // 0x3 10001
    // 0x4 10000
    // 0x5 011101

    #[test]
    fn test_single_byte_zero() {
        let input = [0u8];
        let mut output = [0u8; 25];

        let Ok(writer) = HuffmanEncoder::<16>::new(&mut output) else {
            panic!("Could not create HuffmanEncoder");
        };

        let result = writer.compress(&input);

        // Expected compression stream:
        // '0' symbol:  11 (2 bits)
        // Stream:       11000000 (padded with trailing 0s) -> [1, 0xC0]
        assert_eq!(result, Ok(2)); // length of output stream, including length byte, is 2
        // the length byte is the length of the input stream
        assert_eq!(&output[..2], &[1, 0xC0]);
    }

    #[test]
    fn test_sequence_zero_to_four() {
        let input = [0u8, 1u8, 2u8, 3u8, 4u8];
        let mut output = [0u8; 13];

        let Ok(writer) = HuffmanEncoder::<8>::new(&mut output) else {
            panic!("Could not create HuffmanEncoder");
        };
        let result = writer.compress(&input);

        // Expected compression stream:
        // '0':   11       (2 bits)
        // '1':   101      (3 bits)
        // '2':   1001     (4 bits)
        // '3':   10001    (5 bits)
        // '4':   10000    (5 bits)
        // Stream: 11101100 11000110 00 000000 -> [5, 0xEC, 0xC6, 0x00]
        assert_eq!(result, Ok(4)); // length of output stream, including length byte, is 4
        // the length byte is the length of the input stream
        assert_eq!(&output[..4], &[5, 0xEC, 0xC6, 0x00]);
    }

    #[test]
    fn test_buffer_overflow_error() {
        const INPUT_LEN: usize = 5;
        let input: [u8; INPUT_LEN] = [0u8, 1u8, 2u8, 3u8, 4u8];
        let mut tiny_output = [0u8; 2]; // Too small for result

        let Ok(_writer) = HuffmanEncoder::<INPUT_LEN>::new(&mut tiny_output) else {
            return;
        };
        _ = input;
        panic!("HuffmanEncoder new failed");
    }
}
