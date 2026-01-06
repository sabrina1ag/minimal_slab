//! Un allocateur de pages simulé pour le slab allocator.
//! Dans un environnement réel, ceci serait remplacé par un vrai page allocator.

use core::alloc::Layout;
use core::ptr::NonNull;

/// Taille d'une page en octets (4KB)
pub const PAGE_SIZE: usize = 4096;

pub struct PageAllocator;
