// SPDX-License-Identifier: MIT
// Myoxus regressions for the parent/subdocument ownership fix.
// Run serially in an isolated Linux process; see README.myoxus.md for limits.

use crate::updates::decoder::Decode;
use crate::{Doc, GetString, Map, ReadTxn, StateVector, Text, Transact};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

struct CountingAllocator;
static LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);

// Count requested live allocations, not allocator arenas or RSS. Forward the
// unchanged pointer/layout to System, and count only successful allocations.
// SAFETY: every allocation operation delegates to System with the caller's
// original pointer, layout and size; accounting does not access allocated memory.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            LIVE_BYTES.fetch_add(layout.size(), Ordering::SeqCst);
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE_BYTES.fetch_sub(layout.size(), Ordering::SeqCst);
        unsafe { System.dealloc(ptr, layout) };
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let next = unsafe { System.realloc(ptr, layout, size) };
        if !next.is_null() {
            if size >= layout.size() {
                LIVE_BYTES.fetch_add(size - layout.size(), Ordering::SeqCst);
            } else {
                LIVE_BYTES.fetch_sub(layout.size() - size, Ordering::SeqCst);
            }
        }
        next
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn parent(nested: bool) -> (Doc, Doc) {
    let parent = Doc::new();
    let child = Doc::new();
    child
        .get_or_insert_text("text")
        .insert(&mut child.transact_mut(), 0, "child 😀");
    if nested {
        child
            .get_or_insert_map("subdocs")
            .insert(&mut child.transact_mut(), "nested", Doc::new());
    }
    parent
        .get_or_insert_map("subdocs")
        .insert(&mut parent.transact_mut(), "child", child.clone());
    (parent, child)
}

#[test]
fn myoxus_ownership_release_orders_and_orphan_operations() {
    for nested in [false, true] {
        let (parent, child) = parent(nested);
        let parent_store = Arc::downgrade(&parent.store.0);
        let child_store = Arc::downgrade(&child.store.0);
        assert!(Doc::ptr_eq(&child.parent_doc().unwrap(), &parent));
        assert!(child.branch_id().is_some());
        drop(parent);
        assert!(
            parent_store.upgrade().is_none(),
            "parent store retained by child"
        );
        assert!(child.parent_doc().is_none());
        assert!(child.branch_id().is_none());
        assert_eq!(
            child
                .get_or_insert_text("text")
                .get_string(&child.transact()),
            "child 😀"
        );
        let unrelated = Doc::new();
        let events = Arc::new(AtomicUsize::new(0));
        let seen = events.clone();
        unrelated
            .observe_subdocs("ownership-regression", move |_, _| {
                seen.fetch_add(1, Ordering::SeqCst);
            })
            .unwrap();
        child.load(&mut unrelated.transact_mut());
        child.destroy(Some(&mut unrelated.transact_mut()));
        child.destroy(None);
        assert_eq!(events.load(Ordering::SeqCst), 0);
        drop(child);
        assert!(
            child_store.upgrade().is_none(),
            "child store retained after final handle"
        );

        let (parent, child) = self::parent(nested);
        let child_store = Arc::downgrade(&child.store.0);
        drop(child);
        assert!(
            child_store.upgrade().is_some(),
            "parent must own inserted child"
        );
        drop(parent);
        assert!(
            child_store.upgrade().is_none(),
            "child-first release leaked store"
        );
    }
}

#[test]
fn myoxus_ownership_load_destroy_remove_and_events() {
    let parent = Doc::new();
    let map = parent.get_or_insert_map("subdocs");
    let child = Doc::new();
    let guid = child.guid();
    let events = Arc::new(Mutex::new(Vec::new()));
    let seen = events.clone();
    parent
        .observe_subdocs("ownership-regression", move |_, event| {
            seen.lock().unwrap().push((
                event.added().count(),
                event.removed().count(),
                event.loaded().count(),
            ));
        })
        .unwrap();
    map.insert(&mut parent.transact_mut(), "child", child.clone());
    assert_eq!(events.lock().unwrap()[0].0, 1);
    let unrelated = Doc::new();
    child.destroy(Some(&mut unrelated.transact_mut()));
    assert!(Doc::ptr_eq(&child.parent_doc().unwrap(), &parent));
    child.load(&mut parent.transact_mut());
    child.destroy(Some(&mut parent.transact_mut()));
    assert!(child.parent_doc().is_none());
    let replacement: Doc = map
        .get(&parent.transact(), "child")
        .unwrap()
        .cast()
        .unwrap();
    assert_eq!(replacement.guid(), guid);
    assert!(Doc::ptr_eq(&replacement.parent_doc().unwrap(), &parent));
    assert!(!replacement.should_load());
    replacement.load(&mut parent.transact_mut());
    assert!(events.lock().unwrap().iter().any(|event| event.2 == 1));
    map.remove(&mut parent.transact_mut(), "child");
    assert!(replacement.parent_doc().is_none());
    assert!(events.lock().unwrap().iter().any(|event| event.1 == 1));
    let parent_store = Arc::downgrade(&parent.store.0);
    drop(replacement);
    drop(child);
    drop(parent);
    assert!(parent_store.upgrade().is_none());
}

#[test]
fn myoxus_ownership_repeated_materialization_releases_heap() {
    // Predeclared envelope: 1 MiB source text; two warm-ups, then twelve calls
    // per case. 256 KiB permits harness bookkeeping but cannot hide even one
    // retained 1 MiB parent. All source/input buffers predate the measurement.
    let text = "x".repeat(1024 * 1024);
    for subdocs in [false, true] {
        let source = Doc::new();
        source
            .get_or_insert_text("text")
            .insert(&mut source.transact_mut(), 0, &text);
        if subdocs {
            source.get_or_insert_map("subdocs").insert(
                &mut source.transact_mut(),
                "child",
                Doc::new(),
            );
        }
        let bytes = source
            .transact()
            .encode_state_as_update_v1(&StateVector::default());
        drop(source);
        for decoder_only in [false, true] {
            let run = || {
                let update = crate::Update::decode_v1(&bytes).unwrap();
                if !decoder_only {
                    let doc = Doc::new();
                    doc.transact_mut().apply_update(update).unwrap();
                    assert_eq!(
                        doc.get_or_insert_text("text")
                            .get_string(&doc.transact())
                            .len(),
                        text.len()
                    );
                }
            };
            run();
            run();
            let baseline = LIVE_BYTES.load(Ordering::SeqCst);
            for call in 0..12 {
                run();
                let retained = LIVE_BYTES.load(Ordering::SeqCst).saturating_sub(baseline);
                assert!(
                    retained < 256 * 1024,
                    "subdocs={} decoder_only={} call={}: retained {} bytes",
                    subdocs,
                    decoder_only,
                    call,
                    retained
                );
            }
        }
    }
}

#[test]
fn myoxus_ownership_error_and_cancelled_scope_release() {
    fn rejected() -> Result<(), crate::encoding::read::Error> {
        let (_parent, _child) = parent(true);
        crate::Update::decode_v1(&[255])?;
        Ok(())
    }
    for _ in 0..2 {
        assert!(rejected().is_err());
    }
    let baseline = LIVE_BYTES.load(Ordering::SeqCst);
    for _ in 0..12 {
        assert!(rejected().is_err());
        // Dropping an uncompleted future must release captured document handles.
        let (parent, child) = parent(true);
        let weak = Arc::downgrade(&parent.store.0);
        let task = async move {
            std::future::pending::<()>().await;
            drop((parent, child));
        };
        drop(task);
        assert!(weak.upgrade().is_none());
    }
    assert!(LIVE_BYTES.load(Ordering::SeqCst).saturating_sub(baseline) < 256 * 1024);
}
