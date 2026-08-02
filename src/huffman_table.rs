#![cfg(feature = "huffman")]

pub const HUFFMAN_TABLE_SIZE: usize = 256;
pub const HUFFMAN_MAX_ENCODED_BITS: usize = 13;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct HuffmanCode {
    pub len: u16,
    pub code: u16,
}

pub const HUFFMAN_TABLE: [HuffmanCode; HUFFMAN_TABLE_SIZE] = [
    //                 Len      Code         Char   bit-code
    HuffmanCode { len: 2, code: 0xc000 }, // 0x00 . 0b_11
    HuffmanCode { len: 3, code: 0x6000 }, // 0x01 . 0b_011
    HuffmanCode { len: 3, code: 0x2000 }, // 0x02 . 0b_001
    HuffmanCode { len: 5, code: 0xb800 }, // 0x03 . 0b_10111
    HuffmanCode { len: 4, code: 0x1000 }, // 0x04 . 0b_0001
    HuffmanCode { len: 6, code: 0xb400 }, // 0x05 . 0b_101101
    HuffmanCode { len: 6, code: 0xa800 }, // 0x06 . 0b_101010
    HuffmanCode { len: 6, code: 0x8000 }, // 0x07 . 0b_100000
    HuffmanCode { len: 6, code: 0x9000 }, // 0x08 . 0b_100100
    HuffmanCode { len: 7, code: 0xb200 }, // 0x09 . 0b_1011001
    HuffmanCode { len: 6, code: 0x0000 }, // 0x0a . 0b_000000
    HuffmanCode { len: 7, code: 0x9a00 }, // 0x0b . 0b_1001101
    HuffmanCode { len: 7, code: 0xa400 }, // 0x0c . 0b_1010010
    HuffmanCode { len: 7, code: 0x8400 }, // 0x0d . 0b_1000010
    HuffmanCode { len: 7, code: 0x8600 }, // 0x0e . 0b_1000011
    HuffmanCode { len: 7, code: 0x5000 }, // 0x0f . 0b_0101000
    //
    HuffmanCode { len: 5, code: 0x0800 }, // 0x10 . 0b_00001
    HuffmanCode { len: 7, code: 0x9400 }, // 0x11 . 0b_1001010
    HuffmanCode { len: 7, code: 0x4800 }, // 0x12 . 0b_0100100
    HuffmanCode { len: 8, code: 0xa700 }, // 0x13 . 0b_10100111
    HuffmanCode { len: 7, code: 0x4000 }, // 0x14 . 0b_0100000
    HuffmanCode { len: 8, code: 0x9c00 }, // 0x15 . 0b_10011100
    HuffmanCode { len: 8, code: 0x9700 }, // 0x16 . 0b_10010111
    HuffmanCode { len: 8, code: 0x8a00 }, // 0x17 . 0b_10001010
    HuffmanCode { len: 8, code: 0x8c00 }, // 0x18 . 0b_10001100
    HuffmanCode { len: 8, code: 0x5900 }, // 0x19 . 0b_01011001
    HuffmanCode { len: 8, code: 0x5e00 }, // 0x1a . 0b_01011110
    HuffmanCode { len: 8, code: 0x0500 }, // 0x1b . 0b_00000101
    HuffmanCode { len: 8, code: 0x4700 }, // 0x1c . 0b_01000111
    HuffmanCode { len: 9, code: 0xaf80 }, // 0x1d . 0b_101011111
    HuffmanCode { len: 8, code: 0x5800 }, // 0x1e . 0b_01011000
    HuffmanCode { len: 7, code: 0x4200 }, // 0x1f . 0b_0100001
    //
    HuffmanCode { len: 8, code: 0xb000 },  // 0x20 . 0b_10110000
    HuffmanCode { len: 8, code: 0x4500 },  // 0x21 ! 0b_01000101
    HuffmanCode { len: 9, code: 0xa680 },  // 0x22 " 0b_101001101
    HuffmanCode { len: 9, code: 0xae80 },  // 0x23 # 0b_101011101
    HuffmanCode { len: 9, code: 0x8900 },  // 0x24 $ 0b_100010010
    HuffmanCode { len: 9, code: 0x4b80 },  // 0x25 % 0b_010010111
    HuffmanCode { len: 9, code: 0x5200 },  // 0x26 & 0b_010100100
    HuffmanCode { len: 9, code: 0x0400 },  // 0x27 ' 0b_000001000
    HuffmanCode { len: 9, code: 0x4400 },  // 0x28 ( 0b_010001000
    HuffmanCode { len: 10, code: 0xad00 }, // 0x29 ) 0b_1010110100
    HuffmanCode { len: 10, code: 0xae00 }, // 0x2a * 0b_1010111000
    HuffmanCode { len: 10, code: 0x9ec0 }, // 0x2b + 0b_1001111011
    HuffmanCode { len: 10, code: 0xacc0 }, // 0x2c , 0b_1010110011
    HuffmanCode { len: 10, code: 0x9e80 }, // 0x2d - 0b_1001111010
    HuffmanCode { len: 10, code: 0xaf00 }, // 0x2e . 0b_1010111100
    HuffmanCode { len: 9, code: 0x8880 },  // 0x2f / 0b_100010001
    //
    HuffmanCode { len: 9, code: 0x9f80 },  // 0x30 0 0b_100111111
    HuffmanCode { len: 10, code: 0xb180 }, // 0x31 1 0b_1011000110
    HuffmanCode { len: 10, code: 0xaf40 }, // 0x32 2 0b_1010111101
    HuffmanCode { len: 10, code: 0x9680 }, // 0x33 3 0b_1001011010
    HuffmanCode { len: 10, code: 0x89c0 }, // 0x34 4 0b_1000100111
    HuffmanCode { len: 10, code: 0x8980 }, // 0x35 5 0b_1000100110
    HuffmanCode { len: 10, code: 0x5d40 }, // 0x36 6 0b_0101110101
    HuffmanCode { len: 10, code: 0x5b00 }, // 0x37 7 0b_0101101100
    HuffmanCode { len: 10, code: 0x5b80 }, // 0x38 8 0b_0101101110
    HuffmanCode { len: 10, code: 0x4cc0 }, // 0x39 9 0b_0100110011
    HuffmanCode { len: 10, code: 0x53c0 }, // 0x3a : 0b_0101001111
    HuffmanCode { len: 10, code: 0x4a00 }, // 0x3b ; 0b_0100101000
    HuffmanCode { len: 10, code: 0x5ac0 }, // 0x3c < 0b_0101101011
    HuffmanCode { len: 10, code: 0x4a40 }, // 0x3d = 0b_0100101001
    HuffmanCode { len: 10, code: 0x5a40 }, // 0x3e > 0b_0101101001
    HuffmanCode { len: 10, code: 0x8d00 }, // 0x3f ? 0b_1000110100
    //
    HuffmanCode { len: 6, code: 0xa000 },  // 0x40 @ 0b_101000
    HuffmanCode { len: 8, code: 0x4d00 },  // 0x41 A 0b_01001101
    HuffmanCode { len: 10, code: 0x5280 }, // 0x42 B 0b_0101001010
    HuffmanCode { len: 10, code: 0x06c0 }, // 0x43 C 0b_0000011011
    HuffmanCode { len: 7, code: 0x4e00 },  // 0x44 D 0b_0100111
    HuffmanCode { len: 9, code: 0x8800 },  // 0x45 E 0b_100010000
    HuffmanCode { len: 10, code: 0x0640 }, // 0x46 F 0b_0000011001
    HuffmanCode { len: 11, code: 0xae60 }, // 0x47 G 0b_10101110011
    HuffmanCode { len: 11, code: 0xb1c0 }, // 0x48 H 0b_10110001110
    HuffmanCode { len: 11, code: 0xae40 }, // 0x49 I 0b_10101110010
    HuffmanCode { len: 11, code: 0xb120 }, // 0x4a J 0b_10110001001
    HuffmanCode { len: 11, code: 0xadc0 }, // 0x4b K 0b_10101101110
    HuffmanCode { len: 11, code: 0xac00 }, // 0x4c L 0b_10101100000
    HuffmanCode { len: 11, code: 0xad60 }, // 0x4d M 0b_10101101011
    HuffmanCode { len: 11, code: 0xb100 }, // 0x4e N 0b_10110001000
    HuffmanCode { len: 11, code: 0xad80 }, // 0x4f O 0b_10101101100
    //
    HuffmanCode { len: 10, code: 0x8d40 }, // 0x50 P 0b_1000110101
    HuffmanCode { len: 10, code: 0x9d80 }, // 0x51 Q 0b_1001110110
    HuffmanCode { len: 10, code: 0x0480 }, // 0x52 R 0b_0000010010
    HuffmanCode { len: 11, code: 0xada0 }, // 0x53 S 0b_10101101101
    HuffmanCode { len: 9, code: 0x4c00 },  // 0x54 T 0b_010011000
    HuffmanCode { len: 10, code: 0x5c40 }, // 0x55 U 0b_0101110001
    HuffmanCode { len: 11, code: 0x9de0 }, // 0x56 V 0b_10011101111
    HuffmanCode { len: 11, code: 0x9880 }, // 0x57 W 0b_10011000100
    HuffmanCode { len: 11, code: 0x9e20 }, // 0x58 X 0b_10011110001
    HuffmanCode { len: 11, code: 0x8f60 }, // 0x59 Y 0b_10001111011
    HuffmanCode { len: 11, code: 0x9e60 }, // 0x5a Z 0b_10011110011
    HuffmanCode { len: 11, code: 0x9900 }, // 0x5b [ 0b_10011001000
    HuffmanCode { len: 11, code: 0x98e0 }, // 0x5c \ 0b_10011000111
    HuffmanCode { len: 11, code: 0x9800 }, // 0x5d ] 0b_10011000000
    HuffmanCode { len: 11, code: 0x9660 }, // 0x5e ^ 0b_10010110011
    HuffmanCode { len: 11, code: 0x9820 }, // 0x5f _ 0b_10011000001
    //
    HuffmanCode { len: 11, code: 0x8fa0 }, // 0x60 ` 0b_10001111101
    HuffmanCode { len: 11, code: 0x9e00 }, // 0x61 a 0b_10011110000
    HuffmanCode { len: 11, code: 0x9980 }, // 0x62 b 0b_10011001100
    HuffmanCode { len: 11, code: 0x9940 }, // 0x63 c 0b_10011001010
    HuffmanCode { len: 11, code: 0x99e0 }, // 0x64 d 0b_10011001111
    HuffmanCode { len: 11, code: 0x9960 }, // 0x65 e 0b_10011001011
    HuffmanCode { len: 11, code: 0x8b60 }, // 0x66 f 0b_10001011011
    HuffmanCode { len: 11, code: 0x9640 }, // 0x67 g 0b_10010110010
    HuffmanCode { len: 11, code: 0x8fe0 }, // 0x68 h 0b_10001111111
    HuffmanCode { len: 11, code: 0x8e60 }, // 0x69 i 0b_10001110011
    HuffmanCode { len: 11, code: 0x8ea0 }, // 0x6a j 0b_10001110101
    HuffmanCode { len: 11, code: 0x8be0 }, // 0x6b k 0b_10001011111
    HuffmanCode { len: 11, code: 0x8b80 }, // 0x6c l 0b_10001011100
    HuffmanCode { len: 11, code: 0x8b00 }, // 0x6d m 0b_10001011000
    HuffmanCode { len: 11, code: 0x5f20 }, // 0x6e n 0b_01011111001
    HuffmanCode { len: 11, code: 0x5fc0 }, // 0x6f o 0b_01011111110
    //
    HuffmanCode { len: 11, code: 0x5c00 }, // 0x70 p 0b_01011100000
    HuffmanCode { len: 11, code: 0x5aa0 }, // 0x71 q 0b_01011010101
    HuffmanCode { len: 11, code: 0x8da0 }, // 0x72 r 0b_10001101101
    HuffmanCode { len: 11, code: 0x5d00 }, // 0x73 s 0b_01011101000
    HuffmanCode { len: 11, code: 0x8b40 }, // 0x74 t 0b_10001011010
    HuffmanCode { len: 11, code: 0x5b60 }, // 0x75 u 0b_01011011011
    HuffmanCode { len: 11, code: 0x5da0 }, // 0x76 v 0b_01011101101
    HuffmanCode { len: 11, code: 0x4b60 }, // 0x77 w 0b_01001011011
    HuffmanCode { len: 11, code: 0x5be0 }, // 0x78 x 0b_01011011111
    HuffmanCode { len: 11, code: 0x4ae0 }, // 0x79 y 0b_01001010111
    HuffmanCode { len: 11, code: 0x53a0 }, // 0x7a z 0b_01010011101
    HuffmanCode { len: 11, code: 0x52e0 }, // 0x7b { 0b_01010010111
    HuffmanCode { len: 11, code: 0x4a80 }, // 0x7c | 0b_01001010100
    HuffmanCode { len: 11, code: 0x4c80 }, // 0x7d } 0b_01001100100
    HuffmanCode { len: 11, code: 0x04e0 }, // 0x7e ~ 0b_00000100111
    HuffmanCode { len: 11, code: 0x07e0 }, // 0x7f . 0b_00000111111
    //
    HuffmanCode { len: 11, code: 0x9860 }, // 0x80 . 0b_10011000011
    HuffmanCode { len: 11, code: 0x8f80 }, // 0x81 . 0b_10001111100
    HuffmanCode { len: 11, code: 0x9600 }, // 0x82 . 0b_10010110000
    HuffmanCode { len: 11, code: 0x8e40 }, // 0x83 . 0b_10001110010
    HuffmanCode { len: 11, code: 0x9840 }, // 0x84 . 0b_10011000010
    HuffmanCode { len: 11, code: 0x5dc0 }, // 0x85 . 0b_01011101110
    HuffmanCode { len: 11, code: 0x98a0 }, // 0x86 . 0b_10011000101
    HuffmanCode { len: 11, code: 0x8e00 }, // 0x87 . 0b_10001110000
    HuffmanCode { len: 11, code: 0x8e20 }, // 0x88 . 0b_10001110001
    HuffmanCode { len: 11, code: 0x5fa0 }, // 0x89 . 0b_01011111101
    HuffmanCode { len: 11, code: 0x8bc0 }, // 0x8a . 0b_10001011110
    HuffmanCode { len: 11, code: 0x5f00 }, // 0x8b . 0b_01011111000
    HuffmanCode { len: 11, code: 0x8b20 }, // 0x8c . 0b_10001011001
    HuffmanCode { len: 11, code: 0x5cc0 }, // 0x8d . 0b_01011100110
    HuffmanCode { len: 11, code: 0x96e0 }, // 0x8e . 0b_10010110111
    HuffmanCode { len: 11, code: 0x5c20 }, // 0x8f . 0b_01011100001
    //
    HuffmanCode { len: 10, code: 0x5f40 }, // 0x90 . 0b_0101111101
    HuffmanCode { len: 11, code: 0x4b00 }, // 0x91 . 0b_01001011000
    HuffmanCode { len: 11, code: 0x9620 }, // 0x92 . 0b_10010110001
    HuffmanCode { len: 11, code: 0x5bc0 }, // 0x93 . 0b_01011011110
    HuffmanCode { len: 11, code: 0xac60 }, // 0x94 . 0b_10101100011
    HuffmanCode { len: 11, code: 0x5340 }, // 0x95 . 0b_01010011010
    HuffmanCode { len: 11, code: 0x5c80 }, // 0x96 . 0b_01011100100
    HuffmanCode { len: 11, code: 0x5b40 }, // 0x97 . 0b_01011011010
    HuffmanCode { len: 11, code: 0x5a00 }, // 0x98 . 0b_01011010000
    HuffmanCode { len: 11, code: 0x5380 }, // 0x99 . 0b_01010011100
    HuffmanCode { len: 11, code: 0x5ca0 }, // 0x9a . 0b_01011100101
    HuffmanCode { len: 11, code: 0x4b20 }, // 0x9b . 0b_01001011001
    HuffmanCode { len: 11, code: 0x9dc0 }, // 0x9c . 0b_10011101110
    HuffmanCode { len: 11, code: 0x4ac0 }, // 0x9d . 0b_01001010110
    HuffmanCode { len: 11, code: 0x8ec0 }, // 0x9e . 0b_10001110110
    HuffmanCode { len: 11, code: 0x44a0 }, // 0x9f . 0b_01000100101
    //
    HuffmanCode { len: 10, code: 0x8dc0 }, // 0xa0 . 0b_1000110111
    HuffmanCode { len: 11, code: 0x4aa0 }, // 0xa1 . 0b_01001010101
    HuffmanCode { len: 11, code: 0x99c0 }, // 0xa2 . 0b_10011001110
    HuffmanCode { len: 12, code: 0xb140 }, // 0xa3 . 0b_101100010100
    HuffmanCode { len: 11, code: 0xa640 }, // 0xa4 . 0b_10100110010
    HuffmanCode { len: 12, code: 0xb160 }, // 0xa5 . 0b_101100010110
    HuffmanCode { len: 11, code: 0x5320 }, // 0xa6 . 0b_01010011001
    HuffmanCode { len: 12, code: 0xb1e0 }, // 0xa7 . 0b_101100011110
    HuffmanCode { len: 11, code: 0x52c0 }, // 0xa8 . 0b_01010010110
    HuffmanCode { len: 12, code: 0xb150 }, // 0xa9 . 0b_101100010101
    HuffmanCode { len: 11, code: 0x0780 }, // 0xaa . 0b_00000111100
    HuffmanCode { len: 12, code: 0xade0 }, // 0xab . 0b_101011011110
    HuffmanCode { len: 11, code: 0x06a0 }, // 0xac . 0b_00000110101
    HuffmanCode { len: 12, code: 0xa670 }, // 0xad . 0b_101001100111
    HuffmanCode { len: 12, code: 0xb1f0 }, // 0xae . 0b_101100011111
    HuffmanCode { len: 12, code: 0xa660 }, // 0xaf . 0b_101001100110
    //
    HuffmanCode { len: 11, code: 0x0680 }, // 0xb0 . 0b_00000110100
    HuffmanCode { len: 12, code: 0xac50 }, // 0xb1 . 0b_101011000101
    HuffmanCode { len: 12, code: 0xac40 }, // 0xb2 . 0b_101011000100
    HuffmanCode { len: 12, code: 0xa610 }, // 0xb3 . 0b_101001100001
    HuffmanCode { len: 12, code: 0x9e40 }, // 0xb4 . 0b_100111100100
    HuffmanCode { len: 12, code: 0xa630 }, // 0xb5 . 0b_101001100011
    HuffmanCode { len: 12, code: 0xacb0 }, // 0xb6 . 0b_101011001011
    HuffmanCode { len: 12, code: 0x99a0 }, // 0xb7 . 0b_100110011010
    HuffmanCode { len: 12, code: 0xa600 }, // 0xb8 . 0b_101001100000
    HuffmanCode { len: 12, code: 0x99b0 }, // 0xb9 . 0b_100110011011
    HuffmanCode { len: 12, code: 0x9e50 }, // 0xba . 0b_100111100101
    HuffmanCode { len: 12, code: 0x98c0 }, // 0xbb . 0b_100110001100
    HuffmanCode { len: 12, code: 0x96d0 }, // 0xbc . 0b_100101101101
    HuffmanCode { len: 12, code: 0x8e80 }, // 0xbd . 0b_100011101000
    HuffmanCode { len: 12, code: 0x9920 }, // 0xbe . 0b_100110010010
    HuffmanCode { len: 12, code: 0x98d0 }, // 0xbf . 0b_100110001101
    //
    HuffmanCode { len: 12, code: 0xaca0 }, // 0xc0 . 0b_101011001010
    HuffmanCode { len: 12, code: 0x8ef0 }, // 0xc1 . 0b_100011101111
    HuffmanCode { len: 12, code: 0x8fd0 }, // 0xc2 . 0b_100011111101
    HuffmanCode { len: 12, code: 0x8ba0 }, // 0xc3 . 0b_100010111010
    HuffmanCode { len: 12, code: 0x8f40 }, // 0xc4 . 0b_100011110100
    HuffmanCode { len: 12, code: 0x8e90 }, // 0xc5 . 0b_100011101001
    HuffmanCode { len: 12, code: 0x8bb0 }, // 0xc6 . 0b_100010111011
    HuffmanCode { len: 12, code: 0x5de0 }, // 0xc7 . 0b_010111011110
    HuffmanCode { len: 12, code: 0x8f50 }, // 0xc8 . 0b_100011110101
    HuffmanCode { len: 12, code: 0x5ce0 }, // 0xc9 . 0b_010111001110
    HuffmanCode { len: 12, code: 0x5df0 }, // 0xca . 0b_010111011111
    HuffmanCode { len: 12, code: 0x8d80 }, // 0xcb . 0b_100011011000
    HuffmanCode { len: 12, code: 0x5fe0 }, // 0xcc . 0b_010111111110
    HuffmanCode { len: 12, code: 0x5360 }, // 0xcd . 0b_010100110110
    HuffmanCode { len: 12, code: 0x8ee0 }, // 0xce . 0b_100011101110
    HuffmanCode { len: 12, code: 0x5ff0 }, // 0xcf . 0b_010111111111
    //
    HuffmanCode { len: 10, code: 0x0600 }, // 0xd0 . 0b_0000011000
    HuffmanCode { len: 11, code: 0x5a20 }, // 0xd1 . 0b_01011010001
    HuffmanCode { len: 12, code: 0x96c0 }, // 0xd2 . 0b_100101101100
    HuffmanCode { len: 12, code: 0x5d90 }, // 0xd3 . 0b_010111011001
    HuffmanCode { len: 12, code: 0x5a90 }, // 0xd4 . 0b_010110101001
    HuffmanCode { len: 12, code: 0x5300 }, // 0xd5 . 0b_010100110000
    HuffmanCode { len: 12, code: 0x07b0 }, // 0xd6 . 0b_000001111011
    HuffmanCode { len: 12, code: 0x4cb0 }, // 0xd7 . 0b_010011001011
    HuffmanCode { len: 12, code: 0x5a80 }, // 0xd8 . 0b_010110101000
    HuffmanCode { len: 12, code: 0x5370 }, // 0xd9 . 0b_010100110111
    HuffmanCode { len: 12, code: 0x5d20 }, // 0xda . 0b_010111010010
    HuffmanCode { len: 12, code: 0x44e0 }, // 0xdb . 0b_010001001110
    HuffmanCode { len: 12, code: 0x5d30 }, // 0xdc . 0b_010111010011
    HuffmanCode { len: 12, code: 0x4b50 }, // 0xdd . 0b_010010110101
    HuffmanCode { len: 12, code: 0x8fc0 }, // 0xde . 0b_100011111100
    HuffmanCode { len: 11, code: 0x44c0 }, // 0xdf . 0b_01000100110
    //
    HuffmanCode { len: 8, code: 0x4600 },  // 0xe0 . 0b_01000110
    HuffmanCode { len: 10, code: 0x8f00 }, // 0xe1 . 0b_1000111100
    HuffmanCode { len: 11, code: 0x07c0 }, // 0xe2 . 0b_00000111110
    HuffmanCode { len: 12, code: 0x5310 }, // 0xe3 . 0b_010100110001
    HuffmanCode { len: 12, code: 0x4480 }, // 0xe4 . 0b_010001001000
    HuffmanCode { len: 13, code: 0xadf8 }, // 0xe5 . 0b_1010110111111
    HuffmanCode { len: 12, code: 0x4ca0 }, // 0xe6 . 0b_010011001010
    HuffmanCode { len: 13, code: 0xb178 }, // 0xe7 . 0b_1011000101111
    HuffmanCode { len: 12, code: 0x5cf0 }, // 0xe8 . 0b_010111001111
    HuffmanCode { len: 13, code: 0xa628 }, // 0xe9 . 0b_1010011000101
    HuffmanCode { len: 12, code: 0x44f0 }, // 0xea . 0b_010001001111
    HuffmanCode { len: 12, code: 0x4b40 }, // 0xeb . 0b_010010110100
    HuffmanCode { len: 12, code: 0x8d90 }, // 0xec . 0b_100011011001
    HuffmanCode { len: 12, code: 0xad50 }, // 0xed . 0b_101011010101
    HuffmanCode { len: 11, code: 0x04c0 }, // 0xee . 0b_00000100110
    HuffmanCode { len: 9, code: 0x0700 },  // 0xef . 0b_000001110
    //
    HuffmanCode { len: 6, code: 0x5400 },  // 0xf0 . 0b_010101
    HuffmanCode { len: 9, code: 0x9f00 },  // 0xf1 . 0b_100111110
    HuffmanCode { len: 11, code: 0xac20 }, // 0xf2 . 0b_10101100001
    HuffmanCode { len: 11, code: 0x5f80 }, // 0xf3 . 0b_01011111100
    HuffmanCode { len: 12, code: 0x4490 }, // 0xf4 . 0b_010001001001
    HuffmanCode { len: 13, code: 0xad40 }, // 0xf5 . 0b_1010110101000
    HuffmanCode { len: 13, code: 0xb170 }, // 0xf6 . 0b_1011000101110
    HuffmanCode { len: 13, code: 0x9938 }, // 0xf7 . 0b_1001100100111
    HuffmanCode { len: 12, code: 0x07a0 }, // 0xf8 . 0b_000001111010
    HuffmanCode { len: 13, code: 0xa620 }, // 0xf9 . 0b_1010011000100
    HuffmanCode { len: 13, code: 0xadf0 }, // 0xfa . 0b_1010110111110
    HuffmanCode { len: 13, code: 0x9930 }, // 0xfb . 0b_1001100100110
    HuffmanCode { len: 13, code: 0xad48 }, // 0xfc . 0b_1010110101001
    HuffmanCode { len: 12, code: 0x5d80 }, // 0xfd . 0b_010111011000
    HuffmanCode { len: 11, code: 0xac80 }, // 0xfe . 0b_10101100100
    HuffmanCode { len: 9, code: 0x9d00 },  // 0xff . 0b_100111010
];

#[cfg(test)]
mod tests {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}

    #[test]
    fn normal_types() {
        is_full::<HuffmanCode>();
    }
}
