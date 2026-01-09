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


    pub unsafe fn new(
        memory: NonNull<u8>,
        object_size: usize,
        num_objects: usize,
    ) -> Self {
        // Construire la liste libre chaînée en partant du dernier objet vers le premier
        // Cela garantit que free_list pointe vers le premier objet
        let mut free_list = None;
        
        // Parcourir les objets de la fin vers le début
        for i in (0..num_objects).rev() {
            let current = unsafe {
                memory.as_ptr().add(i * object_size)
            };
            
            // Stocker le pointeur vers le prochain objet (qui sera None pour le dernier)
            unsafe {
                core::ptr::write(
                    current as *mut Option<NonNull<u8>>,
                    free_list,
                );
            }
            
            // Mettre à jour free_list pour pointer vers l'objet actuel
            free_list = unsafe {
                Some(NonNull::new_unchecked(current))
            };
        }

        Self {
            memory,
            object_size,
            num_objects,
            free_list,
            allocated_count: 0,
        }
    }

    
    
    pub fn allocate(&mut self) -> Option<NonNull<u8>> {
        let free = self.free_list?;

        // Safety: free est garanti valide car il vient de free_list qui contient
        // uniquement des pointeurs vers des objets valides dans la mémoire du slab.
        // Le pointeur a été initialisé lors de la création du slab et pointe vers
        // une région de mémoire valide de taille object_size.
        unsafe {
            // Lire le prochain élément de la liste libre
            // Safety: free.as_ptr() pointe vers un objet valide qui contient
            // un Option<NonNull<u8>> stocké lors de l'initialisation
            self.free_list = core::ptr::read(free.as_ptr() as *const Option<NonNull<u8>>);
            self.allocated_count += 1;
        }

        Some(free)
    }

    /// Libère un objet dans le slab.
    ///
    /// # Safety
    ///
    /// L'appelant doit garantir que:
    /// - `ptr` pointe vers un objet précédemment alloué depuis ce slab via `allocate()`
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
    /// n'appartient pas à ce slab (dans ce cas, l'objet n'est pas libéré).
    pub unsafe fn deallocate(&mut self, ptr: NonNull<u8>) -> bool {
        // Vérifier que le pointeur appartient à ce slab
        let start = self.memory.as_ptr() as usize;
        let end = start + self.num_objects * self.object_size;
        let ptr_addr = ptr.as_ptr() as usize;

        if ptr_addr < start || ptr_addr >= end {
            return false;
        }

        // Vérifier l'alignement
        if (ptr_addr - start) % self.object_size != 0 {
            return false;
        }

        // Ajouter à la liste libre
        // Safety: ptr est garanti valide et aligné, et appartient à ce slab.
        // Nous écrivons un Option<NonNull<u8>> dans la mémoire de l'objet,
        // ce qui est sûr car object_size >= sizeof(Option<NonNull<u8>>) (précondition).
        unsafe {
            core::ptr::write(ptr.as_ptr() as *mut Option<NonNull<u8>>, self.free_list);
        }
        self.free_list = Some(ptr);
        self.allocated_count -= 1;

        true
    }

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
