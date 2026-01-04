#![no_std]

//! # Slab Allocator
//!
//! Un allocateur de mémoire de type "slab" minimal pour environnements no_std.
//!
//! ## Concept
//!
//! Un slab allocator pré-alloue des blocs de mémoire de taille fixe (slabs) et les réutilise
//! pour éviter les fragmentations et améliorer les performances. Chaque slab contient plusieurs
//! objets de la même taille.
//!


use core::ptr::{self, NonNull};

/// Taille minimale d'un objet allouable (en bytes)
const MIN_OBJECT_SIZE: usize = 8;

/// Nombre d'objets par slab par défaut
const DEFAULT_OBJECTS_PER_SLAB: usize = 16;

/// Alignement par défaut
const DEFAULT_ALIGNMENT: usize = 8;

/// Nombre maximum de slabs gérés par l'allocateur
const MAX_SLABS: usize = 8;

/// Un nœud dans la liste chaînée des objets libres
///
#[repr(transparent)]
struct FreeListNode {
    next: *mut FreeListNode,
}

impl FreeListNode {
    /// Crée un nouveau nœud avec le pointeur suivant spécifié
    ///
    /// # Safety
    ///
    /// Le pointeur `next` doit être valide ou null.
    #[inline]
    unsafe fn new(next: *mut FreeListNode) -> Self {
        Self { next }
    }
}

/// Représente un slab de mémoire
///
/// Un slab est un bloc de mémoire contigu qui contient plusieurs objets de taille fixe.
/// Les objets libres sont organisés en liste chaînée.
struct Slab {
    /// Pointeur vers le début de la mémoire du slab
    memory: NonNull<u8>,
    /// Taille de chaque objet dans ce slab (slots)
    object_size: usize,
    /// Nombre d'objets dans ce slab
    object_count: usize,
    /// Tête de la liste des objets libres (liste chainées)
    free_list: *mut FreeListNode,
    /// Nombre d'objets actuellement alloués
    allocated_count: usize,
}

impl Slab {
    /// Crée un nouveau slab avec la taille d'objet et le nombre d'objets spécifiés
    ///
    /// # Safety
    ///
    /// - `object_size` doit être >= MIN_OBJECT_SIZE
    /// - `object_count` doit être > 0
    /// - `memory` doit pointer vers un bloc de mémoire valide d'au moins
    ///   `object_size * object_count` bytes
    /// - La mémoire doit être alignée selon `object_size.max(DEFAULT_ALIGNMENT)`
    /// - La mémoire ne doit pas être utilisée ailleurs
    unsafe fn new(
        memory: NonNull<u8>,
        object_size: usize,
        object_count: usize,
    ) -> Result<Self, AllocError> {
        if object_size < MIN_OBJECT_SIZE {
            return Err(AllocError);
        }
        if object_count == 0 {
            return Err(AllocError);
        }

        // Vérifier que la mémoire est suffisamment alignée
        let alignment = object_size.max(DEFAULT_ALIGNMENT);
        let addr = memory.as_ptr() as usize;
        if addr % alignment != 0 {
            return Err(AllocError);
        }

        // Initialiser la liste libre en chaînant tous les objets
        let mut current: *mut FreeListNode = memory.as_ptr() as *mut FreeListNode;
        let object_ptr_size = object_size;

        for i in 0..object_count {
            let next_index = i + 1;
            let next = if next_index < object_count {
                let next_addr = (memory.as_ptr() as usize) + (next_index * object_ptr_size);
                next_addr as *mut FreeListNode
            } else {
                ptr::null_mut()
            };

            // Écrire le nœud de liste libre dans la mémoire
            ptr::write(current, FreeListNode::new(next));
            current = next;
        }

        Ok(Self {
            memory,
            object_size,
            object_count,
            free_list: memory.as_ptr() as *mut FreeListNode,
            allocated_count: 0,
        })
    }

    /// Alloue un objet depuis ce slab
    ///
    /// # Safety
    ///
    /// - Le slab doit être valide et initialisé
    /// - La mémoire du slab ne doit pas avoir été corrompue
    ///
    /// # Returns
    ///
    /// Retourne un pointeur vers l'objet alloué, ou None si le slab est plein.
    unsafe fn allocate(&mut self) -> Option<NonNull<u8>> {
        if self.free_list.is_null() {
            return None;
        }

        // Retirer le premier nœud de la liste libre
        let node = self.free_list;
        self.free_list = (*node).next;
        self.allocated_count += 1;

        Some(NonNull::new_unchecked(node as *mut u8))
    }

    /// Libère un objet et le remet dans la liste libre
    ///
    /// # Safety
    ///
    /// - `ptr` doit pointer vers un objet valide alloué depuis ce slab
    /// - L'objet ne doit pas être libéré deux fois (double-free)
    /// - L'objet ne doit plus être utilisé après cet appel
    /// - Le pointeur doit être aligné correctement
    unsafe fn deallocate(&mut self, ptr: NonNull<u8>) -> bool {
        let ptr_addr = ptr.as_ptr() as usize;
        let slab_start = self.memory.as_ptr() as usize;
        let slab_end = slab_start + (self.object_size * self.object_count);

        // Vérifier que le pointeur est dans les limites du slab
        if ptr_addr < slab_start || ptr_addr >= slab_end {
            return false;
        }

        // Vérifier l'alignement
        let offset = ptr_addr - slab_start;
        if offset % self.object_size != 0 {
            return false;
        }

        // Remettre l'objet dans la liste libre
        let node = ptr.as_ptr() as *mut FreeListNode;
        ptr::write(node, FreeListNode::new(self.free_list));
        self.free_list = node;
        self.allocated_count -= 1;

        true
    }

    /// Vérifie si le slab peut allouer un objet de la taille spécifiée
    #[inline]
    fn can_allocate(&self, size: usize) -> bool {
        self.object_size >= size && !self.free_list.is_null()
    }

    /// Vérifie si le slab est vide (aucun objet alloué)
    #[inline]
    fn is_empty(&self) -> bool {
        self.allocated_count == 0
    }

    /// Vérifie si le slab est plein
    #[inline]
    fn is_full(&self) -> bool {
        self.allocated_count >= self.object_count
    }
}

