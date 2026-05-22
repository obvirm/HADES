use std::mem::size_of;

/// A simple frame-based linear allocator to store transient data
/// like primitives and matrices before they are pushed to GPU.
pub struct FrameArena {
    data: Vec<u8>,
    offset: usize,
}

impl FrameArena {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![0; capacity],
            offset: 0,
        }
    }

    /// Resets the pointer to reuse memory for the next frame
    pub fn reset(&mut self) {
        self.offset = 0;
    }

    /// Pushes a single struct into the arena, returning a reference if successful
    pub fn alloc<T: bytemuck::Pod>(&mut self, item: T) -> Option<&mut T> {
        let size = size_of::<T>();
        let align = std::mem::align_of::<T>();
        
        let padding = (align - (self.offset % align)) % align;
        if self.offset + padding + size > self.data.len() {
            return None;
        }

        self.offset += padding;
        
        let ptr = self.data[self.offset..self.offset + size].as_mut_ptr() as *mut T;
        unsafe {
            ptr.write(item);
        }
        
        let start = self.offset;
        self.offset += size;
        
        unsafe {
            Some(&mut *(self.data[start..start + size].as_mut_ptr() as *mut T))
        }
    }
    
    /// Pushes a slice into the arena
    pub fn alloc_slice<T: bytemuck::Pod>(&mut self, items: &[T]) -> Option<&mut [T]> {
        let size = size_of::<T>() * items.len();
        let align = std::mem::align_of::<T>();
        
        let padding = (align - (self.offset % align)) % align;
        if self.offset + padding + size > self.data.len() {
            return None;
        }

        self.offset += padding;
        
        let start = self.offset;
        let dest = &mut self.data[start..start + size];
        dest.copy_from_slice(bytemuck::cast_slice(items));
        
        self.offset += size;
        
        unsafe {
            Some(std::slice::from_raw_parts_mut(dest.as_mut_ptr() as *mut T, items.len()))
        }
    }
    
    pub fn get_allocated_bytes(&self) -> &[u8] {
        &self.data[..self.offset]
    }
}

pub struct TripleBuffer<T> {
    buffers: [T; 3],
    current: usize,
}

impl<T> TripleBuffer<T> {
    pub fn new<F>(mut init: F) -> Self
    where
        F: FnMut(usize) -> T,
    {
        Self {
            buffers: [init(0), init(1), init(2)],
            current: 0,
        }
    }

    pub fn current(&self) -> &T {
        &self.buffers[self.current]
    }

    pub fn current_mut(&mut self) -> &mut T {
        &mut self.buffers[self.current]
    }

    pub fn next_frame(&mut self) {
        self.current = (self.current + 1) % 3;
    }
}
