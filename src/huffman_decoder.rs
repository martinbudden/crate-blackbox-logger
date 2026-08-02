#![cfg(feature = "huffman")]
#![allow(unused)]

use crate::huffman_table::{HUFFMAN_MAX_ENCODED_BITS, HUFFMAN_TABLE, HUFFMAN_TABLE_SIZE};

pub const LUT_SIZE: usize = 1 << HUFFMAN_MAX_ENCODED_BITS; // 4096 entries

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct DecoderEntry {
    pub symbol: u16, // 0..255 for bytes, 256 for EOF
    pub len: u8,     // The exact bit length of this code
}

#[allow(clippy::large_const_arrays)]
pub const DECODER_LUT: [DecoderEntry; LUT_SIZE] = {
    let mut lut = [DecoderEntry { symbol: 0, len: 0 }; LUT_SIZE];

    let mut ii = 0;
    while ii < HUFFMAN_TABLE_SIZE {
        let huff = HUFFMAN_TABLE[ii];
        let len = huff.len as usize;

        if len > 0 {
            // Your HUFFMAN_TABLE code is left-aligned in a u16.
            // Move it to the highest bits of a HUFFMAN_MAX_ENCODED_BITS-bit index.
            let base_idx = (huff.code >> (16 - HUFFMAN_MAX_ENCODED_BITS)) as usize;

            // How many trailing bit combinations exist for this prefix?
            // E.g., if len is 2, there are HUFFMAN_MAX_ENCODED_BITS - 2 = 10 variable trailing bits.
            let num_entries = 1 << (HUFFMAN_MAX_ENCODED_BITS - len);

            let mut jj = 0;
            #[allow(clippy::cast_possible_truncation)]
            while jj < num_entries {
                lut[base_idx + jj] = DecoderEntry { symbol: ii as u16, len: huff.len as u8 };
                jj += 1;
            }
        }
        ii += 1;
    }
    lut
};

pub struct HuffmanDecoder<'a> {
    input: &'a [u8],
    read_idx: usize,
    bit_buffer: u64, // Using u64 makes it easy to keep at least HUFFMAN_MAX_ENCODED_BITS bits ready
    bit_count: u32,
}

impl<'a> HuffmanDecoder<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self { input, read_idx: 0, bit_buffer: 0, bit_count: 0 }
    }

    /// Pulls data from the input slice until we have at least `HUFFMAN_MAX_ENCODED_BITS` bits buffered.
    #[inline]
    fn refill_buffer(&mut self) {
        while self.bit_count <= 56 && self.read_idx < self.input.len() {
            // Read a byte and shift it into the lower part of our bit buffer
            self.bit_buffer |= u64::from(self.input[self.read_idx]) << (56 - self.bit_count);
            self.read_idx += 1;
            self.bit_count += 8;
        }
    }

    /// Decodes a stream that contains an embedded prefix length byte.
    pub fn decompress(&mut self, output: &mut [u8]) -> Result<usize, ()> {
        // Read the uncompressed length byte from the front of the input stream
        if self.input.is_empty() {
            return Err(());
        }
        let uncompressed_len = self.input[0] as usize;

        // Advance our read pointer past the length byte
        self.read_idx = 1;

        if uncompressed_len > output.len() {
            return Err(()); // Target buffer is too small
        }

        let mut write_idx = 0;

        // Loop strictly until we recover the promised number of bytes
        while write_idx < uncompressed_len {
            self.refill_buffer();

            if self.bit_count == 0 {
                return Err(()); // Stream ended early
            }

            // Peek the top HUFFMAN_MAX_ENCODED_BITS bits from the buffer to form our LUT index
            let lut_idx = (self.bit_buffer >> (64 - HUFFMAN_MAX_ENCODED_BITS)) as usize;
            let entry = DECODER_LUT[lut_idx];

            // If length is 0, the bit sequence is invalid/corrupt
            // or if we matched a symbol but don't actually have that many bits left, error out
            if entry.len == 0 || self.bit_count < u32::from(entry.len) {
                return Err(());
            }

            // Consume the matched bits from the buffer
            self.bit_buffer <<= entry.len;
            self.bit_count -= u32::from(entry.len);

            #[allow(clippy::cast_possible_truncation)]
            {
                output[write_idx] = entry.symbol as u8;
            }
            write_idx += 1;
        }

        Ok(write_idx)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    use crate::huffman_encoder::HuffmanEncoder;

    use super::*;

    #[test]
    fn test_round_trip_lookup_table() {
        let original_input = [0u8, 1u8, 2u8, 3u8, 4u8];
        let mut compressed_buffer = [0u8; 32];

        // 1. Compress
        let Ok(writer) = HuffmanEncoder::<16>::new(&mut compressed_buffer) else {
            panic!("Could not create HuffmanEncoder");
        };
        let Ok(compressed_size) = writer.try_compress(&original_input) else {
            panic!("Compression failed unexpectedly with Err(())");
        };

        // 2. Decompress using the LUT
        let mut decompressed_buffer = [0u8; 16];
        let mut reader = HuffmanDecoder::new(&compressed_buffer[..compressed_size]);
        let Ok(decompressed_size) = reader.decompress(&mut decompressed_buffer) else {
            panic!("Decompression failed unexpectedly with Err(())");
        };

        // 3. Match results
        assert_eq!(decompressed_size, original_input.len());
        assert_eq!(&decompressed_buffer[..decompressed_size], &original_input);
    }
}
