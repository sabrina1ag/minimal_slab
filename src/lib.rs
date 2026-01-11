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
}
