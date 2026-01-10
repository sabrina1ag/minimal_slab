//! # Slab Cache
//!
//! Un cache de slabs gère plusieurs slabs et alloue automatiquement
//! de nouveaux slabs quand les existants sont pleins.

use core::ptr::NonNull;
use crate::page_allocator::PageAllocator;
use crate::slab::{Slab, objects_per_page};

/// Un cache de slabs gère plusieurs slabs pour allouer des objets de taille fixe.
///
/// Le cache maintient une liste de slabs et alloue automatiquement un nouveau
/// slab quand tous les slabs existants sont pleins.

pub struct SlabCache {

    /// Taille de chaque objet
    object_size: usize,
    /// Liste des slabs (minimum 2 pour simuler la structure)
    slabs: [Option<Slab>; 2],
    /// Allocateur de pages utilisé pour créer de nouveaux slabs
    page_allocator: PageAllocator,

}
impl SlabCache {
    /// Crée un nouveau cache de slabs
    ///
    /// # Arguments
    ///
    /// * `object_size` - Taille de chaque objet en octets (doit être > 0)
    ///
    /// # Panics
    ///
    /// Panique si `object_size` est 0.
    pub fn new(object_size: usize) -> Self {
        if object_size == 0 {
            panic!("object_size must be greater than 0");
        }

        Self {
            object_size,
            slabs: [None, None],
            page_allocator: PageAllocator::new(),
        }
    }
