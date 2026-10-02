# `blackbox-logger` Rust Crate<br>[![Crates.io](https://img.shields.io/crates/v/pidsk-controller.svg)](https://crates.io/crates/pidsk-controller) [![Documentation](https://docs.rs/pidsk-controller/badge.svg)](https://docs.rs/pidsk-controller) [![License: GPLv3](https://img.shields.io/badge/License-GPLv3_or_later-blue.svg)](https://opensource.org/licenses/gpl-3.0) ![open source](https://badgen.net/badge/open/source/blue?icon=github)

Betaflight compatible blackbox flight data recorder.
That is it produces output that be viewed using the [Betaflight Blackbox Explorer](https://blackbox.betaflight.com/),
and can be processed by Nick's [Blackbox tools](https://github.com/cleanflight/blackbox-tools).

`blackbox-logger` is based on the Blackbox implementation by Nicholas Sherlock (aka thenickdude),
see <https://github.com/thenickdude/blackbox>.

The main changes are:

1. Code is written in Rust.
2. Dependencies (ie configs, features, sensors etc) have been removed so this library can be used on its own.
3. Added support for compressing **P frames** using Huffman encoding.

This crate is `no_std`, `no alloc`, and the Minimum Supported Rust Version (MSRV) is `Rust 1.89`.

## Standard blackbox frame types

Blackbox logs contain several types of frame.

### I frames

I frames are keyframes containing the state required to initialize or resynchronise the decoder.

They are written at initialization and periodically thereafter.

### P frames

P frames contain the main high-frequency flight data. They may be recorded at frequencies of 1kHz or more.

Rather than storing every field as an absolute value, Blackbox predicts values and stores differences from those predictions.
When the prediction is good, these differences are usually small, which makes P frames considerably more compact.

### S frames

S frames contain slower-changing data.

### G frames

GPS data. When GPS support is enabled these frames are written when new GPS data is available, typically at a rate of about 10Hz.

### H frames

The GPS "home" frame written at the start of the log when GPS support is enabled.

### E frames

E frames contain Blackbox events, such as:

- arming beep synchronization events;
- flight-mode changes.

## Huffman compression

The `huffman` Cargo feature adds an optional second level of compression for **P frames**.

### Rational for Huffman encoding P frames

The standard blackbox encoding works making a prediction of the value of a field and storing the difference from that prediction.
(That's a gross simplification, but is enough to understand why further compressing using Huffman encoding makes sense).

If the prediction is good (which it generally is) it means a small value will be stored. So small values will be much more frequent
than large values. So we have some values that are quite frequent and other values that are much rarer - this is ripe for Huffman compression.
The frequent values are stored in fewer than 8 bits (indeed zero is so frequent that it is stored in 2 bits), whereas infrequent
values are stored in more than 8 bits. So overall fewer bits are used.

If Huffman encoding is switched on, then each time a **P frame** is generated it will be Huffman encoded. If the encoded frame
is smaller than the **P frame** then it will be written to file as a **Q frame**. If the Huffman encoded frame is larger than
the **P frame** then it will be written as a standard **P frame**.

Huffman encoding is extremely fast. Encoding a byte involves just a table lookup and some bit shifting, so the
overhead of trying to encode each **P frame** is negligible.

Early testing indicates that **Q frames** are often less than half the size of the corresponding **P frame**.

### Rational for not encoding other types of frames

1. **I frames** are key-frames, that is they are used to reset values if there has been a corruption at some point. So they should not be further encoded.
2. **H frames**, **S frames**, and **E frames** are small and rare, so the impact of compressing them is very small.
3. **G frames** are fairly rare, so the impact of compressing them is small. Also their content is not conducive to Huffman encoding.

### Reading a Huffman encoded Blackbox log

The process is very similar to reading an unencoded log:

1. Read the log header.
2. At the end of the header look for a **T frame**
3. If there is no **T frame** then the file is not encoded, an just read it as a normal log.
4. If there is a **T frame**, then use it to build a Huffman decoding tree.
5. Proceed to read the file normally, process **I**, **P**, **S**, **E**, **G**, and **H** frames normally.
6. If you encounter a **Q frame**, then that is an encoded **P frame**, so decode it using the Huffman tree.
   This will produce a **P frame**, which you just treat as a normal **P frame**.

A **T frame** consists of the letter 'T' followed by 768 bytes of binary data, this is 256 triplets.
Each triplet consists of a `u8` encoded length followed by a `u16` of the encoded bits.

## Earlier implementation

I originally implemented this crate as a C++ library:
[Library-Blackbox](https://github.com/martinbudden/Library-Blackbox).
