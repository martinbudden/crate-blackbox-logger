# `blackbox-logger` Rust Crate<br>![license](https://img.shields.io/badge/License-GPLv3_or_later-blue.svg) ![open source](https://badgen.net/badge/open/source/blue?icon=github)

Betaflight compatible blackbox flight data recorder.
That is it produces output that be viewed using the [Betaflight Blackbox Explorer](https://blackbox.betaflight.com/),
and can be processed by Nick's [Blackbox tools](https://github.com/cleanflight/blackbox-tools).

`blackbox-logger` is based on the Blackbox implementation by Nicholas Sherlock (aka thenickdude),
see <https://github.com/thenickdude/blackbox>.

The main changes are:

1. Code is written in Rust.
2. Dependencies (ie configs, features, sensors etc) have been removed so this library can be used on its own.
3. Support for compressing P frames using Huffman encoding.

This crate is `no_std`, that it does not link to the standard library and so does not depend on an operating system
and uses no allocation. This means it is suitable for embedded systems.

## Rational for Huffman encoding P frames

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

## Rational for not encoding other types of frames

1. **I frames** are key-frames, that is they are used to reset values if there has been a corruption at some point. So they should not be further encoded.
2. **H frames**, **S frames**, and **E frames** are small and rare, so the impact of compressing them is very small.
3. **G frames** are fairly rare, so the impact of compressing them is small.

## Earlier implementation

I originally implemented this crate as a C++ library:
[Library-Blackbox](https://github.com/martinbudden/Library-Blackbox).
