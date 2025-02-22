use core::pin::Pin;

use crate::{ChannelBufferRef, ChannelBufferRefMut};

/// A memory-efficient buffer of samples with a fixed compile-time number of instances each with a
/// fixed compile-time number of `CHANNELS`. Each channel has a fixed runtime number of `frames`
/// (samples in a single channel of audio).
#[derive(Debug)]
pub struct InstanceChannelBuffer<
    T: Clone + Copy + Default + Unpin + Sized,
    const INSTANCES: usize,
    const CHANNELS: usize,
> {
    data: Pin<Vec<T>>,
    offsets: [[*mut T; CHANNELS]; INSTANCES],
    frames: usize,
    instance_length: usize,
}

impl<T: Clone + Copy + Default + Unpin + Sized, const INSTANCES: usize, const CHANNELS: usize>
    InstanceChannelBuffer<T, INSTANCES, CHANNELS>
{
    const _COMPILE_TIME_ASSERTS: () = {
        assert!(INSTANCES > 0);
        assert!(CHANNELS > 0);
    };

    /// Create an empty [`InstanceBuffer`] with no allocated capacity.
    pub fn empty() -> Self {
        let _ = Self::_COMPILE_TIME_ASSERTS;

        let mut data = Pin::new(Vec::<T>::new());

        let offsets = core::array::from_fn(|_| core::array::from_fn(|_| data.as_mut_ptr()));

        Self {
            data,
            offsets,
            frames: 0,
            instance_length: 0,
        }
    }

    /// Create a new [`InstanceChannelBuffer`] allocated with the given number of
    /// `instances`, each with the given number of `frames` (samples in a single channel
    /// of audio).
    ///
    /// All data will be initialized with the default value.
    pub fn new(num_instances: usize, frames: usize) -> Self {
        let _ = Self::_COMPILE_TIME_ASSERTS;

        let instance_length = frames * CHANNELS;
        let buffer_len = instance_length * num_instances;

        let mut data = Vec::<T>::new();
        data.reserve_exact(buffer_len);
        data.resize(buffer_len, Default::default());

        let mut data = Pin::new(data);

        // SAFETY: All of these pointers point to valid memory in the vec.
        let offsets = unsafe {
            core::array::from_fn(|inst_i| {
                core::array::from_fn(|ch_i| {
                    data.as_mut_ptr()
                        .add((instance_length * inst_i) + (frames * ch_i))
                })
            })
        };

        Self {
            data,
            offsets,
            frames,
            instance_length,
        }
    }

    /// Create a new [`InstanceChannelBuffer`] allocated with the given number of
    /// `instances`, each with the given number of `frames` (samples in a single channel
    /// of audio).
    ///
    /// No data will be initialized.
    ///
    /// # Safety
    /// Any data must be initialized before reading.
    pub unsafe fn new_uninit(num_instances: usize, frames: usize) -> Self {
        let _ = Self::_COMPILE_TIME_ASSERTS;

        let instance_length = frames * CHANNELS;
        let buffer_len = instance_length * num_instances;

        let mut data = Vec::<T>::new();
        data.reserve_exact(buffer_len);
        data.set_len(buffer_len);

        let mut data = Pin::new(data);

        // SAFETY: All of these pointers point to valid memory in the vec.
        let offsets = unsafe {
            core::array::from_fn(|inst_i| {
                core::array::from_fn(|ch_i| {
                    data.as_mut_ptr()
                        .add((instance_length * inst_i) + (frames * ch_i))
                })
            })
        };

        Self {
            data,
            offsets,
            frames,
            instance_length,
        }
    }

    /// The number of instances in this buffer.
    pub fn num_instances(&self) -> usize {
        self.offsets.len()
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

    /// Get an immutable reference to the instance at the given index.
    ///
    /// Returns `None` if `index` is out of bounds.
    #[inline(always)]
    pub fn instance<'a>(&'a self, index: usize) -> Option<ChannelBufferRef<'a, T, CHANNELS>> {
        if index < self.num_instances() {
            // # SAFETY:
            // We have checked that `index` is within bounds.
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
    ) -> ChannelBufferRef<'a, T, CHANNELS> {
        // SAFETY:
        //
        // * The constructors ensure that the pointed-to data vec has a length of at
        // least `num_instances * frames * CHANNELS`.
        // * The caller upholds that `index` is within bounds.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * `[*const T; CHANNELS]` and `[*mut T; CHANNELS]` are interchangeable bit-for-bit.
        // * We have asserted at compile-time that both `INSTANCES` and `CHANNELS` are non-zero.
        ChannelBufferRef::from_raw(
            core::slice::from_raw_parts(
                *self.offsets.get_unchecked(index).get_unchecked(0),
                self.instance_length,
            ),
            core::mem::transmute_copy(self.offsets.get_unchecked(index)),
            self.frames,
        )
    }

    /// Get a mutable reference to the instance at the given index.
    ///
    /// Returns `None` if `index` is out of bounds.
    #[inline(always)]
    pub fn instance_mut<'a>(
        &'a mut self,
        index: usize,
    ) -> Option<ChannelBufferRefMut<'a, T, CHANNELS>> {
        if index < self.num_instances() {
            // # SAFETY:
            // We have checked that `index` is within bounds.
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
    ) -> ChannelBufferRefMut<'a, T, CHANNELS> {
        // SAFETY:
        //
        // * The constructors ensure that the pointed-to data vec has a length of at
        // least `num_instances * frames * CHANNELS`.
        // * The caller upholds that `index` is within bounds.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * `self` is borrowed as mutable, ensuring that no other references to the
        // data Vec can exist.
        // * We have asserted at compile-time that both `INSTANCES` and `CHANNELS` are non-zero.
        ChannelBufferRefMut::from_raw(
            core::slice::from_raw_parts_mut(
                *self.offsets.get_unchecked(index).get_unchecked(0),
                self.instance_length,
            ),
            self.offsets.get_unchecked(index).clone(),
            self.frames,
        )
    }

    /// Get an immutable reference to all instances.
    pub fn all_instances<'a>(&'a self) -> [ChannelBufferRef<'a, T, CHANNELS>; INSTANCES] {
        // SAFETY: `inst_i` is always within bounds.
        unsafe { std::array::from_fn(|inst_i| self.instance_unchecked(inst_i)) }
    }

    /// Get a mutable reference to all instances.
    pub fn all_instances_mut<'a>(
        &'a mut self,
    ) -> [ChannelBufferRefMut<'a, T, CHANNELS>; INSTANCES] {
        // SAFETY:
        // * The constructors ensure that the pointed-to data vec has a length of at
        // least `num_instances * frames * CHANNELS`.
        // * `inst_i` is always within bounds.
        // * The Vec is pinned and cannot be moved, so the pointers are valid for the lifetime
        // of the struct.
        // * `self` is borrowed as mutable, and none of these slices overlap, so all
        // mutability rules are being upheld.
        // * We have asserted at compile-time that both `INSTANCES` and `CHANNELS` are non-zero.
        unsafe {
            std::array::from_fn(|inst_i| {
                ChannelBufferRefMut::from_raw(
                    core::slice::from_raw_parts_mut(
                        *self.offsets.get_unchecked(inst_i).get_unchecked(0),
                        self.instance_length,
                    ),
                    self.offsets.get_unchecked(inst_i).clone(),
                    self.frames,
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
}

impl<T: Clone + Copy + Default + Unpin + Sized, const INSTANCES: usize, const CHANNELS: usize>
    Default for InstanceChannelBuffer<T, INSTANCES, CHANNELS>
{
    fn default() -> Self {
        Self::empty()
    }
}

impl<T: Clone + Copy + Default + Unpin + Sized, const INSTANCES: usize, const CHANNELS: usize>
    Into<Vec<T>> for InstanceChannelBuffer<T, INSTANCES, CHANNELS>
{
    fn into(self) -> Vec<T> {
        Pin::<Vec<T>>::into_inner(self.data)
    }
}

impl<T: Clone + Copy + Default + Unpin + Sized, const INSTANCES: usize, const CHANNELS: usize> Clone
    for InstanceChannelBuffer<T, INSTANCES, CHANNELS>
{
    fn clone(&self) -> Self {
        // SAFETY: We initialize all the data below.
        let mut new_self = unsafe { Self::new_uninit(self.num_instances(), self.frames) };

        new_self.raw_mut().copy_from_slice(self.raw());

        new_self
    }
}

// # SAFETY: All the stored pointers are valid for the lifetime of the struct, and
// the public API prevents misuse of the pointers.
unsafe impl<T: Clone + Copy + Default + Unpin + Sized, const INSTANCES: usize, const CHANNELS: usize>
    Send for InstanceChannelBuffer<T, INSTANCES, CHANNELS>
{
}
// # SAFETY: All the stored pointers are valid for the lifetime of the struct, and
// the public API prevents misuse of the pointers.
unsafe impl<T: Clone + Copy + Default + Unpin + Sized, const INSTANCES: usize, const CHANNELS: usize>
    Sync for InstanceChannelBuffer<T, INSTANCES, CHANNELS>
{
}
