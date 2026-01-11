#![no_std]

extern crate alloc;

pub mod page_allocator;
pub mod slab;
pub mod slab_cache;

#[cfg(test)]
mod tests {
    use super::slab_cache::SlabCache;

    #[test]
    fn slab_cache_creation() {
        let cache = SlabCache::new(32);
        assert_eq!(cache.object_size(), 32);
        assert_eq!(cache.slab_count(), 0);
        assert_eq!(cache.total_allocated(), 0);
    }

    #[test]
    fn slab_cache_allocate_one() {
    let mut cache = SlabCache::new(64);

    let ptr = cache.allocate();
    assert!(ptr.is_some());
    assert_eq!(cache.total_allocated(), 1);
    }

    #[test]
    fn slab_cache_allocate_multiple() {
    let mut cache = SlabCache::new(32);

    let a = cache.allocate();
    let b = cache.allocate();
    let c = cache.allocate();

    assert!(a.is_some());
    assert!(b.is_some());
    assert!(c.is_some());
    assert_eq!(cache.total_allocated(), 3);
    }
#[test]
fn slab_cache_deallocate_one() {
    let mut cache = SlabCache::new(32);

    let ptr = cache.allocate().unwrap();

    unsafe {
        let ok = cache.deallocate(ptr);
        assert!(ok);
    }

    assert_eq!(cache.total_allocated(), 0);
}

#[test]
fn slab_cache_reuse_after_free() {
    let mut cache = SlabCache::new(64);

    let ptr1 = cache.allocate().unwrap();
    unsafe {
        cache.deallocate(ptr1);
    }

    let ptr2 = cache.allocate();
    assert!(ptr2.is_some());
    assert_eq!(cache.total_allocated(), 1);
}

#[test]
fn slab_cache_multiple_slabs() {
    let mut cache = SlabCache::new(128);

    // on force plusieurs allocations
    let mut ptrs = Vec::new();
    for _ in 0..100 {
        if let Some(p) = cache.allocate() {
            ptrs.push(p);
        }
    }

    assert!(cache.slab_count() >= 1);
    assert_eq!(cache.total_allocated(), ptrs.len());
}

#[test]
fn slab_cache_deallocate_invalid_pointer() {
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

}
