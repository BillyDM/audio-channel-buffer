use alloc::vec::Vec;
use core::num::NonZeroUsize;
use core::ops::{Index, IndexMut, Range};

use arrayvec::ArrayVec;

use crate::{
    var_buffer_ref::{as_mut_slices, as_slices, channel_unchecked, channel_unchecked_mut},
    VarChannelBufferRef, VarChannelBufferRefMut,
};

/// A memory-efficient buffer of samples with a fixed runtime number of channels each
/// with a fixed runtime number of frames (samples in a single channel of audio).
#[derive(Debug, Clone)]
pub struct VarChannelBuffer<T: Clone + Copy + Default, const MAX_CHANNELS: usize> {
    data: Vec<T>,
    channels: NonZeroUsize,
    frames: usize,
}

impl<T: Clone + Copy + Default, const MAX_CHANNELS: usize> VarChannelBuffer<T, MAX_CHANNELS> {
    /// Create an empty [`VarChannelBuffer`] with no allocated capacity.
    pub const fn empty() -> Self {
        Self {
            data: Vec::new(),
            channels: NonZeroUsize::MIN,
            frames: 0,
        }
    }

    /// Create a new [`VarChannelBuffer`] allocated with the given number of channels
    /// each allocated with the given number of frames (samples in a single channel
    /// of audio).
    ///
    /// All data will be initialized with the default value.
    ///
    /// # Panics
    /// Panics if `channels.get() > MAX_CHANNELS`.
    pub fn new(channels: NonZeroUsize, frames: usize) -> Self {
        assert!(channels.get() <= MAX_CHANNELS);

        let buffer_len = channels.get() * frames;

        let mut buffer = Vec::new();
        buffer.reserve_exact(buffer_len);
        buffer.resize(buffer_len, Default::default());

        Self {
            data: buffer,
            channels,
            frames,
        }
    }

    /// Create a new [`VarChannelBuffer`] allocated with the given number of channels
    /// each allocated with the given number of frames (samples in a single channel
    /// of audio).
    ///
    /// No data will be initialized.
    ///
    /// # Safety
    /// Any data must be initialized before reading.
    ///
    /// # Panics
    /// Panics if `channels.get() > MAX_CHANNELS`.
    pub unsafe fn new_uninit(channels: NonZeroUsize, frames: usize) -> Self {
        assert!(channels.get() <= MAX_CHANNELS);

        let buffer_len = channels.get() * frames;

        let mut buffer = Vec::new();
        buffer.reserve_exact(buffer_len);
        buffer.set_len(buffer_len);

        Self {
            data: buffer,
            channels,
            frames,
        }
    }

    /// The number of channels in this buffer.
    pub fn channels(&self) -> NonZeroUsize {
        self.channels
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
        if index < MAX_CHANNELS {
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
        if index < MAX_CHANNELS {
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
        unsafe {
            channel_unchecked::<T, MAX_CHANNELS>(&self.data, self.frames, index, 0, self.frames)
        }
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
            channel_unchecked_mut::<T, MAX_CHANNELS>(
                &mut self.data,
                self.frames,
                index,
                0,
                self.frames,
            )
        }
    }

    /// Get all channels as immutable slices. Each slice will have a length of `self.frames()`.
    #[inline]
    pub fn as_slices(&self) -> ArrayVec<&[T], MAX_CHANNELS> {
        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // so this is always within range.
        unsafe { as_slices(&self.data, self.channels.get(), self.frames, 0, self.frames) }
    }

    /// Get all channels as mutable slices. Each slice will have a length of `self.frames()`.
    #[inline]
    pub fn as_mut_slices(&mut self) -> ArrayVec<&mut [T], MAX_CHANNELS> {
        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // so this is always within range.
        unsafe {
            as_mut_slices(
                &mut self.data,
                self.channels.get(),
                self.frames,
                0,
                self.frames,
            )
        }
    }

    /// Get all channels as immutable slices with the given length in frames.
    ///
    /// If `frames > self.frames()`, then each slice will have a length of `self.frames()`
    /// instead.
    #[inline]
    pub fn as_slices_with_length(&self, frames: usize) -> ArrayVec<&[T], MAX_CHANNELS> {
        let frames = frames.min(self.frames);

        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // and we have constrained `frames` above, so this is always within range.
        unsafe { as_slices(&self.data, self.channels.get(), self.frames, 0, frames) }
    }

    /// Get all channels as mutable slices with the given length in frames.
    ///
    /// If `frames > self.frames()`, then each slice will have a length of `self.frames()`
    /// instead.
    #[inline]
    pub fn as_mut_slices_with_length(&mut self, frames: usize) -> ArrayVec<&mut [T], MAX_CHANNELS> {
        let frames = frames.min(self.frames);

        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // and we have constrained `frames` above, so this is always within range.
        unsafe { as_mut_slices(&mut self.data, self.channels.get(), self.frames, 0, frames) }
    }

    /// Get all channels as immutable slices in the given range.
    ///
    /// If all or part of the range falls out of bounds, then only the part that falls
    /// within range will be returned.
    #[inline]
    pub fn as_slices_with_range(&self, range: Range<usize>) -> ArrayVec<&[T], MAX_CHANNELS> {
        let start_frame = range.start.min(self.frames);
        let frames = range.end.min(self.frames) - start_frame;

        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // and we have constrained `start_frame` and `frames` above, so this is always
        // within range.
        unsafe {
            as_slices(
                &self.data,
                self.channels.get(),
                self.frames,
                start_frame,
                frames,
            )
        }
    }

    /// Get all channels as mutable slices in the given range.
    ///
    /// If all or part of the range falls out of bounds, then only the part that falls
    /// within range will be returned.
    #[inline]
    pub fn as_mut_slices_with_range(
        &mut self,
        range: Range<usize>,
    ) -> ArrayVec<&mut [T], MAX_CHANNELS> {
        let start_frame = range.start.min(self.frames);
        let frames = range.end.min(self.frames) - start_frame;

        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // and we have constrained `start_frame` and `frames` above, so this is always
        // within range.
        unsafe {
            as_mut_slices(
                &mut self.data,
                self.channels.get(),
                self.frames,
                start_frame,
                frames,
            )
        }
    }

    /// Set the number of channels.
    ///
    /// This method may allocate and is not realtime-safe.
    ///
    /// # Panics
    /// Panics if `channels.get() > MAX_CHANNELS`.
    pub fn set_num_channels(&mut self, channels: NonZeroUsize) {
        if self.channels == channels {
            return;
        }

        assert!(channels.get() <= MAX_CHANNELS);

        let buffer_len = self.frames * channels.get();

        self.data.resize(buffer_len, T::default());

        self.channels = channels;
    }

    /// Set the number of channels without initialize any new data.
    ///
    /// This method may allocate and is not realtime-safe.
    ///
    /// # Safety
    /// Any data must be initialized before reading.
    ///
    /// # Panics
    /// Panics if `channels.get() > MAX_CHANNELS`.
    pub unsafe fn set_num_instances_uninit(&mut self, channels: NonZeroUsize) {
        if self.channels == channels {
            return;
        }

        assert!(channels.get() <= MAX_CHANNELS);

        let buffer_len = self.frames * channels.get();

        if self.data.len() < buffer_len {
            self.data.reserve(buffer_len - self.data.len());
        }

        self.data.set_len(buffer_len);

        self.channels = channels;
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

    pub fn as_ref<'a>(&'a self) -> VarChannelBufferRef<'a, T, MAX_CHANNELS> {
        VarChannelBufferRef {
            data: &self.data,
            channels: self.channels,
            frames: self.frames,
        }
    }

    pub fn as_mut<'a>(&'a mut self) -> VarChannelBufferRefMut<'a, T, MAX_CHANNELS> {
        VarChannelBufferRefMut {
            data: &mut self.data,
            channels: self.channels,
            frames: self.frames,
        }
    }
}

impl<T: Clone + Copy + Default, const MAX_CHANNELS: usize> Index<usize>
    for VarChannelBuffer<T, MAX_CHANNELS>
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        self.channel(index).unwrap()
    }
}

impl<T: Clone + Copy + Default, const MAX_CHANNELS: usize> IndexMut<usize>
    for VarChannelBuffer<T, MAX_CHANNELS>
{
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.channel_mut(index).unwrap()
    }
}

impl<T: Clone + Copy + Default, const MAX_CHANNELS: usize> Default
    for VarChannelBuffer<T, MAX_CHANNELS>
{
    fn default() -> Self {
        Self::empty()
    }
}

impl<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize>
    Into<VarChannelBufferRef<'a, T, MAX_CHANNELS>> for &'a VarChannelBuffer<T, MAX_CHANNELS>
{
    fn into(self) -> VarChannelBufferRef<'a, T, MAX_CHANNELS> {
        self.as_ref()
    }
}

impl<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize>
    Into<VarChannelBufferRefMut<'a, T, MAX_CHANNELS>>
    for &'a mut VarChannelBuffer<T, MAX_CHANNELS>
{
    fn into(self) -> VarChannelBufferRefMut<'a, T, MAX_CHANNELS> {
        self.as_mut()
    }
}

impl<T: Clone + Copy + Default, const MAX_CHANNELS: usize> Into<Vec<T>>
    for VarChannelBuffer<T, MAX_CHANNELS>
{
    fn into(self) -> Vec<T> {
        self.data
    }
}
