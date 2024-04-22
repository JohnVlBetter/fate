use ash::{util::Align, vk::DeviceSize};
use std::{ffi::c_void, mem::size_of};

pub unsafe fn mem_copy<T: Copy>(ptr: *mut c_void, data: &[T]) {
    let elem_size = size_of::<T>() as DeviceSize;
    let size = data.len() as DeviceSize * elem_size;
    let mut align = Align::new(ptr, elem_size, size);
    align.copy_from_slice(data);
}

pub unsafe fn mem_copy_aligned<T: Copy>(ptr: *mut c_void, alignment: DeviceSize, data: &[T]) {
    let size = data.len() as DeviceSize * alignment;
    let mut align = Align::new(ptr, alignment, size);
    align.copy_from_slice(data);
}
