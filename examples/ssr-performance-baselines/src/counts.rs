//! Sample-only instrumentation; never compiled into the ordinary timing binary.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

pub struct Counter;

#[global_allocator]
static GLOBAL: Counter = Counter;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static ZEROED: AtomicUsize = AtomicUsize::new(0);
static REALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);
static REALLOC_OLD: AtomicUsize = AtomicUsize::new(0);
static REALLOC_NEW: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);
static FAILED_ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static FAILED_REALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

fn add_live(bytes: usize) {
    let live = LIVE.fetch_add(bytes, Relaxed).wrapping_add(bytes);
    PEAK.fetch_max(live, Relaxed);
}

// SAFETY: Every operation preserves System's pointer/layout contracts. Bookkeeping
// uses only nonallocating, nonunwinding atomics. Failed realloc keeps the old block.
unsafe impl GlobalAlloc for Counter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let result = unsafe { System.alloc(layout) };
        if result.is_null() {
            FAILED_ALLOCATIONS.fetch_add(1, Relaxed);
            #[cfg(feature = "buffer-study")]
            crate::buffer_study::events::record(5, 0, layout.size());
            #[cfg(feature = "construction-study")]
            crate::construction_study::events::allocation(layout.size(), false, true);
            #[cfg(feature = "serializer-attribution")]
            attribution::allocation(layout.size(), false, true);
        } else {
            ALLOCATIONS.fetch_add(1, Relaxed);
            ALLOCATED_BYTES.fetch_add(layout.size(), Relaxed);
            add_live(layout.size());
            #[cfg(feature = "buffer-study")]
            crate::buffer_study::events::record(1, 0, layout.size());
            #[cfg(feature = "construction-study")]
            crate::construction_study::events::allocation(layout.size(), false, false);
            #[cfg(feature = "serializer-attribution")]
            attribution::allocation(layout.size(), false, false);
        }
        result
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let result = unsafe { System.alloc_zeroed(layout) };
        if result.is_null() {
            FAILED_ALLOCATIONS.fetch_add(1, Relaxed);
            #[cfg(feature = "buffer-study")]
            crate::buffer_study::events::record(5, 0, layout.size());
            #[cfg(feature = "construction-study")]
            crate::construction_study::events::allocation(layout.size(), true, true);
            #[cfg(feature = "serializer-attribution")]
            attribution::allocation(layout.size(), true, true);
        } else {
            ALLOCATIONS.fetch_add(1, Relaxed);
            ZEROED.fetch_add(1, Relaxed);
            ALLOCATED_BYTES.fetch_add(layout.size(), Relaxed);
            add_live(layout.size());
            #[cfg(feature = "buffer-study")]
            crate::buffer_study::events::record(2, 0, layout.size());
            #[cfg(feature = "construction-study")]
            crate::construction_study::events::allocation(layout.size(), true, false);
            #[cfg(feature = "serializer-attribution")]
            attribution::allocation(layout.size(), true, false);
        }
        result
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let result = unsafe { System.realloc(ptr, layout, new_size) };
        if result.is_null() {
            FAILED_REALLOCATIONS.fetch_add(1, Relaxed);
            #[cfg(feature = "buffer-study")]
            crate::buffer_study::events::record(6, layout.size(), new_size);
            #[cfg(feature = "construction-study")]
            crate::construction_study::events::reallocation(layout.size(), new_size, true);
            #[cfg(feature = "serializer-attribution")]
            attribution::reallocation(layout.size(), new_size, true);
        } else {
            REALLOCATIONS.fetch_add(1, Relaxed);
            #[cfg(feature = "construction-study")]
            crate::construction_study::events::reallocation(layout.size(), new_size, false);
            #[cfg(feature = "serializer-attribution")]
            attribution::reallocation(layout.size(), new_size, false);
            REALLOC_OLD.fetch_add(layout.size(), Relaxed);
            REALLOC_NEW.fetch_add(new_size, Relaxed);
            if new_size >= layout.size() {
                add_live(new_size - layout.size());
            } else {
                LIVE.fetch_sub(layout.size() - new_size, Relaxed);
            }
            #[cfg(feature = "buffer-study")]
            crate::buffer_study::events::record(3, layout.size(), new_size);
        }
        result
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        DEALLOCATIONS.fetch_add(1, Relaxed);
        DEALLOCATED_BYTES.fetch_add(layout.size(), Relaxed);
        LIVE.fetch_sub(layout.size(), Relaxed);
        #[cfg(feature = "buffer-study")]
        crate::buffer_study::events::record(4, layout.size(), 0);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub allocations: usize,
    pub zeroed: usize,
    pub reallocations: usize,
    pub deallocations: usize,
    pub allocated_bytes: usize,
    pub realloc_old_bytes: usize,
    pub realloc_new_bytes: usize,
    pub deallocated_bytes: usize,
    pub failed_allocations: usize,
    pub failed_reallocations: usize,
    pub live: usize,
    pub peak: usize,
}

pub fn snapshot() -> Snapshot {
    Snapshot {
        allocations: ALLOCATIONS.load(Relaxed),
        zeroed: ZEROED.load(Relaxed),
        reallocations: REALLOCATIONS.load(Relaxed),
        deallocations: DEALLOCATIONS.load(Relaxed),
        allocated_bytes: ALLOCATED_BYTES.load(Relaxed),
        realloc_old_bytes: REALLOC_OLD.load(Relaxed),
        realloc_new_bytes: REALLOC_NEW.load(Relaxed),
        deallocated_bytes: DEALLOCATED_BYTES.load(Relaxed),
        failed_allocations: FAILED_ALLOCATIONS.load(Relaxed),
        failed_reallocations: FAILED_REALLOCATIONS.load(Relaxed),
        live: LIVE.load(Relaxed),
        peak: PEAK.load(Relaxed),
    }
}

pub fn begin() -> Snapshot {
    PEAK.store(LIVE.load(Relaxed), Relaxed);
    snapshot()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hint::black_box;

    #[test]
    fn known_layout_allocation_reallocation_and_release() {
        let small = Layout::from_size_align(16, 8).unwrap();
        let grown = Layout::from_size_align(32, 8).unwrap();
        let zeroed = Layout::from_size_align(8, 8).unwrap();
        let start = begin();
        // SAFETY: Nonzero valid layouts; successful blocks are reallocated/freed
        // exactly once through the same allocator with their current layout.
        let (p, z) = unsafe {
            let p = GLOBAL.alloc(black_box(small));
            assert!(!p.is_null());
            let p = GLOBAL.realloc(black_box(p), small, 32);
            assert!(!p.is_null());
            let z = GLOBAL.alloc_zeroed(black_box(zeroed));
            assert!(!z.is_null());
            (p, z)
        };
        let held = snapshot();
        unsafe {
            GLOBAL.dealloc(black_box(p), grown);
            GLOBAL.dealloc(black_box(z), zeroed);
        }
        let released = snapshot();
        assert_eq!(held.allocations - start.allocations, 2);
        assert_eq!(held.zeroed - start.zeroed, 1);
        assert_eq!(held.reallocations - start.reallocations, 1);
        assert_eq!(held.allocated_bytes - start.allocated_bytes, 24);
        assert_eq!(held.realloc_old_bytes - start.realloc_old_bytes, 16);
        assert_eq!(held.realloc_new_bytes - start.realloc_new_bytes, 32);
        assert_eq!(held.live - start.live, 40);
        assert_eq!(held.peak - start.live, 40);
        assert_eq!(released.deallocations - start.deallocations, 2);
        assert_eq!(released.deallocated_bytes - start.deallocated_bytes, 40);
        assert_eq!(released.live, start.live);
        assert_eq!(released.failed_allocations, start.failed_allocations);
        assert_eq!(released.failed_reallocations, start.failed_reallocations);
    }
}

// Single-thread sample-only event tags, not allocation ownership tracking.
#[cfg(feature = "serializer-attribution")]
pub mod attribution {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(usize)]
    pub enum Bucket {
        Text,
        Attribute,
        Title,
        Url,
        Body,
        Envelope,
        Other,
    }
    pub const NAMES: [&str; 7] = [
        "text",
        "attribute",
        "title",
        "url",
        "body",
        "envelope",
        "other",
    ];
    // alloc, zeroed subtotal, realloc, alloc bytes, realloc old/new, failures A/R
    static EVENTS: [[AtomicUsize; 8]; 7] = [const { [const { AtomicUsize::new(0) }; 8] }; 7];
    static ACTIVE: AtomicUsize = AtomicUsize::new(Bucket::Other as usize);

    struct Restore(usize);
    impl Drop for Restore {
        fn drop(&mut self) {
            ACTIVE.store(self.0, Relaxed);
        }
    }

    #[inline(never)]
    pub fn tagged<T>(bucket: Bucket, work: impl FnOnce() -> T) -> T {
        let _restore = Restore(ACTIVE.swap(bucket as usize, Relaxed));
        work()
    }

    pub fn snapshot() -> [[usize; 8]; 7] {
        std::array::from_fn(|b| std::array::from_fn(|f| EVENTS[b][f].load(Relaxed)))
    }

    pub(super) fn allocation(bytes: usize, zeroed: bool, failed: bool) {
        let fields = &EVENTS[ACTIVE.load(Relaxed)];
        if failed {
            fields[6].fetch_add(1, Relaxed);
        } else {
            fields[0].fetch_add(1, Relaxed);
            fields[1].fetch_add(usize::from(zeroed), Relaxed);
            fields[3].fetch_add(bytes, Relaxed);
        }
    }

    pub(super) fn reallocation(old: usize, new: usize, failed: bool) {
        let fields = &EVENTS[ACTIVE.load(Relaxed)];
        if failed {
            fields[7].fetch_add(1, Relaxed);
        } else {
            fields[2].fetch_add(1, Relaxed);
            fields[4].fetch_add(old, Relaxed);
            fields[5].fetch_add(new, Relaxed);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn yt146_nested_tags_restore_and_reconcile() {
            assert_eq!(ACTIVE.load(Relaxed), Bucket::Other as usize);
            tagged(Bucket::Body, || {
                tagged(Bucket::Text, || assert_eq!(ACTIVE.load(Relaxed), 0));
                assert_eq!(ACTIVE.load(Relaxed), Bucket::Body as usize);
                let error: Result<(), ()> = tagged(Bucket::Url, || Err(()));
                assert_eq!(error, Err(()));
                assert_eq!(ACTIVE.load(Relaxed), Bucket::Body as usize);
                let panic = std::panic::catch_unwind(|| {
                    tagged(Bucket::Title, || {
                        panic!("intentional tag restoration check")
                    });
                });
                assert!(panic.is_err());
                assert_eq!(ACTIVE.load(Relaxed), Bucket::Body as usize);
            });
            assert_eq!(ACTIVE.load(Relaxed), Bucket::Other as usize);
            let layout = Layout::from_size_align(16, 8).unwrap();
            let grown = Layout::from_size_align(32, 8).unwrap();
            let global = super::super::snapshot();
            let buckets = snapshot();
            // SAFETY: Valid nonzero layouts; same allocator and current layouts.
            let p = tagged(Bucket::Text, || unsafe {
                let p = GLOBAL.alloc(std::hint::black_box(layout));
                assert!(!p.is_null());
                let p = GLOBAL.realloc(std::hint::black_box(p), layout, 32);
                assert!(!p.is_null());
                p
            });
            let held_global = super::super::snapshot();
            let held_buckets = snapshot();
            unsafe { GLOBAL.dealloc(p, grown) };
            assert_eq!(held_global.allocations - global.allocations, 1);
            assert_eq!(held_global.reallocations - global.reallocations, 1);
            assert_eq!(held_buckets[0][0] - buckets[0][0], 1);
            assert_eq!(held_buckets[0][2] - buckets[0][2], 1);
            assert_eq!(held_buckets[0][3] - buckets[0][3], 16);
            assert_eq!(held_buckets[0][4] - buckets[0][4], 16);
            assert_eq!(held_buckets[0][5] - buckets[0][5], 32);
            for b in 1..7 {
                assert_eq!(held_buckets[b], buckets[b]);
            }
            assert_eq!(super::super::snapshot().live, global.live);
        }
    }
}
