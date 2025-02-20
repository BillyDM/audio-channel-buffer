use core::num::NonZeroUsize;

use crate::{VarChannelBufferRef, VarChannelBufferRefMut};

/// A memory-efficient buffer of samples with variable number of instances each with up to
/// `MAX_CHANNELS` channels. Each channel has a fixed runtime number of `frames` (samples
/// in a single channel of audio).
#[derive(Debug, Clone)]
pub struct InstanceChannelBuffer<T: Clone + Copy + Default, const MAX_CHANNELS: usize> {
    data: Vec<T>,
    num_instances: usize,
    channels: NonZeroUsize,
    frames: usize,
    instance_length: usize,
}

impl<T: Clone + Copy + Default, const MAX_CHANNELS: usize> InstanceChannelBuffer<T, MAX_CHANNELS> {
    /// Create an empty [`InstanceBuffer`] with no allocated capacity.
    pub const fn empty() -> Self {
        Self {
            data: Vec::new(),
            num_instances: 0,
            channels: NonZeroUsize::MIN,
            frames: 0,
            instance_length: 0,
        }
    }

    /// Create a new [`InstanceChannelBuffer`] allocated with the given number of
    /// `instances`, each with the given number of `channels`, each with a length
    /// of the given number of frames (samples in a single channel of audio).
    ///
    /// All data will be initialized with the default value.
    ///
    /// # Panics
    /// Panics if `channels.get() > MAX_CHANNELS`.
    pub fn new(num_instances: usize, channels: NonZeroUsize, frames: usize) -> Self {
        assert!(channels.get() <= MAX_CHANNELS);

        let instance_length = frames * channels.get();
        let buffer_len = instance_length * num_instances;

        let mut data = Vec::new();
        data.reserve_exact(buffer_len);
        data.resize(buffer_len, Default::default());

        Self {
            data,
            num_instances,
            channels,
            frames,
            instance_length,
        }
    }

    /// Create a new [`InstanceChannelBuffer`] allocated with the given number of
    /// `instances`, each with the given number of `channels`, each with a length
    /// of the given number of frames (samples in a single channel of audio).
    ///
    /// No data will be initialized.
    ///
    /// # Safety
    /// Any data must be initialized before reading.
    ///
    /// # Panics
    /// Panics if `channels.get() > MAX_CHANNELS`.
    pub unsafe fn new_uninit(num_instances: usize, channels: NonZeroUsize, frames: usize) -> Self {
        assert!(channels.get() <= MAX_CHANNELS);

        let instance_length = frames * channels.get();
        let buffer_len = instance_length * num_instances;

        let mut data = Vec::new();
        data.reserve_exact(buffer_len);
        data.set_len(buffer_len);

        Self {
            data,
            num_instances,
            channels,
            frames,
            instance_length,
        }
    }

    /// The number of instances in this buffer.
    pub fn num_instances(&self) -> usize {
        self.num_instances
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

    /// Get an immutable reference to the instance at the given index.
    ///
    /// Returns `None` if `index` is out of bounds.
    #[inline(always)]
    pub fn instance<'a>(
        &'a self,
        index: usize,
    ) -> Option<VarChannelBufferRef<'a, T, MAX_CHANNELS>> {
        if index < self.num_instances {
            // # SAFETY:
            // We have checked that `instance` is within bounds.
            unsafe { Some(self.instance_unchecked(index)) }
        } else {
            None
        }
    }

    /// Get an immutable reference to the instance at the given index.
    ///
    /// # Safety
    /// `index` must be less than `self.num_instances()`.
    #[inline(always)]
    pub unsafe fn instance_unchecked<'a>(
        &'a self,
        index: usize,
    ) -> VarChannelBufferRef<'a, T, MAX_CHANNELS> {
        VarChannelBufferRef {
            // # SAFETY:
            // * The constructors, `set_num_instances`, and `set_num_instances_uninit` ensure
            // that `self.data.len() == self.instance_length * self.num_instances`.
            data: core::slice::from_raw_parts(
                self.data.as_ptr().add(self.instance_length * index),
                self.instance_length,
            ),
            channels: self.channels,
            frames: self.frames,
        }
    }

    /// Get a mutable reference to the instance at the given index.
    ///
    /// Returns `None` if `index` is out of bounds.
    #[inline(always)]
    pub fn instance_mut<'a>(
        &'a mut self,
        index: usize,
    ) -> Option<VarChannelBufferRefMut<'a, T, MAX_CHANNELS>> {
        if index < self.num_instances {
            // # SAFETY:
            // We have checked that `instance` is within bounds.
            unsafe { Some(self.instance_unchecked_mut(index)) }
        } else {
            None
        }
    }

    /// Get a mutable reference to the instance at the given index.
    ///
    /// # Safety
    /// `index` must be less than `self.num_instances()`.
    #[inline(always)]
    pub unsafe fn instance_unchecked_mut<'a>(
        &'a mut self,
        index: usize,
    ) -> VarChannelBufferRefMut<'a, T, MAX_CHANNELS> {
        VarChannelBufferRefMut {
            // # SAFETY:
            // * The constructors, `set_num_instances`, and `set_num_instances_uninit` ensure
            // that `self.data.len() == self.instance_length * self.num_instances`.
            // * `self` is borrowed as mutable, ensuring that all mutability rules are being
            // upheld.
            data: core::slice::from_raw_parts_mut(
                self.data.as_mut_ptr().add(self.instance_length * index),
                self.instance_length,
            ),
            channels: self.channels,
            frames: self.frames,
        }
    }

    /// Set the number of instances.
    ///
    /// This method may allocate and is not realtime-safe.
    pub fn set_num_instances(&mut self, num_instances: usize) {
        if self.num_instances == num_instances {
            return;
        }

        let buffer_len = self.instance_length * num_instances;

        self.data.resize(buffer_len, T::default());

        self.num_instances = num_instances;
    }

    /// Set the number of instances without initialize any new data.
    ///
    /// This method may allocate and is not realtime-safe.
    ///
    /// # Safety
    /// Any data must be initialized before reading.
    pub unsafe fn set_num_instances_uninit(&mut self, num_instances: usize) {
        if self.num_instances == num_instances {
            return;
        }

        let buffer_len = self.instance_length * num_instances;

        if self.data.len() < buffer_len {
            self.data.reserve(buffer_len - self.data.len());
        }

        self.data.set_len(buffer_len);

        self.num_instances = num_instances;
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

    /// Get an immutable iterator over all the instances.
    pub fn iter<'a>(&'a self) -> InstanceChannelIter<'a, T, MAX_CHANNELS> {
        InstanceChannelIter { buf: self, curr: 0 }
    }

    /// Get a mutable iterator over all the instances.
    pub fn iter_mut<'a>(&'a mut self) -> InstanceChannelIterMut<'a, T, MAX_CHANNELS> {
        InstanceChannelIterMut { buf: self, curr: 0 }
    }
}

impl<T: Clone + Copy + Default, const MAX_CHANNELS: usize> Default
    for InstanceChannelBuffer<T, MAX_CHANNELS>
{
    fn default() -> Self {
        Self::empty()
    }
}

impl<T: Clone + Copy + Default, const MAX_CHANNELS: usize> Into<Vec<T>>
    for InstanceChannelBuffer<T, MAX_CHANNELS>
{
    fn into(self) -> Vec<T> {
        self.data
    }
}

pub struct InstanceChannelIter<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize> {
    buf: &'a InstanceChannelBuffer<T, MAX_CHANNELS>,
    curr: usize,
}

impl<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize> Iterator
    for InstanceChannelIter<'a, T, MAX_CHANNELS>
{
    type Item = VarChannelBufferRef<'a, T, MAX_CHANNELS>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.curr;
        self.curr += 1;

        if current < self.buf.num_instances {
            // # SAFETY:
            // We have checked that `current` is within bounds.
            unsafe { Some(self.buf.instance_unchecked(current)) }
        } else {
            None
        }
    }
}

pub struct InstanceChannelIterMut<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize> {
    buf: &'a mut InstanceChannelBuffer<T, MAX_CHANNELS>,
    curr: usize,
}

impl<'a, T: Clone + Copy + Default, const MAX_CHANNELS: usize> Iterator
    for InstanceChannelIterMut<'a, T, MAX_CHANNELS>
{
    type Item = VarChannelBufferRefMut<'a, T, MAX_CHANNELS>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.curr;
        self.curr += 1;

        if current < self.buf.num_instances {
            // # SAFETY:
            // * We have checked that `current` is within bounds.
            // * The constructors, `set_num_instances`, and `set_num_instances_uninit` ensure
            // that `self.data.len() == self.instance_length * self.num_instances`.
            // * The buffer is borrowed as mutable, and these slices do not overlap, so
            // iterating over them will not break mutability rules.
            unsafe {
                Some(VarChannelBufferRefMut {
                    data: core::slice::from_raw_parts_mut(
                        self.buf
                            .data
                            .as_mut_ptr()
                            .add(self.buf.instance_length * current),
                        self.buf.instance_length,
                    ),
                    channels: self.buf.channels,
                    frames: self.buf.frames,
                })
            }
        } else {
            None
        }
    }
}
