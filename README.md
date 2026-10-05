# `blackbox-logger` Rust Crate<br>[![Crates.io](https://img.shields.io/crates/v/blackbox-logger.svg)](https://crates.io/crates/blackbox-logger) [![Documentation](https://docs.rs/blackbox-logger/badge.svg)](https://docs.rs/blackbox-logger) [![License: GPLv3](https://img.shields.io/badge/License-GPLv3_or_later-blue.svg)](https://opensource.org/licenses/gpl-3.0) ![open source](https://badgen.net/badge/open/source/blue?icon=github)

Betaflight compatible blackbox flight data encoder.

That is it produces output that be viewed using the [Betaflight Blackbox Explorer](https://blackbox.betaflight.com/),
and can be processed by Nick's [Blackbox tools](https://github.com/cleanflight/blackbox-tools).

`blackbox-logger` encodes the flight data in blackbox format, but does not write that data to storage.
It is up to calling the application to store that data as it chooses, eg write it to flash, store it SD card,
or write it to a serial-attached storage device.

`blackbox-logger` is based on the Blackbox implementation by Nicholas Sherlock (aka thenickdude),
see <https://github.com/thenickdude/blackbox>.

The main changes are:

1. The code is written in Rust.
2. Dependencies (ie configs, features, sensors etc) have been removed so this library can be used on its own.
3. It has added support for optionally compressing **P-frames** using Huffman encoding.

This crate is `no_std`, `no alloc`, and the Minimum Supported Rust Version (MSRV) is `Rust 1.89`.

## Blackbox frame types

Blackbox logs contain several types of frame.

### I-frames

I-frames are Intra-frames (aka Keyframes): independent frames that do not require any other frames to decode.

They serve as "reset" points to stop error propagation.

They are written at initialization and periodically thereafter (typically every 32ms).

### P-frames

P-frames are Inter-frames they store data as changes (deltas) from previous frames.
This means they are smaller than I-frames.

They are typically recorded at a frequency of 1kHz.

### S-frames

Slow frames, they are stored as intraframes (independent full-state records). S-frames contain slower-changing data.
They are written at the start of the log, and when the monitored data changes.

### G-frames

GPS data frames (independent frames). When GPS support is enabled these frames are written when new GPS data is received, typically at a rate of about 10Hz.

### H-frames

The GPS "home" frame (independent frames). When GPS support is enabled these are written at the start of the log.

### E-frames

Event frames (independent frames), these contain Blackbox events, such as:

- arming beep synchronization events
- flight-mode changes.

## Huffman compression

The `huffman` Cargo feature adds an optional second level of compression for **P-frames**.

### Rational for Huffman encoding P-frames

The standard blackbox encoding works by making a prediction of the value of a field and storing the difference from that prediction.
(That's a gross simplification, but is enough to understand why further compressing using Huffman encoding makes sense).

If the prediction is good (which it generally is) it means a small value will be stored. So small values will be much more frequent
than large values. So we have some values that are quite frequent and other values that are much rarer - this is ripe for Huffman compression.
The frequent values are stored in fewer than 8 bits (indeed zero is so frequent that it is stored in 2 bits), whereas infrequent
values are stored in more than 8 bits. So overall fewer bits are used.

If Huffman encoding is switched on, then each time a **P-frame** is generated it will be Huffman encoded. If the encoded frame
is smaller than the **P-frame** then it will be written to file as a **Q-frame**. If the Huffman encoded frame is larger than
the **P-frame** then it will be written as a standard **P-frame**.

Huffman encoding is extremely fast. Encoding a byte involves just a table lookup and some bit shifting, so the
overhead of trying to encode each **P-frame** is negligible.

Early testing indicates that **Q-frames** are often less than half the size of the corresponding **P-frame**.

### Rational for not encoding other types of frames

1. **I-frames** are key-frames, that is they are used to reset values if there has been data corruption at some point. So they should not be further encoded.
2. **H-frames**, **S-frames**, and **E-frames** are small and rare, so the impact of compressing them is very small.
3. **G-frames** are fairly rare, so the impact of compressing them is small. Also their content is not conducive to Huffman encoding.

### Reading a Huffman encoded Blackbox log

The process is very similar to reading an unencoded log:

1. Read the log header.
2. At the end of the header look for a **T-frame**
3. If there is no **T-frame** then the file is not encoded, an just read it as a normal log.
4. If there is a **T-frame**, then use it to build a Huffman decoding tree.
5. Proceed to read the file normally, process **I**, **P**, **S**, **E**, **G**, and **H** frames normally.
6. If you encounter a **Q-frame**, then that is an encoded **P-frame**, so decode it using the Huffman tree.
   This will produce a **P-frame**, which you just treat as a normal **P-frame**.

A **T-frame** consists of the letter 'T' followed by 768 bytes of binary data, this is 256 triplets.
Each triplet consists of a `u8` encoded length followed by a `u16` of the encoded bits.

### Example Blackbox log file

An example of a Blackbox log file is given below. The header is in plain text and the subsequent binary data is
formatted as a hex dump.

The I-frames and P-frames are clearly visible.

The P-frames are 30 bytes long, and its not uncommon for more than 18 of those bytes to be zero.
Using Huffman compression those 18 bytes will compress down to 5 bytes. So its easy to see how
a 30-byte P-frame could compress to less than 20 bytes.

```text
H Product:Blackbox flight data recorder by Nicholas Sherlock
H Data version:2
H Field I name:loopIteration,time,axisP[0],axisP[1],axisP[2],axisI[0],axisI[1],axisI[2],axisD[0],axisD[1],axisF[0],axisF[1],axisF[2],rcCommand[0],rcCommand[1],rcCommand[2],rcCommand[3],setpoint[0],setpoint[1],setpoint[2],setpoint[3],vbatLatest,amperageLatest,magADC[0],magADC[1],magADC[2],baroAlt,rssi,gyroADC[0],gyroADC[1],gyroADC[2],accSmooth[0],accSmooth[1],accSmooth[2],imuQuaternion[0],imuQuaternion[1],imuQuaternion[2],motor[0],motor[1],motor[2],motor[3]
H Field I signed:0,0,1,1,1,1,1,1,1,1,1,1,1,1,1,1,0,1,1,1,1,0,1,1,1,1,1,0,1,1,1,1,1,1,1,1,1,0,0,0,0
H Field I predictor:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,9,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,11,5,5,5
H Field I encoding:1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,3,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,1,0,0,0
H Field P predictor:6,2,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,3,3,3,3,3,3,3,3,3,3,3,3,3
H Field P encoding:9,0,0,0,0,7,7,7,0,0,0,0,0,8,8,8,8,8,8,8,8,6,6,6,6,6,6,6,0,0,0,0,0,0,0,0,0,0,0,0,0
...
H Firmware type:Betaflight
H Firmware revision:Betaflight 2025.12.5 (7348054f2) STM32F405
H Firmware date:Jul  6 2026 07:51:16
...
H I interval:128
H P interval:16
H P ratio:8
...
00001930  64 62 61 6e 64 3a 32 30  0a 48 20 70 6f 73 5f 68  |dband:20.H pos_h|
00001940  6f 6c 64 5f 77 69 74 68  6f 75 74 5f 6d 61 67 3a  |old_without_mag:|
00001950  30 0a 48 20 70 6f 73 5f  68 6f 6c 64 5f 64 65 61  |0.H pos_hold_dea|
00001960  64 62 61 6e 64 3a 35 0a  49 00 dd cc fd 5a 00 01  |dband:5.I....Z..|
00001970  00 00 00 00 00 01 00 00  00 02 02 01 e8 07 00 00  |................|
00001980  00 00 05 63 f7 07 e2 08  db 0f 26 f5 05 00 02 00  |...c......&.....|
00001990  10 0f ee 1f 0c 36 ff d1  03 00 00 00 00 45 00 a0  |.....6.......E..|
000019a0  e8 f3 5a 53 01 00 05 50  a4 3f 00 00 00 00 00 02  |..ZS...P.?......|
000019b0  00 00 00 00 00 1d 08 03  1e 0a 00 00 00 00 08 01  |................|
000019c0  00 00 00 00 00 00 00 50  02 00 00 00 00 00 00 00  |.......P........|
000019d0  00 00 00 00 14 0d 1b 00  00 00 05 04 04 04 00 00  |................|
000019e0  00 00 00 00 50 01 00 00  00 00 01 00 00 00 00 00  |....P...........|
000019f0  00 00 00 01 00 0b 00 08  02 00 00 00 00 00 00 50  |...............P|
00001a00  02 00 02 00 00 02 00 00  00 00 00 00 1a 5e 10 1c  |.............^..|
00001a10  00 00 00 0f 01 08 00 00  00 00 00 00 00 50 01 00  |.............P..|
00001a20  00 00 00 00 02 00 00 00  00 00 1c 0e 0e 10 00 00  |................|
00001a30  00 0b 07 02 02 00 00 00  00 00 00 50 02 01 00 00  |...........P....|
00001a40  00 00 00 00 00 00 00 00  1d 02 06 1d 0e 00 00 00  |................|
00001a50  07 07 07 02 00 00 00 00  00 00 50 01 00 00 00 00  |..........P.....|
00001a60  00 01 00 00 00 00 00 1c  17 1e 13 00 00 00 01 00  |................|
00001a70  09 00 01 00 00 00 00 00  49 80 01 f1 c9 ff 5a 00  |........I.....Z.|
00001a80  00 00 00 00 00 00 00 00  00 00 02 02 01 e8 07 00  |................|
00001a90  00 00 00 00 05 8d 08 9e  09 c7 0f 26 f6 05 00 00  |...........&....|
00001aa0  00 13 0d ea 1f 12 34 ff  d1 03 00 00 00 00 50 a4  |......4.......P.|
00001ab0  3f 00 00 00 00 00 00 00  00 00 00 00 1e 5d 08 1d  |?............]..|
00001ac0  06 00 00 00 08 0a 02 00  00 00 00 00 00 00 50 00  |..............P.|
00001ad0  00 00 00 00 00 00 00 00  00 00 00 1c 1e 06 17 00  |................|
00001ae0  00 00 0a 0c 08 02 00 00  00 00 00 00 50 02 00 00  |............P...|
00001af0  00 00 00 00 00 00 00 00  00 1d 03 05 18 1e 00 00  |................|
00001b00  00 08 06 0a 02 00 00 00  00 00 00 50 00 00 00 00  |...........P....|
00001b10  00 00 00 00 00 00 00 00  1c 11 06 27 00 00 00 04  |...........'....|
00001b20  01 06 00 01 00 00 00 00  00 50 01 00 00 00 00 00  |.........P......|
00001b30  00 00 00 00 00 00 40 02  00 00 00 02 09 04 00 00  |......@.........|
00001b40  00 00 00 00 00 50 00 00  00 00 00 00 00 00 00 00  |.....P..........|
00001b50  00 00 16 ea 01 09 1c 00  00 00 00 09 00 00 00 00  |................|
00001b60  00 00 00 00 50 02 00 00  00 00 00 00 00 00 00 00  |....P...........|
00001b70  00 1c 01 13 15 00 00 00  03 05 01 02 00 00 00 00  |................|
00001b80  00 00 49 80 02 84 c7 81  5b 00 00 00 00 00 00 00  |..I.....[.......|
00001b90  00 00 00 00 02 02 01 e8  07 00 00 00 00 05 86 01  |................|
00001ba0  f3 07 9a 09 c9 0f 26 f7  05 00 00 00 01 0b f8 1f  |......&.........|
00001bb0  16 32 ff d1 03 00 00 00  00 50 a8 3f 00 00 00 00  |.2.......P.?....|
00001bc0  00 00 00 00 00 00 00 1c  06 09 19 00 00 00 00 02  |................|
00001bd0  04 02 00 00 00 00 00 00  50 03 00 00 00 00 00 00  |........P.......|
00001be0  00 00 00 00 00 40 02 00  00 00 00 00 04 02 00 00  |.....@..........|
00001bf0  00 00 00 00 50 00 00 00  00 00 00 00 00 00 00 00  |....P...........|
00001c00  00 1a d1 01 0a 0c 00 00  00 01 03 02 02 00 01 00  |................|
00001c10  00 00 00 50 00 01 00 00  00 00 00 00 00 00 00 00  |...P............|
00001c20  1c 01 05 19 00 00 00 03  03 05 02 00 01 00 00 00  |................|
00001c30  00 50 02 02 00 00 00 00  00 00 00 00 00 00 1d 02  |.P..............|
00001c40  02 10 0c 00 00 00 00 01  03 00 00 00 00 00 00 00  |................|
00001c50  50 04 01 01 00 00 00 00  00 00 00 00 00 00 00 00  |P...............|
00001c60  00 02 02 02 02 01 00 00  00 00 00 50 09 00 02 00  |...........P....|
00001c70  00 00 00 00 00 00 00 00  5e 82 02 09 1d 1c 02 00  |........^.......|
00001c80  00 00 00 02 06 02 00 00  00 00 00 00 49 80 03 98  |............I...|
```

## Earlier implementation

I originally implemented this crate as a C++ library:
[Library-Blackbox](https://github.com/martinbudden/Library-Blackbox).
