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

## Earlier implementation

I originally implemented this crate as a C++ library:
[Library-Blackbox](https://github.com/martinbudden/Library-Blackbox).
