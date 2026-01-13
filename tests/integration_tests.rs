//! Tests d'intégration pour le slab allocator

#![no_std]
extern crate alloc;

use slab_allocator::{SlabAllocator, SlabCache};
use core::alloc::Layout;
use alloc::vec::Vec;

#[test]
fn test_basic_allocation() {
    let allocator = SlabAllocator::new();
    let layout = Layout::from_size_align(64, 8).unwrap();

    unsafe {
        let ptr = allocator.allocate(layout);
        assert!(ptr.is_some());
        let ptr = ptr.unwrap();
        
        // Écrire et lire des données
        for i in 0..64 {
            core::ptr::write(ptr.as_ptr().add(i), i as u8);
        }
        
        for i in 0..64 {
            assert_eq!(core::ptr::read(ptr.as_ptr().add(i)), i as u8);
        }
        
        allocator.deallocate(ptr, layout);
    }
}

#[test]
fn test_multiple_allocations() {
    let allocator = SlabAllocator::new();
    let layout = Layout::from_size_align(64, 8).unwrap();

    unsafe {
        let mut pointers = Vec::new();
        
        // Allouer 10 objets
        for i in 0..10 {
            let ptr = allocator.allocate(layout);
            assert!(ptr.is_some());
            let ptr = ptr.unwrap();
            
            // Écrire un identifiant unique
            core::ptr::write(ptr.as_ptr(), i as u8);
            pointers.push((ptr, i));
        }
        
        // Vérifier que tous les pointeurs sont différents
        for i in 0..pointers.len() {
            for j in (i + 1)..pointers.len() {
                assert_ne!(pointers[i].0.as_ptr(), pointers[j].0.as_ptr());
            }
        }
        
        // Vérifier les valeurs
        for (ptr, value) in &pointers {
            assert_eq!(core::ptr::read(ptr.as_ptr()), *value as u8);
        }
        
        // Libérer tous les objets
        for (ptr, _) in pointers {
            allocator.deallocate(ptr, layout);
        }
    }
}


