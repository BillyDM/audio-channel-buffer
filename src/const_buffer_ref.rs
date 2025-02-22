use core::ops::{Index, IndexMut, Range};

/// An immutable memory-efficient buffer of samples with a fixed compile-time number
/// of channels each with a fixed runtime number of frames (samples in a single channel
/// of audio).
///
/// This version uses a reference to a slice as its data source.
#[derive(Debug, Clone, Copy)]
pub struct ChannelBufferRef<'a, T: Clone + Copy + Default, const CHANNELS: usize> {
    data: &'a [T],
    offsets: [*const T; CHANNELS],
    frames: usize,
}

impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> ChannelBufferRef<'a, T, CHANNELS> {
    const _COMPILE_TIME_ASSERTS: () = {
        assert!(CHANNELS > 0);
    };

    #[inline(always)]
    pub(crate) unsafe fn from_raw(
        data: &'a [T],
        offsets: [*const T; CHANNELS],
        frames: usize,
    ) -> Self {
        let _ = Self::_COMPILE_TIME_ASSERTS;

        Self {
            data,
            offsets,
            frames,
        }
    }

    /// Create an empty [`ChannelBufferRef`] with no data.
    pub fn empty() -> Self {
        let _ = Self::_COMPILE_TIME_ASSERTS;

        let data = &[];
        let offsets = core::array::from_fn(|_| data.as_ptr());

        Self {
            data,
            offsets,
            frames: 0,
        }
    }

    /// Create a new [`ChannelBufferRef`] using the given slice as the data.
    pub fn new(data: &'a [T]) -> Self {
        let _ = Self::_COMPILE_TIME_ASSERTS;

        let frames = data.len() / CHANNELS;

        Self {
            data,
            // SAFETY:
            // * All of these pointers point to valid memory in the slice.
            // * We have asserted at compile-time that `CHANNELS` is non-zero.
            offsets: unsafe { core::array::from_fn(|ch_i| data.as_ptr().add(ch_i * frames)) },
            frames,
        }
    }

    /// Create a new [`ChannelBufferRef`] using the given slice as the data.
    ///
    /// # Safety
    /// The caller must uphold that `data.len() >= frames * CHANNELS`.
    pub unsafe fn new_unchecked(data: &'a [T], frames: usize) -> Self {
        let _ = Self::_COMPILE_TIME_ASSERTS;

        Self {
            data,
            // SAFETY:
            // * All of these pointers point to valid memory in the slice.
            // * We have asserted at compile-time that `CHANNELS` is non-zero.
            offsets: core::array::from_fn(|ch_i| data.as_ptr().add(ch_i * frames)),
            frames,
        }
    }

    /// The number of frames (samples in a single channel of audio) that are allocated
    /// in this buffer.
    pub fn frames(&self) -> usize {
        self.frames
    }

    /// The number of channels in this buffer.
    pub fn channels(&self) -> usize {
        CHANNELS
    }

    #[inline(always)]
    /// Get an immutable reference to the channel at `index`. The slice will have a length
    /// of `self.frames()`.
    ///
    /// Returns `None` if `index` is out of bounds.
    pub fn channel(&self, index: usize) -> Option<&[T]> {
        if index < CHANNELS {
            // SAFETY: We haved checked that `index` is within bounds.
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
        // * We have asserted at compile-time that `CHANNELS` is non-zero.
        core::slice::from_raw_parts(*self.offsets.get_unchecked(index), self.frames)
    }

    /// Get all channels as immutable slices. Each slice will have a length of `self.frames()`.
    #[inline]
    pub fn as_slices(&self) -> [&[T]; CHANNELS] {
        // SAFETY:
        //
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
        // * We have asserted at compile-time that `CHANNELS` is non-zero.
        unsafe {
            core::array::from_fn(|ch_i| {
                core::slice::from_raw_parts(*self.offsets.get_unchecked(ch_i), self.frames)
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
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
        // * We have constrained `frames` above.
        // * We have asserted at compile-time that `CHANNELS` is non-zero.
        unsafe {
            core::array::from_fn(|ch_i| {
                core::slice::from_raw_parts(*self.offsets.get_unchecked(ch_i), frames)
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
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
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

    /// Get the entire contents of the buffer as a single immutable slice.
    pub fn raw(&self) -> &[T] {
        self.data
    }
}

impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> Index<usize>
    for ChannelBufferRef<'a, T, CHANNELS>
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        self.channel(index).unwrap()
    }
}

impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> Default
    for ChannelBufferRef<'a, T, CHANNELS>
{
    fn default() -> Self {
        Self::empty()
    }
}

impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> Into<&'a [T]>
    for ChannelBufferRefMut<'a, T, CHANNELS>
{
    fn into(self) -> &'a [T] {
        self.data
    }
}

// # SAFETY: All the stored pointers are valid for the lifetime of the struct, and
// the public API prevents misuse of the pointers.
unsafe impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> Send
    for ChannelBufferRef<'a, T, CHANNELS>
{
}
// # SAFETY: All the stored pointers are valid for the lifetime of the struct, and
// the public API prevents misuse of the pointers.
unsafe impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> Sync
    for ChannelBufferRef<'a, T, CHANNELS>
{
}

/// A mutable memory-efficient buffer of samples with a fixed compile-time number of
/// channels each with a fixed runtime number of frames (samples in a single channel
/// of audio).
///
/// This version uses a reference to a slice as its data source.
#[derive(Debug)]
pub struct ChannelBufferRefMut<'a, T: Clone + Copy + Default, const CHANNELS: usize> {
    data: &'a mut [T],
    offsets: [*mut T; CHANNELS],
    frames: usize,
}

impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> ChannelBufferRefMut<'a, T, CHANNELS> {
    const _COMPILE_TIME_ASSERTS: () = {
        assert!(CHANNELS > 0);
    };

    #[inline(always)]
    pub(crate) unsafe fn from_raw(
        data: &'a mut [T],
        offsets: [*mut T; CHANNELS],
        frames: usize,
    ) -> Self {
        let _ = Self::_COMPILE_TIME_ASSERTS;

        Self {
            data,
            offsets,
            frames,
        }
    }

    /// Create an empty [`ChannelBufferRefMut`] with no data.
    pub fn empty() -> Self {
        let _ = Self::_COMPILE_TIME_ASSERTS;

        let data = &mut [];
        let offsets = core::array::from_fn(|_| data.as_mut_ptr());

        Self {
            data,
            offsets,
            frames: 0,
        }
    }

    /// Create a new [`ChannelBufferRefMut`] using the given slice as the data.
    pub fn new(data: &'a mut [T]) -> Self {
        let _ = Self::_COMPILE_TIME_ASSERTS;

        let frames = data.len() / CHANNELS;

        // SAFETY:
        // * All of these pointers point to valid memory in the slice.
        // * We have asserted at compile-time that `CHANNELS` is non-zero.
        let offsets = unsafe { core::array::from_fn(|ch_i| data.as_mut_ptr().add(ch_i * frames)) };

        Self {
            data,
            offsets,
            frames,
        }
    }

    /// Create a new [`ChannelBufferRefMut`] using the given slice as the data.
    ///
    /// # Safety
    /// The caller must uphold that `data.len() >= frames * CHANNELS`.
    pub unsafe fn new_unchecked(data: &'a mut [T], frames: usize) -> Self {
        let _ = Self::_COMPILE_TIME_ASSERTS;

        // SAFETY:
        // * All of these pointers point to valid memory in the slice.
        // * We have asserted at compile-time that `CHANNELS` is non-zero.
        let offsets = core::array::from_fn(|ch_i| data.as_mut_ptr().add(ch_i * frames));

        Self {
            data,
            offsets,
            frames,
        }
    }

    /// The number of frames (samples in a single channel of audio) that are allocated
    /// in this buffer.
    pub fn frames(&self) -> usize {
        self.frames
    }

    /// The number of channels in this buffer.
    pub fn channels(&self) -> usize {
        CHANNELS
    }

    #[inline(always)]
    /// Get an immutable reference to the channel at `index`. The slice will have a length
    /// of `self.frames()`.
    ///
    /// Returns `None` if `index` is out of bounds.
    pub fn channel(&self, index: usize) -> Option<&[T]> {
        if index < CHANNELS {
            // SAFETY: We haved checked that `index` is within bounds.
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
            // SAFETY: We haved checked that `index` is within bounds.
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
        // * We have asserted at compile-time that `CHANNELS` is non-zero.
        core::slice::from_raw_parts_mut(*self.offsets.get_unchecked(index), self.frames)
    }

    /// Get all channels as immutable slices. Each slice will have a length of `self.frames()`.
    #[inline]
    pub fn as_slices(&self) -> [&[T]; CHANNELS] {
        // SAFETY:
        //
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
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
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
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
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
        // * We have constrained `frames` above.
        // * We have asserted at compile-time that `CHANNELS` is non-zero.
        unsafe {
            core::array::from_fn(|ch_i| {
                core::slice::from_raw_parts(*self.offsets.get_unchecked(ch_i), frames)
            })
        }
    }

    /// Get all channels as immutable slices with the given length in frames.
    ///
    /// If `frames > self.frames()`, then each slice will have a length of `self.frames()`
    /// instead.
    #[inline]
    pub fn as_mut_slices_with_length(&mut self, frames: usize) -> [&mut [T]; CHANNELS] {
        let frames = frames.min(self.frames);

        // SAFETY:
        //
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
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
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
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

    /// Get all channels as immutable slices in the given range.
    ///
    /// If all or part of the range falls out of bounds, then only the part that falls
    /// within range will be returned.
    #[inline]
    pub fn as_mut_slices_with_range(&mut self, range: Range<usize>) -> [&mut [T]; CHANNELS] {
        let start_frame = range.start.min(self.frames);
        let frames = range.end.min(self.frames) - start_frame;

        // SAFETY:
        //
        // * The constructors ensure that the pointed-to data slice has a length of at
        // least `frames * CHANNELS`.
        // * The data slice cannot be moved, so the pointers are valid for the lifetime
        // of the slice.
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

impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> Index<usize>
    for ChannelBufferRefMut<'a, T, CHANNELS>
{
    type Output = [T];

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        self.channel(index).unwrap()
    }
}

impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> IndexMut<usize>
    for ChannelBufferRefMut<'a, T, CHANNELS>
{
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.channel_mut(index).unwrap()
    }
}

impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> Default
    for ChannelBufferRefMut<'a, T, CHANNELS>
{
    fn default() -> Self {
        Self::empty()
    }
}

impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> Into<ChannelBufferRef<'a, T, CHANNELS>>
    for ChannelBufferRefMut<'a, T, CHANNELS>
{
    #[inline(always)]
    fn into(self) -> ChannelBufferRef<'a, T, CHANNELS> {
        ChannelBufferRef {
            data: self.data,
            // SAFETY: `[*const T; CHANNELS]` and `[*mut T; CHANNELS]` are interchangeable bit-for-bit.
            offsets: unsafe { core::mem::transmute_copy(&self.offsets) },
            frames: self.frames,
        }
    }
}

impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> Into<&'a mut [T]>
    for ChannelBufferRefMut<'a, T, CHANNELS>
{
    fn into(self) -> &'a mut [T] {
        self.data
    }
}

// # SAFETY: All the stored pointers are valid for the lifetime of the struct, and
// the public API prevents misuse of the pointers.
unsafe impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> Send
    for ChannelBufferRefMut<'a, T, CHANNELS>
{
}
// # SAFETY: All the stored pointers are valid for the lifetime of the struct, and
// the public API prevents misuse of the pointers.
unsafe impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> Sync
    for ChannelBufferRefMut<'a, T, CHANNELS>
{
}
