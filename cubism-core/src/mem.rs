use std::{alloc, mem, ptr::NonNull};

#[derive(Debug)]
pub struct AlignedMemory<T> {
    ptr: NonNull<T>,
    layout: alloc::Layout,
}

impl<T> AlignedMemory<T> {
    pub fn alloc(size: usize) -> Self {
        let layout = alloc::Layout::from_size_align(size, mem::align_of::<T>()).unwrap();
        let ptr = unsafe { alloc::alloc(layout) as *mut T };
        if let Some(ptr) = NonNull::new(ptr) {
            AlignedMemory { ptr, layout }
        } else {
            alloc::handle_alloc_error(layout)
        }
    }

    pub fn layout(&self) -> &alloc::Layout {
        &self.layout
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }
}

impl<T> Drop for AlignedMemory<T> {
    fn drop(&mut self) {
        unsafe { alloc::dealloc(self.ptr.as_ptr() as *mut u8, self.layout) };
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_mem_alloc() {
        crate::mem::AlignedMemory::<u32>::alloc(100);
    }
}
