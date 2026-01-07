//! # Slab
//! Chaque slab maintient une liste libre des objets disponibles.
use core::ptr::NonNull;
use crate::page_allocator::PAGE_SIZE;

/// Un slab gère un bloc de mémoire divisé en objets de taille fixe.
///
/// Le slab utilise une liste libre chaînée pour suivre les objets disponibles.
/// Chaque objet libre contient un pointeur vers le prochain objet libre.
pub struct Slab {
    /// Pointeur vers le début de la mémoire du slab
    memory: NonNull<u8>,
    /// Taille de chaque objet dans le slab
    object_size: usize,
    /// Nombre d'objets dans le slab
    num_objects: usize,
    /// Pointeur vers le premier objet libre (None si le slab est plein)
    free_list: Option<NonNull<u8>>,
    /// Nombre d'objets actuellement alloués
    allocated_count: usize,
}
impl Slab {
       /// Vérifie si le slab est plein.
    pub fn is_full(&self) -> bool {
        self.free_list.is_none()
    }

    /// Vérifie si le slab est vide (tous les objets sont libres).
    pub fn is_empty(&self) -> bool {
        self.allocated_count == 0
    }

    /// Retourne le nombre d'objets alloués.
    pub fn allocated_count(&self) -> usize {
        self.allocated_count
    }

    /// Retourne le nombre total d'objets.
    pub fn capacity(&self) -> usize {
        self.num_objects
    }

    /// Retourne la taille de chaque objet.
    pub fn object_size(&self) -> usize {
        self.object_size
    }

    /// Retourne le pointeur vers la mémoire du slab.
    pub fn memory(&self) -> NonNull<u8> {
        self.memory
    }
}
