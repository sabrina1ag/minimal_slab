//! Tests d'intégration pour le slab allocator

#![no_std]
extern crate alloc;

use slab_allocator::{SlabAllocator, SlabCache};
use core::alloc::Layout;
use alloc::vec::Vec;
