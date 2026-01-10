
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
    /// Crée un nouveau cache de slabs.
    ///
    /// # Arguments
    ///
    /// * `object_size` - Taille de chaque objet en octets (doit être > 0)
    ///
    /// # Panics
    ///
    /// Panique si `object_size` est 0.
    pub fn new(object_size: usize) -> Self {
        if object_size == 0 {  // ✅ Ajouté
            panic!("object_size must be greater than 0");
        }

        Self {
            object_size,
            slabs: [None, None],
            page_allocator: PageAllocator::new(),
        }
    }

  /// Retourne la taille des objets dans ce cache.
    pub fn object_size(&self) -> usize {
        self.object_size
    }

    /// Retourne le nombre de slabs actuellement dans le cache.
    pub fn slab_count(&self) -> usize {
        self.slabs.iter().filter(|s| s.is_some()).count()
    }

    /// Retourne le nombre total d'objets alloués dans tous les slabs.
    pub fn total_allocated(&self) -> usize {
        self.slabs
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|s| s.allocated_count())
            .sum()
    }

/// Alloue un nouveau slab et l'ajoute au cache.
    ///
    /// # Returns
    ///
    /// `true` si un nouveau slab a été alloué avec succès, `false` sinon.
    fn allocate_new_slab(&mut self) -> bool {
        // Chercher un emplacement libre dans le cache
        for slab_opt in &mut self.slabs {
            if slab_opt.is_none() {
                unsafe {
                    // ❌ ERREUR : pas de vérification si allocate_pages() retourne Some
                    let memory = self.page_allocator.allocate_pages(1).unwrap();
                    let num_objects = objects_per_page(self.object_size);
                    *slab_opt = Some(Slab::new(memory, self.object_size, num_objects));
                    return true;
                }
            }
        }

        false
    }
}
