use core::num::NonZeroUsize;
use core::ops::{Index, IndexMut, Range};

use arrayvec::ArrayVec;

/// An immutable memory-efficient buffer of samples with a fixed runtime number of
/// channels each with a fixed runtime number of frames (samples in a single channel
/// of audio).
///
/// This version uses a reference to a slice as its data source.
#[derive(Debug, Clone)]
pub struct VarChannelBufferRef<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize> {
    data: &'a [T],
    offsets: ArrayVec<*const T, MAX_CHANNELS>,
    frames: usize,
}

impl<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize>
    VarChannelBufferRef<'a, T, MAX_CHANNELS>
{
    #[inline(always)]
    pub(crate) unsafe fn from_raw(
        data: &'a [T],
        offsets: ArrayVec<*const T, MAX_CHANNELS>,
        frames: usize,
    ) -> Self {
        Self {
            data,
            offsets,
            frames,
        }
    }

    /// Create an empty [`VarChannelBufferRef`] with no data.
    pub fn empty() -> Self {
        let data = &[];
        let mut offsets = ArrayVec::new();
        offsets.push(data.as_ptr());

        Self {
            data,
            offsets,
            frames: 0,
        }
    }

    /// Create a new [`VarChannelBufferRef`] using the given slice as the data.
    ///
    /// # Panics
    /// Panics if `channels.get() > MAX_CHANNELS`.
    pub fn new(data: &'a [T], channels: NonZeroUsize) -> Self {
        assert!(channels.get() <= MAX_CHANNELS);

        let frames = data.len() / channels.get();

        let mut offsets = ArrayVec::new();
        // SAFETY:
        // * All of these pointers point to valid memory in the slice.
        // * We have constrained `channels` above.
        unsafe {
            for ch_i in 0..channels.get() {
                offsets.push_unchecked(data.as_ptr().add(ch_i * frames));
            }
        }

        Self {
            data,
            offsets,
            frames,
        }
    }

    /// Create a new [`VarChannelBufferRef`] using the given slice as the data.
    ///
    /// # Safety
    /// The caller must uphold that:
    /// * `data.len() >= frames * channels.get()`
    /// * and `channels.get() <= MAX_CHANNELS`
    pub unsafe fn new_unchecked(data: &'a [T], frames: usize, channels: NonZeroUsize) -> Self {
        let mut offsets = ArrayVec::new();
        for ch_i in 0..channels.get() {
            offsets.push_unchecked(data.as_ptr().add(ch_i * frames));
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
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The caller upholds that `index` is within bounds.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
        core::slice::from_raw_parts(*self.offsets.get_unchecked(index), self.frames)
    }

    /// Get all channels as immutable slices. Each slice will have a length of `self.frames()`.
    #[inline]
    pub fn as_slices(&self) -> ArrayVec<&[T], MAX_CHANNELS> {
        let mut v = ArrayVec::new();

        // SAFETY:
        //
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
        unsafe {
            for ptr in self.offsets.iter() {
                v.push_unchecked(core::slice::from_raw_parts(*ptr, self.frames));
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
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
        // * We have constrained `frames` above.
        unsafe {
            for ptr in self.offsets.iter() {
                v.push_unchecked(core::slice::from_raw_parts(*ptr, frames));
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
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
        // * We have constrained the given range above.
        unsafe {
            for ptr in self.offsets.iter() {
                v.push_unchecked(core::slice::from_raw_parts(ptr.add(start_frame), frames));
            }
        }

        v
    }

    /// Get the entire contents of the buffer as a single immutable slice.
    pub fn raw(&self) -> &[T] {
        self.data
    }
}

impl<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize> Index<usize>
    for VarChannelBufferRef<'a, T, MAX_CHANNELS>
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        self.channel(index).unwrap()
    }
}

impl<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize> Default
    for VarChannelBufferRef<'a, T, MAX_CHANNELS>
{
    fn default() -> Self {
        Self::empty()
    }
}

impl<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize> Into<&'a [T]>
    for VarChannelBufferRefMut<'a, T, MAX_CHANNELS>
{
    fn into(self) -> &'a [T] {
        self.data
    }
}

// # SAFETY: All the stored pointers are valid for the lifetime of the struct, and
// the public API prevents misuse of the pointers.
unsafe impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> Send
    for VarChannelBufferRef<'a, T, CHANNELS>
{
}
// # SAFETY: All the stored pointers are valid for the lifetime of the struct, and
// the public API prevents misuse of the pointers.
unsafe impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> Sync
    for VarChannelBufferRef<'a, T, CHANNELS>
{
}

/// A mutable memory-efficient buffer of samples with a fixed runtime number of
/// channels each with a fixed runtime number of frames (samples in a single channel
/// of audio).
///
/// This version uses a reference to a slice as its data source.
#[derive(Debug)]
pub struct VarChannelBufferRefMut<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize> {
    data: &'a mut [T],
    offsets: ArrayVec<*mut T, MAX_CHANNELS>,
    frames: usize,
}

impl<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize>
    VarChannelBufferRefMut<'a, T, MAX_CHANNELS>
{
    #[inline(always)]
    pub(crate) unsafe fn from_raw(
        data: &'a mut [T],
        offsets: ArrayVec<*mut T, MAX_CHANNELS>,
        frames: usize,
    ) -> Self {
        Self {
            data,
            offsets,
            frames,
        }
    }

    /// Create an empty [`VarChannelBufferRefMut`] with no data.
    pub fn empty() -> Self {
        let data = &mut [];
        let mut offsets = ArrayVec::new();
        offsets.push(data.as_mut_ptr());

        Self {
            data,
            offsets,
            frames: 0,
        }
    }

    /// Create a new [`VarChannelBufferRefMut`] using the given slice as the data.
    ///
    /// # Panics
    /// Panics if `channels.get() > MAX_CHANNELS`.
    pub fn new(data: &'a mut [T], channels: NonZeroUsize) -> Self {
        assert!(channels.get() <= MAX_CHANNELS);

        let frames = data.len() / channels.get();

        let mut offsets = ArrayVec::new();
        // SAFETY:
        // * All of these pointers point to valid memory in the slice.
        // * We have constrained `channels` above.
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

    /// Create a new [`VarChannelBufferRefMut`] using the given slice as the data.
    ///
    /// # Safety
    /// The caller must uphold that:
    /// * `data.len() >= frames * channels.get()`
    /// * and `channels.get() <= MAX_CHANNELS`
    pub unsafe fn new_unchecked(data: &'a mut [T], frames: usize, channels: NonZeroUsize) -> Self {
        let mut offsets = ArrayVec::new();
        for ch_i in 0..channels.get() {
            offsets.push_unchecked(data.as_mut_ptr().add(ch_i * frames));
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
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The caller upholds that `index` is within bounds.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
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
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The caller upholds that `index` is within bounds.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
        // * `self` is borrowed as mutable, ensuring that no other references to the
        // data slice can exist.
        core::slice::from_raw_parts_mut(*self.offsets.get_unchecked(index), self.frames)
    }

    /// Get all channels as immutable slices. Each slice will have a length of `self.frames()`.
    #[inline]
    pub fn as_slices(&self) -> ArrayVec<&[T], MAX_CHANNELS> {
        let mut v = ArrayVec::new();

        // SAFETY:
        //
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
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
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
        // * `self` is borrowed as mutable, and none of these slices overlap, so all
        // mutability rules are being upheld.
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
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
        // * We have constrained `frames` above.
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
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
        // * We have constrained `frames` above.
        // * `self` is borrowed as mutable, and none of these slices overlap, so all
        // mutability rules are being upheld.
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
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
        // * We have constrained the given range above.
        unsafe {
            for ptr in self.offsets.iter() {
                v.push_unchecked(core::slice::from_raw_parts(ptr.add(start_frame), frames));
            }
        }

        v
    }

    /// Get all channels as immutable slices in the given range.
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
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
        // * We have constrained the given range above.
        // * `self` is borrowed as mutable, and none of these slices overlap, so all
        // mutability rules are being upheld.
        unsafe {
            for ptr in self.offsets.iter_mut() {
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
        self.data
    }

    /// Get the entire contents of the buffer as a single mutable slice.
    pub fn raw_mut(&mut self) -> &mut [T] {
        &mut self.data[..]
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
}

impl<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize> Index<usize>
    for VarChannelBufferRefMut<'a, T, MAX_CHANNELS>
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        self.channel(index).unwrap()
    }
}

impl<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize> IndexMut<usize>
    for VarChannelBufferRefMut<'a, T, MAX_CHANNELS>
{
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.channel_mut(index).unwrap()
    }
}

impl<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize> Default
    for VarChannelBufferRefMut<'a, T, MAX_CHANNELS>
{
    fn default() -> Self {
        Self::empty()
    }
}

impl<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize>
    Into<VarChannelBufferRef<'a, T, MAX_CHANNELS>> for VarChannelBufferRefMut<'a, T, MAX_CHANNELS>
{
    #[inline(always)]
    fn into(self) -> VarChannelBufferRef<'a, T, MAX_CHANNELS> {
        VarChannelBufferRef {
            data: self.data,
            // SAFETY: `ArrayVec<*const T; MAX_CHANNELS>` and `ArrayVec<*mut T; MAX_CHANNELS>`
            // are interchangeable bit-for-bit.
            offsets: unsafe { core::mem::transmute_copy(&self.offsets) },
            frames: self.frames,
        }
    }
}

impl<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize> Into<&'a mut [T]>
    for VarChannelBufferRefMut<'a, T, MAX_CHANNELS>
{
    fn into(self) -> &'a mut [T] {
        self.data
    }
}

// # SAFETY: All the stored pointers are valid for the lifetime of the struct, and
// the public API prevents misuse of the pointers.
unsafe impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> Send
    for VarChannelBufferRefMut<'a, T, CHANNELS>
{
}
// # SAFETY: All the stored pointers are valid for the lifetime of the struct, and
// the public API prevents misuse of the pointers.
unsafe impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> Sync
    for VarChannelBufferRefMut<'a, T, CHANNELS>
{
}
