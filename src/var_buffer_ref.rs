use core::num::NonZeroUsize;
use core::ops::{Index, IndexMut, Range};

use arrayvec::ArrayVec;

/// An immutable memory-efficient buffer of samples with a fixed runtime number of
/// channels each with a fixed runtime number of frames (samples in a single channel
/// of audio).
#[derive(Debug, Clone, Copy)]
pub struct VarChannelBufferRef<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize> {
    pub(crate) data: &'a [T],
    pub(crate) channels: NonZeroUsize,
    pub(crate) frames: usize,
}

impl<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize>
    VarChannelBufferRef<'a, T, MAX_CHANNELS>
{
    /// Create an empty [`VarChannelBufferRef`] with no data.
    pub const fn empty() -> Self {
        Self {
            data: &[],
            channels: NonZeroUsize::MIN,
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

        Self {
            data,
            channels,
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
        Self {
            data,
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
            channel_unchecked::<T, MAX_CHANNELS>(self.data, self.frames, index, 0, self.frames)
        }
    }

    /// Get all channels as immutable slices. Each slice will have a length of `self.frames()`.
    #[inline]
    pub fn as_slices(&self) -> ArrayVec<&[T], MAX_CHANNELS> {
        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // so this is always within range.
        unsafe { as_slices(self.data, self.channels.get(), self.frames, 0, self.frames) }
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
        unsafe { as_slices(self.data, self.channels.get(), self.frames, 0, frames) }
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
                self.data,
                self.channels.get(),
                self.frames,
                start_frame,
                frames,
            )
        }
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

/// A mutable memory-efficient buffer of samples with a fixed runtime number of
/// channels each with a fixed runtime number of frames (samples in a single channel
/// of audio).
#[derive(Debug)]
pub struct VarChannelBufferRefMut<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize> {
    pub(crate) data: &'a mut [T],
    pub(crate) channels: NonZeroUsize,
    pub(crate) frames: usize,
}

impl<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize>
    VarChannelBufferRefMut<'a, T, MAX_CHANNELS>
{
    /// Create an empty [`VarChannelBufferRefMut`] with no data.
    pub const fn empty() -> Self {
        Self {
            data: &mut [],
            channels: NonZeroUsize::MIN,
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

        Self {
            data,
            channels,
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
        Self {
            data,
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
            channel_unchecked::<T, MAX_CHANNELS>(self.data, self.frames, index, 0, self.frames)
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
            channel_unchecked_mut::<T, MAX_CHANNELS>(self.data, self.frames, index, 0, self.frames)
        }
    }

    /// Get all channels as immutable slices. Each slice will have a length of `self.frames()`.
    #[inline]
    pub fn as_slices(&self) -> ArrayVec<&[T], MAX_CHANNELS> {
        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // so this is always within range.
        unsafe { as_slices(self.data, self.channels.get(), self.frames, 0, self.frames) }
    }

    /// Get all channels as mutable slices. Each slice will have a length of `self.frames()`.
    #[inline]
    pub fn as_mut_slices(&mut self) -> ArrayVec<&mut [T], MAX_CHANNELS> {
        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // so this is always within range.
        unsafe { as_mut_slices(self.data, self.channels.get(), self.frames, 0, self.frames) }
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
        unsafe { as_slices(self.data, self.channels.get(), self.frames, 0, frames) }
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
        unsafe { as_mut_slices(self.data, self.channels.get(), self.frames, 0, frames) }
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
                self.data,
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
                self.data,
                self.channels.get(),
                self.frames,
                start_frame,
                frames,
            )
        }
    }

    /// Get the entire contents of the buffer as a single immutable slice.
    pub fn raw(&self) -> &[T] {
        self.data
    }

    /// Get the entire contents of the buffer as a single mutable slice.
    pub fn raw_mut(&mut self) -> &mut [T] {
        self.data
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
    fn into(self) -> VarChannelBufferRef<'a, T, MAX_CHANNELS> {
        VarChannelBufferRef {
            data: self.data,
            channels: self.channels,
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

#[inline(always)]
/// # Safety
/// The caller must uphold that:
/// * `data.len() >= frames * MAX_CHANNELS`
/// * `slice_start_frame < frames`
/// * and `slice_start_frame + slice_frames <= frames`
pub unsafe fn channel_unchecked<T, const MAX_CHANNELS: usize>(
    data: &[T],
    frames: usize,
    index: usize,
    slice_start_frame: usize,
    slice_frames: usize,
) -> &[T] {
    core::slice::from_raw_parts(
        data.as_ptr().add((index * frames) + slice_start_frame),
        slice_frames,
    )
}

#[inline(always)]
/// # Safety
/// The caller must uphold that:
/// * `data.len() >= frames * MAX_CHANNELS`
/// * `slice_start_frame < frames`
/// * and `slice_start_frame + slice_frames <= frames`
pub unsafe fn channel_unchecked_mut<T, const MAX_CHANNELS: usize>(
    data: &mut [T],
    frames: usize,
    index: usize,
    slice_start_frame: usize,
    slice_frames: usize,
) -> &mut [T] {
    // SAFETY:
    // `data` is borrowed mutably in this method, so all mutability rules
    // are being upheld.
    core::slice::from_raw_parts_mut(
        data.as_mut_ptr().add((index * frames) + slice_start_frame),
        slice_frames,
    )
}

#[inline]
/// # Safety
/// The caller must uphold that:
/// * `data.len() >= frames * MAX_CHANNELS`
/// * `channels <= MAX_CHANNELS`
/// * `slice_start_frame < frames`
/// * and `slice_start_frame + slice_frames <= frames`
pub(crate) unsafe fn as_slices<T, const MAX_CHANNELS: usize>(
    data: &[T],
    channels: usize,
    frames: usize,
    slice_start_frame: usize,
    slice_frames: usize,
) -> ArrayVec<&[T], MAX_CHANNELS> {
    let mut v = ArrayVec::new();

    for ch_i in 0..channels {
        v.push_unchecked(core::slice::from_raw_parts(
            data.as_ptr().add((ch_i * frames) + slice_start_frame),
            slice_frames,
        ));
    }

    v
}

#[inline]
/// # Safety
/// The caller must uphold that:
/// * `data.len() >= frames * MAX_CHANNELS`
/// * `channels <= MAX_CHANNELS`
/// * `slice_start_frame < frames`
/// * and `slice_start_frame + slice_frames <= frames`
pub(crate) unsafe fn as_mut_slices<T, const MAX_CHANNELS: usize>(
    data: &mut [T],
    channels: usize,
    frames: usize,
    slice_start_frame: usize,
    slice_frames: usize,
) -> ArrayVec<&mut [T], MAX_CHANNELS> {
    let mut v = ArrayVec::new();

    // SAFETY:
    // None of these slices overlap, and `data` is borrowed mutably in this method,
    // so all mutability rules are being upheld.
    for ch_i in 0..channels {
        v.push_unchecked(core::slice::from_raw_parts_mut(
            data.as_mut_ptr().add((ch_i * frames) + slice_start_frame),
            slice_frames,
        ));
    }

    v
}
