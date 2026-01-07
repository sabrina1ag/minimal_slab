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

