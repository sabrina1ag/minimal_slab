//! # Slab
//! Chaque slab maintient une liste libre des objets disponibles.
use core::ptr::NonNull;
use crate::page_allocator::PAGE_SIZE;

/// Un slab gère un bloc de mémoire divisé en objets de taille fixe.
///
/// Le slab utilise une liste libre chaînée pour suivre les objets disponibles.
/// Chaque objet libre contient un pointeur vers le prochain objet libre.
pub struct Slab;
