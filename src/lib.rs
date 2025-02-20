#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub(crate) mod const_buffer_ref;
pub use const_buffer_ref::{ChannelBufferRef, ChannelBufferRefMut};

#[cfg(feature = "variable-channels")]
pub(crate) mod var_buffer_ref;
#[cfg(feature = "variable-channels")]
pub use var_buffer_ref::{VarChannelBufferRef, VarChannelBufferRefMut};

#[cfg(feature = "alloc")]
mod const_buffer;
#[cfg(feature = "alloc")]
pub use const_buffer::ChannelBuffer;

#[cfg(all(feature = "alloc", feature = "variable-channels"))]
mod var_buffer;
#[cfg(all(feature = "alloc", feature = "variable-channels"))]
pub use var_buffer::VarChannelBuffer;

#[cfg(feature = "instance-buffer")]
mod instance_buffer;
#[cfg(feature = "instance-buffer")]
pub use instance_buffer::InstanceChannelBuffer;
