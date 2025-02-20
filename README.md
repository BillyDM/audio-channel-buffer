# Audio Channel Buffer
![Test](https://github.com/BillyDM/audio-channel-buffer/workflows/Test/badge.svg)
[![Documentation](https://docs.rs/audio-channel-buffer/badge.svg)][documentation]
[![Crates.io](https://img.shields.io/crates/v/audio-channel-buffer.svg)](https://crates.io/crates/audio-channel-buffer)
[![License](https://img.shields.io/crates/l/audio-channel-buffer.svg)](https://github.com/BillyDM/audio-channel-buffer/blob/master/LICENSE)

A collection of memory-efficient audio buffer types for realtime applications. These may have better cache efficiency and take up less memory than `Vec<Vec<T>>`.

This library can be used with or without the standard library and with or without an allocator.

Note, this library is meant to be used when the number of frames (samples in a single channel of audio) are not known at compile-time. If the number of frames are known at compile-time, then you can simply use `Vec<[T; FRAMES]>` or `[[T: FRAMES]; CHANNELS]` instead to get the same effect.