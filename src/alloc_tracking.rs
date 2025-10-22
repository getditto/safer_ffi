#![cfg(feature = "alloc-tracking")]

use std::collections::HashMap;
use std::fmt::Write;
use std::sync::LazyLock;
use std::sync::Mutex;

use backtrace::Backtrace;

#[derive(Debug, Clone)]
pub struct AllocInfo {
    pub type_name: &'static str,
    pub alloc_backtrace: Backtrace,
    pub free_backtrace: Option<Backtrace>,
}

#[derive(Debug, Clone)]
pub struct ArcInfo {
    pub type_name: &'static str,
    pub initial_alloc_backtrace: Backtrace,
    pub ref_count: usize,
    pub clone_backtraces: Vec<Backtrace>,
    pub drop_backtraces: Vec<Backtrace>,
}

#[derive(Debug, Default, Clone)]
pub struct AllocationTracker {
    // Regular allocations (Box, Vec, etc.)
    regular_allocs: HashMap<usize, AllocInfo>,
    // Arc allocations with ref counting
    arc_allocs: HashMap<usize, ArcInfo>,
}

pub static ALLOC_TRACKER: LazyLock<Mutex<AllocationTracker>> = LazyLock::new(<_>::default);

impl AllocationTracker {
    // Regular allocation methods
    #[track_caller]
    pub fn track_alloc<T>(addr: *const T) {
        let mut tracker = ALLOC_TRACKER.lock().unwrap();
        tracker.regular_allocs.insert(
            addr as usize,
            AllocInfo {
                type_name: ::core::any::type_name::<T>(),
                alloc_backtrace: Backtrace::new_unresolved(),
                free_backtrace: None,
            },
        );
    }

    #[track_caller]
    pub fn track_free<T>(addr: *const T) {
        let mut tracker = ALLOC_TRACKER.lock().unwrap();
        match tracker.regular_allocs.get_mut(&(addr as usize)) {
            | Some(&mut AllocInfo {
                free_backtrace: ref mut free_backtrace @ None,
                ..
            }) => {
                // OK, found entry
                *free_backtrace = Some(Backtrace::new_unresolved());
            },
            | Some(AllocInfo {
                free_backtrace: Some(free_backtrace),
                alloc_backtrace,
                type_name,
            }) => {
                alloc_backtrace.resolve();
                free_backtrace.resolve();
                let second_free_backtrace = Backtrace::new();
                panic!(
                    "\
                        Error, double free of pointer {addr:p} pointing to type `{type_name}`\n\
                        > It was allocated at\n{alloc_backtrace:?}.\n\
                        \n\
                        > It was (first) deallocated at\n{free_backtrace:?}.\n\
                        \n\
                        > A second deallocation attempt was made at\n{second_free_backtrace:?}.\
                    ",
                )
            },
            | None => panic!(
                "\
                    Error, pointer {addr:p} pointing to type `{type_name}` was \
                    never allocated to begin with.\
                ",
                type_name = ::core::any::type_name::<T>(),
            ),
        }
    }

    // Arc-specific methods
    #[track_caller]
    pub fn track_arc_new<T>(addr: *const T) {
        let mut tracker = ALLOC_TRACKER.lock().unwrap();
        tracker.arc_allocs.insert(
            addr as usize,
            ArcInfo {
                type_name: ::core::any::type_name::<T>(),
                initial_alloc_backtrace: Backtrace::new_unresolved(),
                ref_count: 1,
                clone_backtraces: Vec::new(),
                drop_backtraces: Vec::new(),
            },
        );
    }

    #[track_caller]
    pub fn track_arc_clone<T>(addr: *const T) {
        let mut tracker = ALLOC_TRACKER.lock().unwrap();
        match tracker.arc_allocs.get_mut(&(addr as usize)) {
            | Some(arc_info) => {
                arc_info.ref_count += 1;
                arc_info.clone_backtraces.push(Backtrace::new_unresolved());
            },
            | None => {
                // First time seeing this Arc - assume it existed before
                tracker.arc_allocs.insert(
                    addr as usize,
                    ArcInfo {
                        type_name: ::core::any::type_name::<T>(),
                        // this isn't actually the actual initial allocation backtrace, but it's the
                        // best we can do here
                        initial_alloc_backtrace: Backtrace::new_unresolved(),
                        // we "guess" 2, the i.e. original allocation and this clone
                        ref_count: 2,
                        clone_backtraces: vec![Backtrace::new_unresolved()],
                        drop_backtraces: Vec::new(),
                    },
                );
            },
        }
    }

    #[track_caller]
    pub fn track_arc_drop<T>(addr: *const T) {
        let mut tracker = ALLOC_TRACKER.lock().unwrap();
        if let Some(arc_info) = tracker.arc_allocs.get_mut(&(addr as usize)) {
            arc_info.ref_count = arc_info.ref_count.saturating_sub(1);
            arc_info.drop_backtraces.push(Backtrace::new_unresolved());
        }
    }

    // Unified snapshot ("regular" and arc)
    pub fn snapshot() -> AllocationTracker {
        ALLOC_TRACKER.lock().unwrap().clone()
    }

    // Combined comparison of all allocations
    pub fn compare_to(
        &mut self,
        past: &mut AllocationTracker,
    ) -> Result<(), String> {
        let mut leaks = Vec::new();

        // Check regular allocations
        for (
            addr,
            AllocInfo {
                free_backtrace,
                alloc_backtrace,
                type_name,
            },
        ) in &mut self.regular_allocs
        {
            let &addr = addr;
            if free_backtrace.is_some() {
                continue;
            }
            // Check if the allocation existed before the snapshot was taken
            match past.regular_allocs.get_mut(&addr) {
                | Some(AllocInfo {
                    free_backtrace: None,
                    ..
                }) => {
                    // OK - allocation existed before snapshot
                },
                | Some(AllocInfo {
                    free_backtrace: Some(_),
                    ..
                }) => {
                    // This shouldn't happen - freed in past but not freed now
                    unreachable!()
                },
                // Otherwise, we have a leak!
                | None => {
                    alloc_backtrace.resolve();
                    leaks.push(format!(
                        "Memory leak! Pointer {addr:#x} pointing to type `{type_name}` was not freed!\n\
                        Allocation origin:\n{alloc_backtrace:?}"
                    ));
                },
            }
        }

        // Check Arc allocations
        for (
            addr,
            ArcInfo {
                ref_count,
                type_name,
                initial_alloc_backtrace,
                clone_backtraces,
                drop_backtraces,
            },
        ) in &mut self.arc_allocs
        {
            let &addr = addr;
            if *ref_count == 0 {
                continue; // Properly cleaned up
            }

            match past.arc_allocs.get(&addr) {
                | Some(_) => {
                    // OK - Arc existed before test
                },
                | None => {
                    // Arc leak - created during test with remaining references
                    initial_alloc_backtrace.resolve();

                    let mut leak_msg = format!(
                        "Arc leak! Pointer {addr:#x} of type `{type_name}` has {ref_count} remaining references\n\
                        Initial allocation:\n{initial_alloc_backtrace:?}"
                    );

                    if !clone_backtraces.is_empty() {
                        leak_msg.push_str("\n\nClone operations:");
                        for (i, mut clone_bt) in clone_backtraces.iter().cloned().enumerate() {
                            clone_bt.resolve();
                            write!(leak_msg, "\nClone #{}: {clone_bt:?}", i + 1)
                                .expect("String::write_fmt");
                        }
                    }

                    if !drop_backtraces.is_empty() {
                        leak_msg.push_str("\n\nDrop operations:");
                        for (i, mut drop_bt) in drop_backtraces.iter().cloned().enumerate() {
                            drop_bt.resolve();
                            write!(leak_msg, "\nDrop #{}: {drop_bt:?}", i + 1)
                                .expect("String::write_fmt");
                        }
                    }

                    leaks.push(leak_msg);
                },
            }
        }

        if leaks.is_empty() {
            Ok(())
        } else {
            Err(leaks.join("\n\n"))
        }
    }

    pub fn reset() {
        ::core::mem::take(&mut *ALLOC_TRACKER.lock().unwrap());
    }

    pub fn run(f: impl FnOnce()) -> Result<(), String> {
        Self::reset();
        f();
        Self::snapshot().compare_to(&mut Self::default())
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;
    use crate::arc::ThinArc;
    use crate::boxed::ThinBox;

    #[test]
    #[serial]
    fn test_no_leak() {
        let res = AllocationTracker::run(|| {
            let b = ThinBox::new(42i32);
            drop(b);
        });
        assert!(res.is_ok());
    }

    #[test]
    #[serial]
    fn test_memory_leak() {
        let res = AllocationTracker::run(|| {
            let b = ThinBox::new(42i32);
            ::core::mem::forget(b);
        });
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Memory leak"));
    }

    #[test]
    #[serial]
    fn test_arc_no_leak() {
        let res = AllocationTracker::run(|| {
            let arc = ThinArc::new(42i32);
            drop(arc);
        });
        assert!(res.is_ok());
    }

    #[test]
    #[serial]
    fn test_arc_clone_no_leak() {
        let res = AllocationTracker::run(|| {
            let arc1 = ThinArc::new(42i32);
            let arc2 = arc1.clone();
            let arc3 = arc2.clone();
            drop(arc1);
            drop(arc2);
            drop(arc3);
        });
        assert!(res.is_ok());
    }

    #[test]
    #[serial]
    fn test_arc_leak() {
        let res = AllocationTracker::run(|| {
            let arc = ThinArc::new(42i32);
            ::core::mem::forget(arc);
        });
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Arc leak"));
    }

    #[test]
    #[serial]
    fn test_arc_clone_leak() {
        let res = AllocationTracker::run(|| {
            let arc1 = ThinArc::new(42i32);
            let arc2 = arc1.clone();
            drop(arc1);
            // arc2 is forgotten, should detect leak with ref count 1
            ::core::mem::forget(arc2);
        });
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Arc leak"));
    }

    #[test]
    #[serial]
    fn test_mixed_allocations_no_leak() {
        let res = AllocationTracker::run(|| {
            let box1 = ThinBox::new(42i32);
            let box2 = ThinBox::new(String::from("test"));
            let arc1 = ThinArc::new(100i32);
            let arc2 = arc1.clone();
            drop(box1);
            drop(box2);
            drop(arc1);
            drop(arc2);
        });
        assert!(res.is_ok());
    }

    #[test]
    #[serial]
    fn test_mixed_allocations_box_leak() {
        let res = AllocationTracker::run(|| {
            let box1 = ThinBox::new(42i32);
            let arc1 = ThinArc::new(100i32);
            let arc2 = arc1.clone();

            // Properly clean up arcs but forget the box
            ::core::mem::forget(box1);
            drop(arc1);
            drop(arc2);
        });
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Memory leak"));
    }

    #[test]
    #[serial]
    fn test_mixed_allocations_arc_leak() {
        let res = AllocationTracker::run(|| {
            let box1 = ThinBox::new(42i32);
            let arc1 = ThinArc::new(100i32);
            let arc2 = arc1.clone();

            // Properly clean up box but forget one arc
            drop(box1);
            drop(arc1);
            ::core::mem::forget(arc2);
        });
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Arc leak"));
    }
}
