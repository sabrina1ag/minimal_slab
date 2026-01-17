extern crate alloc;

use core::alloc::Layout;
use slab_allocator::slab_allocator::SlabAllocator;
use slab_allocator::slab_cache::SlabCache;
use std::vec::Vec;

//
// Tests SlabAllocator
//

#[test]
fn allocator_basic_allocation_read_write() {
    let allocator = SlabAllocator::new();
    let layout = Layout::from_size_align(64, 8).unwrap();

    unsafe {
        let ptr = allocator.allocate(layout).expect("allocation failed");

        // Écrire 0..63
        for i in 0..64 {
            core::ptr::write(ptr.as_ptr().add(i), i as u8);
        }

        // Relire 0..63
        for i in 0..64 {
            let v = core::ptr::read(ptr.as_ptr().add(i));
            assert_eq!(v, i as u8);
        }

        allocator.deallocate(ptr, layout);
    }
}

#[test]
fn allocator_multiple_allocations_unique_and_valid() {
    let allocator = SlabAllocator::new();
    let layout = Layout::from_size_align(64, 8).unwrap();

    unsafe {
        let mut ptrs: Vec<core::ptr::NonNull<u8>> = Vec::new();

        // Allouer plusieurs blocs et écrire un marqueur
        for i in 0..10u8 {
            let p = allocator.allocate(layout).expect("allocation failed");
            core::ptr::write(p.as_ptr(), i);
            ptrs.push(p);
        }

        // Vérifier que tous les pointeurs sont différents
        for i in 0..ptrs.len() {
            for j in (i + 1)..ptrs.len() {
                assert_ne!(ptrs[i].as_ptr(), ptrs[j].as_ptr());
            }
        }

        // Vérifier les valeurs écrites
        for (i, p) in ptrs.iter().enumerate() {
            let v = core::ptr::read(p.as_ptr());
            assert_eq!(v, i as u8);
        }

        // Libérer
        for p in ptrs {
            allocator.deallocate(p, layout);
        }
    }
}

#[test]
fn allocator_reuse_after_free_smoke() {
    let allocator = SlabAllocator::new();
    let layout = Layout::from_size_align(64, 8).unwrap();

    unsafe {
        let p1 = allocator.allocate(layout).expect("allocation failed");
        core::ptr::write(p1.as_ptr(), 42u8);
        allocator.deallocate(p1, layout);

        // Réallouer : on ne suppose pas que l'adresse est identique,
        // on vérifie juste que c'est ré-allouable et utilisable.
        let p2 = allocator.allocate(layout).expect("re-allocation failed");
        core::ptr::write(p2.as_ptr(), 100u8);
        assert_eq!(core::ptr::read(p2.as_ptr()), 100u8);

        allocator.deallocate(p2, layout);
    }
}

//
// Tests SlabCache
//

#[test]
fn slab_cache_creation() {
    let cache = SlabCache::new(32);
    assert_eq!(cache.object_size(), 32);
    assert_eq!(cache.slab_count(), 0);
    assert_eq!(cache.total_allocated(), 0);
}

#[test]
fn slab_cache_allocate_deallocate_one() {
    let mut cache = SlabCache::new(32);

    let p = cache.allocate().expect("allocation failed");
    assert_eq!(cache.total_allocated(), 1);

    unsafe {
        let ok = cache.deallocate(p);
        assert!(ok);
    }
    assert_eq!(cache.total_allocated(), 0);
}

#[test]
fn slab_cache_allocate_many_and_free_all() {
    let mut cache = SlabCache::new(64);

    let mut ptrs = Vec::new();
    for _ in 0..100 {
        if let Some(p) = cache.allocate() {
            ptrs.push(p);
        } else {
            break;
        }
    }

    assert!(!ptrs.is_empty());
    assert_eq!(cache.total_allocated(), ptrs.len());

    unsafe {
        for p in ptrs {
            assert!(cache.deallocate(p));
        }
    }
    assert_eq!(cache.total_allocated(), 0);
    assert!(cache.slab_count() >= 1); // au moins un slab a dû être créé
}

#[test]
fn slab_cache_reuse_after_free_smoke() {
    let mut cache = SlabCache::new(64);

    let p1 = cache.allocate().expect("allocation failed");
    unsafe {
        assert!(cache.deallocate(p1));
    }

    let p2 = cache.allocate();
    assert!(p2.is_some());
    assert_eq!(cache.total_allocated(), 1);

    unsafe {
        assert!(cache.deallocate(p2.unwrap()));
    }
    assert_eq!(cache.total_allocated(), 0);
}

#[test]
fn slab_cache_deallocate_invalid_pointer_returns_false() {
    let mut cache = SlabCache::new(32);

    let fake_ptr = core::ptr::NonNull::dangling();
    unsafe {
        let ok = cache.deallocate(fake_ptr);
        assert!(!ok);
    }
}

#[test]
#[should_panic]
fn slab_cache_zero_size_panics() {
    SlabCache::new(0);
}
