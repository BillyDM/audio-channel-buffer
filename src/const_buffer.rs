use alloc::vec::Vec;
use core::ops::{Index, IndexMut, Range};
use core::pin::Pin;

use crate::{ChannelBufferRef, ChannelBufferRefMut};

/// A memory-efficient buffer of samples with a fixed compile-time number of channels
/// each with a fixed runtime number of frames (samples in a single channel of audio).
///
/// This version uses an owned `Vec` as its data source.
#[derive(Debug)]
pub struct ChannelBuffer<T: Clone + Copy + Default + Sized + Unpin, const CHANNELS: usize> {
    data: Pin<Vec<T>>,
    offsets: [*mut T; CHANNELS],
    frames: usize,
}

impl<T: Clone + Copy + Default + Sized + Unpin, const CHANNELS: usize> ChannelBuffer<T, CHANNELS> {
    const _COMPILE_TIME_ASSERTS: () = {
        assert!(CHANNELS > 0);
    };

    /// Create an empty [`ChannelBuffer`] with no allocated capacity.
    pub fn empty() -> Self {
        let _ = Self::_COMPILE_TIME_ASSERTS;

        let mut data = Pin::new(Vec::<T>::new());

        let offsets = core::array::from_fn(|_| data.as_mut_ptr());

        Self {
            data,
            offsets,
            frames: 0,
        }
    }

    /// Create a new [`ChannelBuffer`] allocated with the given number of channels
    /// each allocated with the given number of frames (samples in a single channel
    /// of audio).
    ///
    /// All data will be initialized with the default value.
    pub fn new(frames: usize) -> Self {
        let _ = Self::_COMPILE_TIME_ASSERTS;

        let buffer_len = CHANNELS * frames;

        let mut data = Vec::<T>::new();
        data.reserve_exact(buffer_len);
        data.resize(buffer_len, Default::default());

        let mut data = Pin::new(data);

        // SAFETY:
        // * All of these pointers point to valid memory in the vec.
        // * We have asserted at compile-time that `CHANNELS` is non-zero.
        let offsets = unsafe { core::array::from_fn(|ch_i| data.as_mut_ptr().add(ch_i * frames)) };

        Self {
            data,
            offsets,
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
        let _ = Self::_COMPILE_TIME_ASSERTS;

        let buffer_len = CHANNELS * frames;

        let mut data = Vec::<T>::new();
        data.reserve_exact(buffer_len);
        data.set_len(buffer_len);

        let mut data = Pin::new(data);

        // SAFETY:
        // * All of these pointers point to valid memory in the vec.
        // * We have asserted at compile-time that `CHANNELS` is non-zero.
        let offsets = unsafe { core::array::from_fn(|ch_i| data.as_mut_ptr().add(ch_i * frames)) };

        Self {
            data,
            offsets,
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
    /// Get an immutable reference to the channel at `index`. The slice will have a length
    /// of `self.frames()`.
    ///
    /// # Safety
    /// `index` must be less than `self.channels()`
    pub unsafe fn channel_unchecked(&self, index: usize) -> &[T] {
        // SAFETY:
        //
        // * The constructors ensure that the pointed-to data vec has a length of at
        // least `frames * CHANNELS`.
        // * The caller upholds that `index` is within bounds.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * We have asserted at compile-time that `CHANNELS` is non-zero.
        core::slice::from_raw_parts(*self.offsets.get_unchecked(index), self.frames)
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
    /// Get a mutable reference to the channel at `index`. The slice will have a length
    /// of `self.frames()`.
    ///
    /// # Safety
    /// `index` must be less than `self.channels()`
    pub unsafe fn channel_unchecked_mut(&mut self, index: usize) -> &mut [T] {
        // SAFETY:
        //
        // * The constructors ensure that the pointed-to data vec has a length of at
        // least `frames * CHANNELS`.
        // * The caller upholds that `index` is within bounds.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * `self` is borrowed as mutable, ensuring that no other references to the
        // data Vec can exist.
        // * We have asserted at compile-time that `CHANNELS` is non-zero.
        core::slice::from_raw_parts_mut(*self.offsets.get_unchecked(index), self.frames)
    }

    /// Get all channels as immutable slices. Each slice will have a length of `self.frames()`.
    #[inline]
    pub fn as_slices(&self) -> [&[T]; CHANNELS] {
        // SAFETY:
        //
        // * The constructors ensure that the pointed-to data vec has a length of at
        // least `frames * CHANNELS`.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * We have asserted at compile-time that `CHANNELS` is non-zero.
        unsafe {
            core::array::from_fn(|ch_i| {
                core::slice::from_raw_parts(*self.offsets.get_unchecked(ch_i), self.frames)
            })
        }
    }

    /// Get all channels as mutable slices. Each slice will have a length of `self.frames()`.
    #[inline]
    pub fn as_mut_slices(&mut self) -> [&mut [T]; CHANNELS] {
        // SAFETY:
        //
        // * The constructors ensure that the pointed-to data vec has a length of at
        // least `frames * CHANNELS`.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * `self` is borrowed as mutable, and none of these slices overlap, so all
        // mutability rules are being upheld.
        // * We have asserted at compile-time that `CHANNELS` is non-zero.
        unsafe {
            core::array::from_fn(|ch_i| {
                core::slice::from_raw_parts_mut(*self.offsets.get_unchecked(ch_i), self.frames)
            })
        }
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
        // * The constructors ensure that the pointed-to data vec has a length of at
        // least `frames * CHANNELS`.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * We have constrained `frames` above.
        // * We have asserted at compile-time that `CHANNELS` is non-zero.
        unsafe {
            core::array::from_fn(|ch_i| {
                core::slice::from_raw_parts(*self.offsets.get_unchecked(ch_i), frames)
            })
        }
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
        // * The constructors ensure that the pointed-to data vec has a length of at
        // least `frames * CHANNELS`.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * We have constrained `frames` above.
        // * `self` is borrowed as mutable, and none of these slices overlap, so all
        // mutability rules are being upheld.
        // * We have asserted at compile-time that `CHANNELS` is non-zero.
        unsafe {
            core::array::from_fn(|ch_i| {
                core::slice::from_raw_parts_mut(*self.offsets.get_unchecked(ch_i), frames)
            })
        }
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
        // * The constructors ensure that the pointed-to data vec has a length of at
        // least `frames * CHANNELS`.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * We have constrained the given range above.
        // * We have asserted at compile-time that `CHANNELS` is non-zero.
        unsafe {
            core::array::from_fn(|ch_i| {
                core::slice::from_raw_parts(
                    self.offsets.get_unchecked(ch_i).add(start_frame),
                    frames,
                )
            })
        }
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
        // * The constructors ensure that the pointed-to data vec has a length of at
        // least `frames * CHANNELS`.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * We have constrained the given range above.
        // * `self` is borrowed as mutable, and none of these slices overlap, so all
        // mutability rules are being upheld.
        // * We have asserted at compile-time that `CHANNELS` is non-zero.
        unsafe {
            core::array::from_fn(|ch_i| {
                core::slice::from_raw_parts_mut(
                    self.offsets.get_unchecked(ch_i).add(start_frame),
                    frames,
                )
            })
        }
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

    #[inline(always)]
    pub fn as_ref<'a>(&'a self) -> ChannelBufferRef<'a, T, CHANNELS> {
        // SAFETY:
        // * The constructors have the same invariants as `ChannelBufferRef`.
        // * `[*const T; CHANNELS]` and `[*mut T; CHANNELS]` are interchangeable bit-for-bit.
        unsafe {
            ChannelBufferRef::from_raw(
                &self.data,
                core::mem::transmute_copy(&self.offsets),
                self.frames,
            )
        }
    }

    #[inline(always)]
    pub fn as_mut<'a>(&'a mut self) -> ChannelBufferRefMut<'a, T, CHANNELS> {
        // SAFETY: The constructors have the same invariants as `ChannelBufferRefMut`.
        unsafe { ChannelBufferRefMut::from_raw(&mut self.data, self.offsets, self.frames) }
    }
}

impl<T: Clone + Copy + Default + Sized + Unpin, const CHANNELS: usize> Index<usize>
    for ChannelBuffer<T, CHANNELS>
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        self.channel(index).unwrap()
    }
}

impl<T: Clone + Copy + Default + Sized + Unpin, const CHANNELS: usize> IndexMut<usize>
    for ChannelBuffer<T, CHANNELS>
{
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.channel_mut(index).unwrap()
    }
}

impl<T: Clone + Copy + Default + Sized + Unpin, const CHANNELS: usize> Default
    for ChannelBuffer<T, CHANNELS>
{
    fn default() -> Self {
        Self::empty()
    }
}

impl<'a, T: Clone + Copy + Default + Sized + Unpin, const CHANNELS: usize>
    Into<ChannelBufferRef<'a, T, CHANNELS>> for &'a ChannelBuffer<T, CHANNELS>
{
    #[inline(always)]
    fn into(self) -> ChannelBufferRef<'a, T, CHANNELS> {
        self.as_ref()
    }
}

impl<'a, T: Clone + Copy + Default + Sized + Unpin, const CHANNELS: usize>
    Into<ChannelBufferRefMut<'a, T, CHANNELS>> for &'a mut ChannelBuffer<T, CHANNELS>
{
    #[inline(always)]
    fn into(self) -> ChannelBufferRefMut<'a, T, CHANNELS> {
        self.as_mut()
    }
}

impl<T: Clone + Copy + Default + Sized + Unpin, const CHANNELS: usize> Into<Vec<T>>
    for ChannelBuffer<T, CHANNELS>
{
    fn into(self) -> Vec<T> {
        Pin::<Vec<T>>::into_inner(self.data)
    }
}

impl<T: Clone + Copy + Default + Sized + Unpin, const CHANNELS: usize> Clone
    for ChannelBuffer<T, CHANNELS>
{
    fn clone(&self) -> Self {
        // SAFETY: We initialize all the data below.
        let mut new_self = unsafe { Self::new_uninit(self.frames) };

        new_self.raw_mut().copy_from_slice(self.raw());

        new_self
    }
}

// # SAFETY: All the stored pointers are valid for the lifetime of the struct, and
// the public API prevents misuse of the pointers.
unsafe impl<T: Clone + Copy + Default + Sized + Unpin, const CHANNELS: usize> Send
    for ChannelBuffer<T, CHANNELS>
{
}
// # SAFETY: All the stored pointers are valid for the lifetime of the struct, and
// the public API prevents misuse of the pointers.
unsafe impl<T: Clone + Copy + Default + Sized + Unpin, const CHANNELS: usize> Sync
    for ChannelBuffer<T, CHANNELS>
{
}
