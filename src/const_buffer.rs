use alloc::vec::Vec;
use core::ops::{Index, IndexMut, Range};

use crate::{
    const_buffer_ref::{as_mut_slices, as_slices, channel_unchecked, channel_unchecked_mut},
    ChannelBufferRef, ChannelBufferRefMut,
};

/// A memory-efficient buffer of samples with a fixed compile-time number of channels
/// each with a fixed runtime number of frames (samples in a single channel of audio).
#[derive(Debug, Clone)]
pub struct ChannelBuffer<T: Clone + Copy + Default, const CHANNELS: usize> {
    data: Vec<T>,
    frames: usize,
}

impl<T: Clone + Copy + Default, const CHANNELS: usize> ChannelBuffer<T, CHANNELS> {
    /// Create an empty [`ChannelBuffer`] with no allocated capacity.
    pub const fn empty() -> Self {
        Self {
            data: Vec::new(),
            frames: 0,
        }
    }

    /// Create a new [`ChannelBuffer`] allocated with the given number of channels
    /// each allocated with the given number of frames (samples in a single channel
    /// of audio).
    ///
    /// All data will be initialized with the default value.
    pub fn new(frames: usize) -> Self {
        let buffer_len = CHANNELS * frames;

        let mut buffer = Vec::new();
        buffer.reserve_exact(buffer_len);
        buffer.resize(buffer_len, Default::default());

        Self {
            data: buffer,
            frames,
        }
    }

    /// Create a new [`ChannelBuffer`] allocated with the given number of channels
    /// each allocated with the given number of frames (samples in a single channel
    /// of audio).
    ///
    /// No data will be initialized.
    ///
    /// # Safety
    /// Any data must be initialized before reading.
    pub unsafe fn new_uninit(frames: usize) -> Self {
        let buffer_len = CHANNELS * frames;

        let mut buffer = Vec::new();
        buffer.reserve_exact(buffer_len);
        buffer.set_len(buffer_len);

        Self {
            data: buffer,
            frames,
        }
    }

    /// The number of channels in this buffer.
    pub fn channels(&self) -> usize {
        CHANNELS
    }

    /// The number of frames (samples in a single channel of audio) that are allocated
    /// in this buffer.
    pub fn frames(&self) -> usize {
        self.frames
    }

    #[inline(always)]
    /// Get an immutable reference to the channel at `index`. The slice will have a length
    /// of `self.frames()`.
    ///
    /// Returns `None` if `index` is out of bounds.
    pub fn channel(&self, index: usize) -> Option<&[T]> {
        if index < CHANNELS {
            // SAFETY:
            // We haved checked that `index` is within bounds.
            unsafe { Some(self.channel_unchecked(index)) }
        } else {
            None
        }
    }

    #[inline(always)]
    /// Get a mutable reference to the channel at `index`. The slice will have a length
    /// of `self.frames()`.
    ///
    /// Returns `None` if `index` is out of bounds.
    pub fn channel_mut(&mut self, index: usize) -> Option<&mut [T]> {
        if index < CHANNELS {
            // SAFETY:
            // We haved checked that `index` is within bounds.
            unsafe { Some(self.channel_unchecked_mut(index)) }
        } else {
            None
        }
    }

    #[inline(always)]
    /// Get an immutable reference to the channel at `index`. The slice will have a length
    /// of `self.frames()`.
    ///
    /// # Safety
    /// `index` must be less than `self.channels()`
    pub unsafe fn channel_unchecked(&self, index: usize) -> &[T] {
        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // so this is always within range.
        unsafe { channel_unchecked::<T, CHANNELS>(&self.data, self.frames, index, 0, self.frames) }
    }

    #[inline(always)]
    /// Get a mutable reference to the channel at `index`. The slice will have a length
    /// of `self.frames()`.
    ///
    /// # Safety
    /// `index` must be less than `self.channels()`
    pub unsafe fn channel_unchecked_mut(&mut self, index: usize) -> &mut [T] {
        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // so this is always within range.
        unsafe {
            channel_unchecked_mut::<T, CHANNELS>(&mut self.data, self.frames, index, 0, self.frames)
        }
    }

    /// Get all channels as immutable slices. Each slice will have a length of `self.frames()`.
    #[inline]
    pub fn as_slices(&self) -> [&[T]; CHANNELS] {
        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // so this is always within range.
        unsafe { as_slices(&self.data, self.frames, 0, self.frames) }
    }

    /// Get all channels as mutable slices. Each slice will have a length of `self.frames()`.
    #[inline]
    pub fn as_mut_slices(&mut self) -> [&mut [T]; CHANNELS] {
        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // so this is always within range.
        unsafe { as_mut_slices(&mut self.data, self.frames, 0, self.frames) }
    }

    /// Get all channels as immutable slices with the given length in frames.
    ///
    /// If `frames > self.frames()`, then each slice will have a length of `self.frames()`
    /// instead.
    #[inline]
    pub fn as_slices_with_length(&self, frames: usize) -> [&[T]; CHANNELS] {
        let frames = frames.min(self.frames);

        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // and we have constrained `frames` above, so this is always within range.
        unsafe { as_slices(&self.data, self.frames, 0, frames) }
    }

    /// Get all channels as mutable slices with the given length in frames.
    ///
    /// If `frames > self.frames()`, then each slice will have a length of `self.frames()`
    /// instead.
    #[inline]
    pub fn as_mut_slices_with_length(&mut self, frames: usize) -> [&mut [T]; CHANNELS] {
        let frames = frames.min(self.frames);

        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // and we have constrained `frames` above, so this is always within range.
        unsafe { as_mut_slices(&mut self.data, self.frames, 0, frames) }
    }

    /// Get all channels as immutable slices in the given range.
    ///
    /// If all or part of the range falls out of bounds, then only the part that falls
    /// within range will be returned.
    #[inline]
    pub fn as_slices_with_range(&self, range: Range<usize>) -> [&[T]; CHANNELS] {
        let start_frame = range.start.min(self.frames);
        let frames = range.end.min(self.frames) - start_frame;

        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // and we have constrained `start_frame` and `frames` above, so this is always
        // within range.
        unsafe { as_slices(&self.data, self.frames, start_frame, frames) }
    }

    /// Get all channels as mutable slices in the given range.
    ///
    /// If all or part of the range falls out of bounds, then only the part that falls
    /// within range will be returned.
    #[inline]
    pub fn as_mut_slices_with_range(&mut self, range: Range<usize>) -> [&mut [T]; CHANNELS] {
        let start_frame = range.start.min(self.frames);
        let frames = range.end.min(self.frames) - start_frame;

        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // and we have constrained `start_frame` and `frames` above, so this is always
        // within range.
        unsafe { as_mut_slices(&mut self.data, self.frames, start_frame, frames) }
    }

    /// Get the entire contents of the buffer as a single immutable slice.
    pub fn raw(&self) -> &[T] {
        &self.data
    }

    /// Get the entire contents of the buffer as a single mutable slice.
    pub fn raw_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Clear all data with the default value.
    pub fn clear(&mut self) {
        self.raw_mut().fill(T::default());
    }

    /// Clear all data in each channel up to `frames` with the default value.
    pub fn clear_frames(&mut self, frames: usize) {
        for ch in self.as_mut_slices_with_length(frames) {
            ch.fill(T::default());
        }
    }

    pub fn as_ref<'a>(&'a self) -> ChannelBufferRef<'a, T, CHANNELS> {
        ChannelBufferRef {
            data: &self.data,
            frames: self.frames,
        }
    }

    pub fn as_mut<'a>(&'a mut self) -> ChannelBufferRefMut<'a, T, CHANNELS> {
        ChannelBufferRefMut {
            data: &mut self.data,
            frames: self.frames,
        }
    }
}

impl<T: Clone + Copy + Default, const CHANNELS: usize> Index<usize> for ChannelBuffer<T, CHANNELS> {
    type Output = [T];

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        self.channel(index).unwrap()
    }
}

impl<T: Clone + Copy + Default, const CHANNELS: usize> IndexMut<usize>
    for ChannelBuffer<T, CHANNELS>
{
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.channel_mut(index).unwrap()
    }
}

impl<T: Clone + Copy + Default, const CHANNELS: usize> Default for ChannelBuffer<T, CHANNELS> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> Into<ChannelBufferRef<'a, T, CHANNELS>>
    for &'a ChannelBuffer<T, CHANNELS>
{
    fn into(self) -> ChannelBufferRef<'a, T, CHANNELS> {
        self.as_ref()
    }
}

impl<'a, T: Clone + Copy + Default, const CHANNELS: usize>
    Into<ChannelBufferRefMut<'a, T, CHANNELS>> for &'a mut ChannelBuffer<T, CHANNELS>
{
    fn into(self) -> ChannelBufferRefMut<'a, T, CHANNELS> {
        self.as_mut()
    }
}

impl<T: Clone + Copy + Default, const CHANNELS: usize> Into<Vec<T>> for ChannelBuffer<T, CHANNELS> {
    fn into(self) -> Vec<T> {
        self.data
    }
}
