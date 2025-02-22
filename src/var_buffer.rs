use alloc::vec::Vec;
use core::num::NonZeroUsize;
use core::ops::{Index, IndexMut, Range};
use core::pin::Pin;

use arrayvec::ArrayVec;

use crate::{VarChannelBufferRef, VarChannelBufferRefMut};

/// A memory-efficient buffer of samples with a fixed runtime number of channels each
/// with a fixed runtime number of frames (samples in a single channel of audio).
///
/// This version uses an owned `Vec` as its data source.
#[derive(Debug)]
pub struct VarChannelBuffer<T: Clone + Copy + Default + Sized + Unpin, const MAX_CHANNELS: usize> {
    data: Pin<Vec<T>>,
    offsets: ArrayVec<*mut T, MAX_CHANNELS>,
    frames: usize,
}

impl<T: Clone + Copy + Default + Sized + Unpin, const MAX_CHANNELS: usize>
    VarChannelBuffer<T, MAX_CHANNELS>
{
    const _COMPILE_TIME_ASSERTS: () = {
        assert!(MAX_CHANNELS > 0);
    };

    /// Create an empty [`VarChannelBuffer`] with no allocated capacity.
    pub fn empty() -> Self {
        let _ = Self::_COMPILE_TIME_ASSERTS;

        let mut data = Pin::new(Vec::<T>::new());

        let mut offsets = ArrayVec::new();
        offsets.push(data.as_mut_ptr());

        Self {
            data,
            offsets,
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
        let _ = Self::_COMPILE_TIME_ASSERTS;

        assert!(channels.get() <= MAX_CHANNELS);

        let buffer_len = channels.get() * frames;

        let mut data = Vec::<T>::new();
        data.reserve_exact(buffer_len);
        data.resize(buffer_len, Default::default());

        let mut data = Pin::new(data);

        let mut offsets = ArrayVec::new();
        // SAFETY:
        // * All of these pointers point to valid memory in the slice.
        // * We have constrained `channels` above.
        // * We have asserted at compile-time that `MAX_CHANNELS` is non-zero.
        unsafe {
            for ch_i in 0..channels.get() {
                offsets.push_unchecked(data.as_mut_ptr().add(ch_i * frames));
            }
        }

        Self {
            data,
            offsets,
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
        let _ = Self::_COMPILE_TIME_ASSERTS;

        assert!(channels.get() <= MAX_CHANNELS);

        let buffer_len = channels.get() * frames;

        let mut data = Vec::<T>::new();
        data.reserve_exact(buffer_len);
        data.set_len(buffer_len);

        let mut data = Pin::new(data);

        let mut offsets = ArrayVec::new();
        // SAFETY:
        // * All of these pointers point to valid memory in the slice.
        // * We have constrained `channels` above.
        // * We have asserted at compile-time that `MAX_CHANNELS` is non-zero.
        unsafe {
            for ch_i in 0..channels.get() {
                offsets.push_unchecked(data.as_mut_ptr().add(ch_i * frames));
            }
        }

        Self {
            data,
            offsets,
            frames,
        }
    }

    /// The number of channels in this buffer.
    pub fn channels(&self) -> NonZeroUsize {
        // SAFETY: The constructors ensure that there is at least one element in `offsets`.
        unsafe { NonZeroUsize::new_unchecked(self.offsets.len()) }
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
        if index < self.offsets.len() {
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
        // least `frames * self.channels()`.
        // * The caller upholds that `index` is within bounds.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * We have asserted at compile-time that `MAX_CHANNELS` is non-zero.
        core::slice::from_raw_parts(*self.offsets.get_unchecked(index), self.frames)
    }

    #[inline(always)]
    /// Get a mutable reference to the channel at `index`. The slice will have a length
    /// of `self.frames()`.
    ///
    /// Returns `None` if `index` is out of bounds.
    pub fn channel_mut(&mut self, index: usize) -> Option<&mut [T]> {
        if index < self.offsets.len() {
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
        // least `frames * self.channels()`.
        // * The caller upholds that `index` is within bounds.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * `self` is borrowed as mutable, ensuring that no other references to the
        // data Vec can exist.
        // * We have asserted at compile-time that `MAX_CHANNELS` is non-zero.
        core::slice::from_raw_parts_mut(*self.offsets.get_unchecked(index), self.frames)
    }

    /// Get all channels as immutable slices. Each slice will have a length of `self.frames()`.
    #[inline]
    pub fn as_slices(&self) -> ArrayVec<&[T], MAX_CHANNELS> {
        let mut v = ArrayVec::new();

        // SAFETY:
        //
        // * The constructors ensure that the pointed-to data vec has a length of at
        // least `frames * self.channels()`.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * We have asserted at compile-time that `MAX_CHANNELS` is non-zero.
        unsafe {
            for ptr in self.offsets.iter() {
                v.push_unchecked(core::slice::from_raw_parts(*ptr, self.frames));
            }
        }

        v
    }

    /// Get all channels as mutable slices. Each slice will have a length of `self.frames()`.
    #[inline]
    pub fn as_mut_slices(&mut self) -> ArrayVec<&mut [T], MAX_CHANNELS> {
        let mut v = ArrayVec::new();

        // SAFETY:
        //
        // * The constructors ensure that the pointed-to data vec has a length of at
        // least `frames * self.channels()`.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * `self` is borrowed as mutable, and none of these slices overlap, so all
        // mutability rules are being upheld.
        // * We have asserted at compile-time that `MAX_CHANNELS` is non-zero.
        unsafe {
            for ptr in self.offsets.iter() {
                v.push_unchecked(core::slice::from_raw_parts_mut(*ptr, self.frames));
            }
        }

        v
    }

    /// Get all channels as immutable slices with the given length in frames.
    ///
    /// If `frames > self.frames()`, then each slice will have a length of `self.frames()`
    /// instead.
    #[inline]
    pub fn as_slices_with_length(&self, frames: usize) -> ArrayVec<&[T], MAX_CHANNELS> {
        let frames = frames.min(self.frames);
        let mut v = ArrayVec::new();

        // SAFETY:
        //
        // * The constructors ensure that the pointed-to data vec has a length of at
        // least `frames * self.channels()`.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * We have constrained `frames` above.
        // * We have asserted at compile-time that `MAX_CHANNELS` is non-zero.
        unsafe {
            for ptr in self.offsets.iter() {
                v.push_unchecked(core::slice::from_raw_parts(*ptr, frames));
            }
        }

        v
    }

    /// Get all channels as mutable slices with the given length in frames.
    ///
    /// If `frames > self.frames()`, then each slice will have a length of `self.frames()`
    /// instead.
    #[inline]
    pub fn as_mut_slices_with_length(&mut self, frames: usize) -> ArrayVec<&mut [T], MAX_CHANNELS> {
        let frames = frames.min(self.frames);
        let mut v = ArrayVec::new();

        // SAFETY:
        //
        // * The constructors ensure that the pointed-to data vec has a length of at
        // least `frames * self.channels()`.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * We have constrained `frames` above.
        // * `self` is borrowed as mutable, and none of these slices overlap, so all
        // mutability rules are being upheld.
        // * We have asserted at compile-time that `MAX_CHANNELS` is non-zero.
        unsafe {
            for ptr in self.offsets.iter() {
                v.push_unchecked(core::slice::from_raw_parts_mut(*ptr, frames));
            }
        }

        v
    }

    /// Get all channels as immutable slices in the given range.
    ///
    /// If all or part of the range falls out of bounds, then only the part that falls
    /// within range will be returned.
    #[inline]
    pub fn as_slices_with_range(&self, range: Range<usize>) -> ArrayVec<&[T], MAX_CHANNELS> {
        let start_frame = range.start.min(self.frames);
        let frames = range.end.min(self.frames) - start_frame;

        let mut v = ArrayVec::new();

        // SAFETY:
        //
        // * The constructors ensure that the pointed-to data vec has a length of at
        // least `frames * self.channels()`.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * We have constrained the given range above.
        // * We have asserted at compile-time that `MAX_CHANNELS` is non-zero.
        unsafe {
            for ptr in self.offsets.iter() {
                v.push_unchecked(core::slice::from_raw_parts(ptr.add(start_frame), frames));
            }
        }

        v
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

        let mut v = ArrayVec::new();

        // SAFETY:
        //
        // * The constructors ensure that the pointed-to data vec has a length of at
        // least `frames * self.channels()`.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * We have constrained the given range above.
        // * `self` is borrowed as mutable, and none of these slices overlap, so all
        // mutability rules are being upheld.
        // * We have asserted at compile-time that `MAX_CHANNELS` is non-zero.
        unsafe {
            for ptr in self.offsets.iter() {
                v.push_unchecked(core::slice::from_raw_parts_mut(
                    ptr.add(start_frame),
                    frames,
                ));
            }
        }

        v
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
    pub fn as_ref<'a>(&'a self) -> VarChannelBufferRef<'a, T, MAX_CHANNELS> {
        // SAFETY:
        // * The constructors have the same invariants as `VarChannelBufferRef`.
        // * `ArrayVec<*const T; MAX_CHANNELS>` and `ArrayVec<*mut T; MAX_CHANNELS>`
        // are interchangeable bit-for-bit.
        unsafe {
            VarChannelBufferRef::from_raw(
                &self.data,
                core::mem::transmute_copy(&self.offsets),
                self.frames,
            )
        }
    }

    #[inline(always)]
    pub fn as_mut<'a>(&'a mut self) -> VarChannelBufferRefMut<'a, T, MAX_CHANNELS> {
        // SAFETY: The constructors have the same invariants as `VarChannelBufferRefMut`.
        unsafe {
            VarChannelBufferRefMut::from_raw(&mut self.data, self.offsets.clone(), self.frames)
        }
    }
}

impl<T: Clone + Copy + Default + Sized + Unpin, const MAX_CHANNELS: usize> Index<usize>
    for VarChannelBuffer<T, MAX_CHANNELS>
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        self.channel(index).unwrap()
    }
}

impl<T: Clone + Copy + Default + Sized + Unpin, const MAX_CHANNELS: usize> IndexMut<usize>
    for VarChannelBuffer<T, MAX_CHANNELS>
{
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.channel_mut(index).unwrap()
    }
}

impl<T: Clone + Copy + Default + Sized + Unpin, const MAX_CHANNELS: usize> Default
    for VarChannelBuffer<T, MAX_CHANNELS>
{
    fn default() -> Self {
        Self::empty()
    }
}

impl<'a, T: Clone + Copy + Default + Sized + Unpin, const MAX_CHANNELS: usize>
    Into<VarChannelBufferRef<'a, T, MAX_CHANNELS>> for &'a VarChannelBuffer<T, MAX_CHANNELS>
{
    #[inline(always)]
    fn into(self) -> VarChannelBufferRef<'a, T, MAX_CHANNELS> {
        self.as_ref()
    }
}

impl<'a, T: Clone + Copy + Default + Sized + Unpin, const MAX_CHANNELS: usize>
    Into<VarChannelBufferRefMut<'a, T, MAX_CHANNELS>>
    for &'a mut VarChannelBuffer<T, MAX_CHANNELS>
{
    #[inline(always)]
    fn into(self) -> VarChannelBufferRefMut<'a, T, MAX_CHANNELS> {
        self.as_mut()
    }
}

impl<T: Clone + Copy + Default + Sized + Unpin, const MAX_CHANNELS: usize> Into<Vec<T>>
    for VarChannelBuffer<T, MAX_CHANNELS>
{
    fn into(self) -> Vec<T> {
        Pin::<Vec<T>>::into_inner(self.data)
    }
}

impl<T: Clone + Copy + Default + Sized + Unpin, const CHANNELS: usize> Clone
    for VarChannelBuffer<T, CHANNELS>
{
    fn clone(&self) -> Self {
        // SAFETY: We initialize all the data below.
        let mut new_self = unsafe { Self::new_uninit(self.channels(), self.frames) };

        new_self.raw_mut().copy_from_slice(self.raw());

        new_self
    }
}

// # SAFETY: All the stored pointers are valid for the lifetime of the struct, and
// the public API prevents misuse of the pointers.
unsafe impl<T: Clone + Copy + Default + Sized + Unpin, const CHANNELS: usize> Send
    for VarChannelBuffer<T, CHANNELS>
{
}
// # SAFETY: All the stored pointers are valid for the lifetime of the struct, and
// the public API prevents misuse of the pointers.
unsafe impl<T: Clone + Copy + Default + Sized + Unpin, const CHANNELS: usize> Sync
    for VarChannelBuffer<T, CHANNELS>
{
}
