//! # Slab Allocator
//!
//! L'allocateur principal 
//! Il peut gérer plusieurs caches pour différentes tailles d'objets.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;
use crate::slab_cache::SlabCache;

/// Un allocateur de type slab.
/// Pour chaque taille d'objet demandée, un cache dédié est utilisé.

pub struct SlabAllocator {
    /// Cache pour les petites allocations (< 256 bytes on a pris 64)
    small_cache: SlabCache,
    /// Cache pour les allocations moyennes (256)
    medium_cache: SlabCache,
}
