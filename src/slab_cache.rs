//! # Slab Cache
//!
//! Un cache de slabs gère plusieurs slabs et alloue automatiquement
//! de nouveaux slabs quand les existants sont pleins.

use crate::page_allocator::PageAllocator;
use crate::slab::{objects_per_page, Slab};
use core::ptr::NonNull;

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
        if object_size == 0 {
            // ✅ Ajouté
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
    /// `true` si un nouveau slab a été alloué avec succès, `false` sinon
    /// (par exemple, si tous les emplacements du cache sont occupés ou si
    /// l'allocation de page a échoué).
    ///
    /// # Safety
    ///
    /// Cette fonction est safe car elle gère correctement les allocations
    /// et initialise le slab de manière sûre. Les opérations unsafe internes
    /// sont encapsulées et vérifiées.  
    fn allocate_new_slab(&mut self) -> bool {
        // Chercher un emplacement libre dans le cache
        for slab_opt in &mut self.slabs {
            if slab_opt.is_none() {
                unsafe {
                    // Safety: allocate_pages() retourne un pointeur valide ou None.
                    // Si Some, la mémoire est valide et alignée sur PAGE_SIZE.
                    if let Some(memory) = self.page_allocator.allocate_pages(1) {
                        let num_objects = objects_per_page(self.object_size);
                        // Safety: Slab::new() nécessite que la mémoire soit valide,
                        // alignée, et de taille suffisante. Ces conditions sont satisfaites
                        // car memory pointe vers une page complète (PAGE_SIZE octets)
                        // et object_size > 0 (vérifié dans new()).
                        *slab_opt = Some(Slab::new(memory, self.object_size, num_objects));
                        return true;
                    }
                }
            }
        }

        false
    }
    /// Alloue un objet depuis le cache.
    ///
    /// Si tous les slabs sont pleins, un nouveau slab est alloué automatiquement
    /// (si de l'espace est disponible dans le cache).
    ///
    /// # Returns
    ///
    /// Un pointeur vers l'objet alloué, ou `None` si l'allocation a échoué
    /// (par exemple, si tous les slabs sont pleins et qu'aucun nouveau slab
    /// ne peut être alloué).

    pub fn allocate(&mut self) -> Option<NonNull<u8>> {
        // Chercher un slab avec de l'espace libre
        for slab_opt in &mut self.slabs {
            if let Some(ref mut slab) = slab_opt {
                if !slab.is_full() {
                    return slab.allocate();
                }
            }
        }

        // Tous les slabs sont pleins ou n'existent pas, allouer un nouveau slab
        if !self.allocate_new_slab() {
            // ✅ Vérification ajoutée
            return None;
        }

        // ✅ Réessayer l'allocation depuis le nouveau slab
        for slab_opt in &mut self.slabs {
            if let Some(ref mut slab) = slab_opt {
                if !slab.is_full() {
                    return slab.allocate();
                }
            }
        }

        None
    }

    /// Libère un objet dans le cache.
    ///
    /// # Safety
    ///
    /// L'appelant doit garantir que:
    /// - `ptr` pointe vers un objet précédemment alloué depuis ce cache via `allocate()`
    /// - L'objet n'a pas déjà été libéré (double-free est undefined behavior)
    /// - Aucune référence active n'existe vers cet objet (use-after-free est undefined behavior)
    /// - Le pointeur n'a pas été modifié depuis l'allocation
    ///
    /// # Arguments
    ///
    /// * `ptr` - Pointeur vers l'objet à libérer
    ///
    /// # Returns
    ///
    /// `true` si l'objet a été libéré avec succès, `false` si le pointeur
    /// n'appartient pas à ce cache (dans ce cas, l'objet n'est pas libéré).
    ///
    /// # Panics
    ///
    /// Ne panique jamais, mais retourne `false` pour les pointeurs invalides.   
    pub unsafe fn deallocate(&mut self, ptr: NonNull<u8>) -> bool {
        // Chercher le slab qui contient ce pointeur
        for slab_opt in &mut self.slabs {
            if let Some(ref mut slab) = slab_opt {
                // Safety: slab.deallocate() vérifie que ptr appartient au slab.
                // Si c'est le cas, les préconditions sont satisfaites.
                unsafe {
                    if slab.deallocate(ptr) {
                        return true;
                    }
                }
            }
        }

        false
    }
}
impl Drop for SlabCache {
    /// Libère automatiquement tous les slabs lors de la destruction du cache.
    ///
    /// # Safety
    ///
    /// Cette fonction est safe car elle libère uniquement la mémoire qui a été
    /// allouée par ce cache. Les opérations unsafe sont encapsulées et vérifiées.
    fn drop(&mut self) {
        // Libérer tous les slabs
        for slab_opt in &mut self.slabs {
            if let Some(slab) = slab_opt.take() {
                // Safety: slab.memory() retourne le pointeur original alloué par
                // allocate_pages(1), donc il est valide pour deallocate_pages(ptr, 1).
                // Aucune référence active n'existe vers cette mémoire car nous prenons
                // possession du slab avec take().
                unsafe {
                    self.page_allocator.deallocate_pages(slab.memory(), 1);
                }
            }
        }
    }
}
