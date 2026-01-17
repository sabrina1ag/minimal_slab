//! # Slab Allocator
//!
//! L'allocateur principal
//! Il peut gérer plusieurs caches pour différentes tailles d'objets.

use crate::slab_cache::SlabCache;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;

/// Un allocateur de type slab.
/// Pour chaque taille d'objet demandée, un cache dédié est utilisé.

pub struct SlabAllocator {
    /// Cache pour les petites allocations (< 256 bytes on a pris 64)
    small_cache: SlabCache,
    /// Cache pour les allocations moyennes (256)
    medium_cache: SlabCache,
}
impl SlabAllocator {
    /// Crée un nouveau slab allocator.
    pub fn new() -> Self {
        Self {
            small_cache: SlabCache::new(64),
            medium_cache: SlabCache::new(256),
        }
    }

    pub unsafe fn allocate(&self, layout: Layout) -> Option<NonNull<u8>> {
        // Sélectionner le cache approprié selon la taille
        let size = layout.size();

        if size <= 64 {
            // Utiliser le cache pour petites allocations
            unsafe {
                let cache_ptr = &self.small_cache as *const SlabCache as *mut SlabCache;
                (*cache_ptr).allocate()
            }
        } else if size <= 256 {
            // Safety: Même justification que ci-dessus
            unsafe {
                let cache_ptr = &self.medium_cache as *const SlabCache as *mut SlabCache;
                (*cache_ptr).allocate()
            }
        } else {
            // Pour les grandes allocations, on ne peut pas utiliser le slab allocator
            // Dans une vraie implémentation, on déléguerait à un autre allocateur
            None
        }
    }

    pub unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let size = layout.size();

        if size <= 64 {
            // Safety: Même justification que dans allocate()
            unsafe {
                let cache_ptr = &self.small_cache as *const SlabCache as *mut SlabCache;
                (*cache_ptr).deallocate(ptr);
            }
        } else if size <= 256 {
            // Safety: Même justification que dans allocate()
            unsafe {
                let cache_ptr = &self.medium_cache as *const SlabCache as *mut SlabCache;
                (*cache_ptr).deallocate(ptr);
            }
        }
        // Pour les grandes allocations, rien à faire ici
    }
}

impl Default for SlabAllocator {
    fn default() -> Self {
        Self::new()
    }
}
