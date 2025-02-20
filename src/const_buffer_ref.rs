use core::ops::{Index, IndexMut, Range};

/// An immutable memory-efficient buffer of samples with a fixed compile-time number
/// of channels each with a fixed runtime number of frames (samples in a single channel
/// of audio).
#[derive(Debug, Clone, Copy)]
pub struct ChannelBufferRef<'a, T: Clone + Copy + Default, const CHANNELS: usize> {
    pub(crate) data: &'a [T],
    pub(crate) frames: usize,
}

impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> ChannelBufferRef<'a, T, CHANNELS> {
    /// Create an empty [`ChannelBufferRef`] with no data.
    pub const fn empty() -> Self {
        Self {
            data: &[],
            frames: 0,
        }
    }

    /// Create a new [`ChannelBufferRef`] using the given slice as the data.
    pub fn new(data: &'a [T]) -> Self {
        let frames = data.len() / CHANNELS;

        Self { data, frames }
    }

    /// Create a new [`VarChannelBufferRef`] using the given slice as the data.
    ///
    /// # Safety
    /// The caller must uphold that `data.len() >= frames * CHANNELS`.
    pub unsafe fn new_unchecked(data: &'a [T], frames: usize) -> Self {
        Self { data, frames }
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
        unsafe { channel_unchecked::<T, CHANNELS>(self.data, self.frames, index, 0, self.frames) }
    }

    /// Get all channels as immutable slices. Each slice will have a length of `self.frames()`.
    #[inline]
    pub fn as_slices(&self) -> [&[T]; CHANNELS] {
        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // so this is always within range.
        unsafe { as_slices(self.data, self.frames, 0, self.frames) }
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
        unsafe { as_slices(self.data, self.frames, 0, frames) }
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
        unsafe { as_slices(self.data, self.frames, start_frame, frames) }
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

/// A mutable memory-efficient buffer of samples with a fixed compile-time number of
/// channels each with a fixed runtime number of frames (samples in a single channel
/// of audio).
#[derive(Debug)]
pub struct ChannelBufferRefMut<'a, T: Clone + Copy + Default, const CHANNELS: usize> {
    pub(crate) data: &'a mut [T],
    pub(crate) frames: usize,
}

impl<'a, T: Clone + Copy + Default, const CHANNELS: usize> ChannelBufferRefMut<'a, T, CHANNELS> {
    /// Create an empty [`ChannelBufferRefMut`] with no data.
    pub const fn empty() -> Self {
        Self {
            data: &mut [],
            frames: 0,
        }
    }

    /// Create a new [`ChannelBufferRefMut`] using the given slice as the data.
    pub fn new(data: &'a mut [T]) -> Self {
        let frames = data.len() / CHANNELS;

        Self { data, frames }
    }

    /// Create a new [`VarChannelBufferRef`] using the given slice as the data.
    ///
    /// # Safety
    /// The caller must uphold that `data.len() >= frames * CHANNELS`.
    pub unsafe fn new_unchecked(data: &'a mut [T], frames: usize) -> Self {
        Self { data, frames }
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
        unsafe { channel_unchecked::<T, CHANNELS>(self.data, self.frames, index, 0, self.frames) }
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
            channel_unchecked_mut::<T, CHANNELS>(self.data, self.frames, index, 0, self.frames)
        }
    }

    /// Get all channels as immutable slices. Each slice will have a length of `self.frames()`.
    #[inline]
    pub fn as_slices(&self) -> [&[T]; CHANNELS] {
        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // so this is always within range.
        unsafe { as_slices(self.data, self.frames, 0, self.frames) }
    }

    /// Get all channels as mutable slices. Each slice will have a length of `self.frames()`.
    #[inline]
    pub fn as_mut_slices(&mut self) -> [&mut [T]; CHANNELS] {
        // SAFETY:
        //
        // * The constructor has set the size of the buffer to`self.frames * self.channels`,
        // so this is always within range.
        unsafe { as_mut_slices(self.data, self.frames, 0, self.frames) }
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
        unsafe { as_slices(self.data, self.frames, 0, frames) }
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
        unsafe { as_mut_slices(self.data, self.frames, 0, frames) }
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
        unsafe { as_slices(self.data, self.frames, start_frame, frames) }
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
        unsafe { as_mut_slices(self.data, self.frames, start_frame, frames) }
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
    fn into(self) -> ChannelBufferRef<'a, T, CHANNELS> {
        ChannelBufferRef {
            data: self.data,
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

#[inline(always)]
/// # Safety
/// The caller must uphold that:
/// * `data.len() >= frames * CHANNELS`
/// * `slice_start_frame < frames`
/// * and `slice_start_frame + slice_frames <= frames`
pub unsafe fn channel_unchecked<T, const CHANNELS: usize>(
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
/// * `data.len() >= frames * CHANNELS`
/// * `slice_start_frame < frames`
/// * and `slice_start_frame + slice_frames <= frames`
pub unsafe fn channel_unchecked_mut<T, const CHANNELS: usize>(
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
/// * `data.len() >= frames * CHANNELS`
/// * `slice_start_frame < frames`
/// * and `slice_start_frame + slice_frames <= frames`
pub(crate) unsafe fn as_slices<T, const CHANNELS: usize>(
    data: &[T],
    frames: usize,
    slice_start_frame: usize,
    slice_frames: usize,
) -> [&[T]; CHANNELS] {
    core::array::from_fn(|ch_i| {
        core::slice::from_raw_parts(
            data.as_ptr().add((ch_i * frames) + slice_start_frame),
            slice_frames,
        )
    })
}

#[inline]
/// # Safety
/// The caller must uphold that:
/// * `data.len() >= frames * CHANNELS`
/// * `slice_start_frame < frames`
/// * and `slice_start_frame + slice_frames <= frames`
pub(crate) unsafe fn as_mut_slices<T, const CHANNELS: usize>(
    data: &mut [T],
    frames: usize,
    slice_start_frame: usize,
    slice_frames: usize,
) -> [&mut [T]; CHANNELS] {
    // SAFETY:
    // None of these slices overlap, and `data` is borrowed mutably in this method,
    // so all mutability rules are being upheld.
    core::array::from_fn(|ch_i| {
        core::slice::from_raw_parts_mut(
            data.as_mut_ptr().add((ch_i * frames) + slice_start_frame),
            slice_frames,
        )
    })
}
