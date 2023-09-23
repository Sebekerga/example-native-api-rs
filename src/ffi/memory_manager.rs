use std::{
    ffi::{c_ulong, c_void},
    ptr::{self, NonNull},
};

/// VTable for MemoryManager object, derived from Native API interface. See original
/// C++ implementation in [example project](https://its.1c.ru/db/files/1CITS/EXE/VNCOMPS/VNCOMPS.zip)
/// from 1C documentation
#[repr(C)]
struct MemoryManagerVTable {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    alloc_memory: unsafe extern "system" fn(
        &MemoryManager,
        *mut *mut c_void,
        c_ulong,
    ) -> bool,
    free_memory: unsafe extern "system" fn(&MemoryManager, *mut *mut c_void),
}

/// MemoryManager object, used to allocate memory for the AddIn
#[repr(C)]
pub struct MemoryManager {
    vptr: &'static MemoryManagerVTable,
}

impl MemoryManager {
    /// Safe wrapper around `alloc_memory` method of the MemoryManager object
    /// to allocate memory for byte array
    /// # Arguments
    /// * `size` - size of the memory block to allocate
    /// # Returns
    /// `Option<NonNull<u8>>` - pointer to the allocated memory block
    pub fn alloc_blob(&self, size: usize) -> Option<NonNull<u8>> {
        let mut ptr = ptr::null_mut::<c_void>();
        unsafe {
            if (self.vptr.alloc_memory)(self, &mut ptr, size as c_ulong) {
                NonNull::new(ptr as *mut u8)
            } else {
                None
            }
        }
    }

    /// Safe wrapper around `alloc_memory` method of the MemoryManager object
    /// to allocate memory for UTF-16 string
    /// # Arguments
    /// * `size` - size of the memory block to allocate
    /// # Returns
    /// `Option<NonNull<u16>>` - pointer to the allocated memory block
    pub fn alloc_str(&self, size: usize) -> Option<NonNull<u16>> {
        let mut ptr = ptr::null_mut::<c_void>();
        unsafe {
            if (self.vptr.alloc_memory)(self, &mut ptr, size as c_ulong * 2) {
                NonNull::new(ptr as *mut u16)
            } else {
                None
            }
        }
    }

    pub fn free_memory(&self, ptr: &mut *mut c_void) {
        unsafe {
            (self.vptr.free_memory)(self, ptr);
        }
    }
}
