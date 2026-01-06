//! Un allocateur de pages simulé pour le slab allocator.
//! Dans un environnement réel, ceci serait remplacé par un vrai page allocator.

use core::alloc::Layout;
use core::ptr::NonNull;

/// Taille d'une page en octets (4KB)
pub const PAGE_SIZE: usize = 4096;

pub struct PageAllocator;

impl PageAllocator {
    /// instance page allocator
    pub const fn new() -> Self {  
        Self
    }
    //Allouer des pages contigues
    pub unsafe fn allocate_pages(&self, num_pages: usize) -> Option<NonNull<u8>> {
        let size = num_pages * PAGE_SIZE;
        let layout = Layout::from_size_align(size, PAGE_SIZE).ok()?;

        unsafe {
            // alloc:alloc pour simuler vrai pages alloués
            let ptr = alloc::alloc::alloc(layout);
            if ptr.is_null() {
                None
            } else {
                Some(NonNull::new_unchecked(ptr))
            }
        }
    }
}


impl Default for PageAllocator {
    fn default() -> Self {
        Self::new()
    }
}


