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


}
