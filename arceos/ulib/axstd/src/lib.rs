//! # The ArceOS Standard Library
//!
//! The [ArceOS] Standard Library is a mini-std library, with an interface similar
//! to rust [std], but calling the functions directly in ArceOS modules, instead
//! of using libc and system calls.
//!
//! These features are exactly the same as those in [axfeat], they are used to
//! provide users with the selection of features in axfeat, without import
//! [axfeat] additionally:
//!
//! ## Cargo Features
//!
//! - CPU
//!     - `smp`: Enable SMP (symmetric multiprocessing) support.
//!     - `fp_simd`: Enable floating point and SIMD support.
//! - Interrupts:
//!     - `irq`: Enable interrupt handling support.
//! - Memory
//!     - `alloc`: Enable dynamic memory allocation.
//!     - `alloc-tlsf`: Use the TLSF allocator.
//!     - `alloc-slab`: Use the slab allocator.
//!     - `alloc-buddy`: Use the buddy system allocator.
//!     - `paging`: Enable page table manipulation.
//!     - `tls`: Enable thread-local storage.
//! - Task management
//!     - `multitask`: Enable multi-threading support.
//!     - `sched_fifo`: Use the FIFO cooperative scheduler.
//!     - `sched_rr`: Use the Round-robin preemptive scheduler.
//!     - `sched_cfs`: Use the Completely Fair Scheduler (CFS) preemptive scheduler.
//! - Upperlayer stacks
//!     - `fs`: Enable file system support.
//!     - `myfs`: Allow users to define their custom filesystems to override the default.
//!     - `net`: Enable networking support.
//!     - `dns`: Enable DNS lookup support.
//!     - `display`: Enable graphics support.
//! - Device drivers
//!     - `bus-mmio`: Use device tree to probe all MMIO devices.
//!     - `bus-pci`: Use PCI bus to probe all PCI devices.
//!     - `driver-ramdisk`: Use the RAM disk to emulate the block device.
//!     - `driver-ixgbe`: Enable the Intel 82599 10Gbit NIC driver.
//!     - `driver-bcm2835-sdhci`: Enable the BCM2835 SDHCI driver (Raspberry Pi SD card).
//! - Logging
//!     - `log-level-off`: Disable all logging.
//!     - `log-level-error`, `log-level-warn`, `log-level-info`, `log-level-debug`,
//!       `log-level-trace`: Keep logging only at the specified level or higher.
//!
//! [ArceOS]: https://github.com/arceos-org/arceos

#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
#[doc(no_inline)]
pub use alloc::{boxed, format, string, vec};

#[cfg(feature = "alloc")]
pub mod collections {
    pub use alloc::collections::*;

    use core::hash::BuildHasherDefault;
    use hashbrown::HashMap as RawHashMap;
    use rustc_hash::FxHasher;

    type Inner<K, V> = RawHashMap<K, V, BuildHasherDefault<FxHasher>>;

    /// 基于 hashbrown 的哈希表，接口对齐 `std::collections::HashMap`。
    pub struct HashMap<K, V> {
        inner: Inner<K, V>,
    }

    impl<K, V> Default for HashMap<K, V> {
        fn default() -> Self {
            Self { inner: Inner::default() }
        }
    }

    impl<K, V> HashMap<K, V> {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn insert(&mut self, key: K, value: V) -> Option<V>
        where
            K: Eq + core::hash::Hash,
        {
            self.inner.insert(key, value)
        }

        pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
            self.inner.iter()
        }

        pub fn len(&self) -> usize {
            self.inner.len()
        }

        pub fn is_empty(&self) -> bool {
            self.inner.is_empty()
        }
    }
}

#[doc(no_inline)]
pub use core::{arch, cell, cmp, hint, marker, mem, ops, ptr, slice, str};

#[macro_use]
mod macros;

pub mod env;
pub mod io;
pub mod os;
pub mod process;
pub mod sync;
pub mod thread;
pub mod time;

#[cfg(feature = "fs")]
pub mod fs;
#[cfg(feature = "net")]
pub mod net;
