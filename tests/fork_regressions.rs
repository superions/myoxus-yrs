// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Daniel Scharf

use std::collections::HashMap;
use std::sync::Arc;
use yrs::encoding::read::Cursor;
use yrs::updates::decoder::Decode;
use yrs::{
    Any, Doc, GetString, Map, Number, ReadTxn, StateVector, Text, Transact, UndoManager, Update,
};

#[test]
fn held_subdocument_detaches_when_its_parent_is_dropped() {
    let parent = Doc::new();
    let child = Doc::new();
    let text = child.get_or_insert_text("body");
    text.push(&mut child.transact_mut(), "independent child");
    parent.get_or_insert_map("children").insert(
        &mut parent.transact_mut(),
        "document",
        child.clone(),
    );
    assert!(Doc::ptr_eq(&parent, &child.parent_doc().unwrap()));
    drop(parent);

    assert!(child.parent_doc().is_none());
    assert!(child.branch_id().is_none());
    text.push(&mut child.transact_mut(), " remains editable");
    assert_eq!(
        text.get_string(&child.transact()),
        "independent child remains editable"
    );
}

fn history_with_undo_and_redo() -> (Doc, yrs::TextRef, UndoManager) {
    let doc = Doc::new();
    let text = doc.get_or_insert_text("body");
    let mut history = UndoManager::new();
    history.expand_scope(&doc, &text);
    text.push(&mut doc.transact_mut(), "first");
    history.reset();
    text.push(&mut doc.transact_mut(), " second");
    assert!(history.undo_blocking());
    assert!(history.can_undo());
    assert!(history.can_redo());
    (doc, text, history)
}

#[test]
fn clearing_undo_preserves_usable_redo_history() {
    let (doc, text, mut history) = history_with_undo_and_redo();
    history.clear_undo();
    assert!(!history.can_undo());
    assert!(history.can_redo());
    assert!(history.redo_blocking());
    assert_eq!(text.get_string(&doc.transact()), "first second");
}

#[test]
fn clearing_redo_preserves_usable_undo_history() {
    let (doc, text, mut history) = history_with_undo_and_redo();
    history.clear_redo();
    assert!(!history.can_redo());
    assert!(history.can_undo());
    assert!(history.undo_blocking());
    assert_eq!(text.get_string(&doc.transact()), "");
}

#[test]
fn object_encoding_has_a_defined_order_and_reads_legacy_order() {
    let fields: HashMap<String, Any> = (0..32)
        .map(|i| (format!("key{i:02}"), Any::Bool(true)))
        .collect();
    let iteration: Vec<_> = fields.keys().collect();
    assert!(iteration.windows(2).any(|pair| pair[0] > pair[1]));
    let mut expected = vec![118, 32];
    for i in 0..32 {
        expected.push(5);
        expected.extend_from_slice(format!("key{i:02}").as_bytes());
        expected.push(120);
    }
    let mut encoded = Vec::new();
    Any::Map(Arc::new(fields)).encode(&mut encoded);
    assert_eq!(encoded, expected);

    let value = Any::Map(Arc::new(HashMap::from([
        ("z".into(), Any::Bool(false)),
        ("a".into(), Any::Bool(true)),
    ])));
    let mut encoded = Vec::new();
    value.encode(&mut encoded);
    // lib0 object: two fields, with `a` before `z` regardless of hash iteration.
    assert_eq!(encoded, [118, 2, 1, b'a', 120, 1, b'z', 121]);
    let legacy = [118, 2, 1, b'z', 121, 1, b'a', 120];
    assert_eq!(Any::decode(&mut Cursor::new(&legacy)).unwrap(), value);
}

#[test]
fn bigint_tag_and_full_i64_precision_survive_binary_roundtrips() {
    for value in [1i64, -1, i64::MIN, i64::MAX, (1i64 << 53) + 1] {
        let mut bytes = vec![122];
        bytes.extend_from_slice(&value.to_be_bytes());
        let decoded = Any::decode(&mut Cursor::new(&bytes)).unwrap();
        assert_eq!(decoded, Any::BigInt(value));
        let mut reencoded = Vec::new();
        decoded.encode(&mut reencoded);
        assert_eq!(reencoded, bytes);
    }
}

#[test]
fn bigint_is_preserved_through_both_document_update_formats() {
    for v2 in [false, true] {
        let source = Doc::new();
        let map = source.get_or_insert_map("values");
        map.insert(&mut source.transact_mut(), "small", Any::BigInt(1));
        map.insert(&mut source.transact_mut(), "large", Any::BigInt(i64::MAX));
        let update = if v2 {
            source
                .transact()
                .encode_state_as_update_v2(&StateVector::default())
        } else {
            source
                .transact()
                .encode_state_as_update_v1(&StateVector::default())
        };
        let destination = Doc::new();
        let destination_map = destination.get_or_insert_map("values");
        let decoded = if v2 {
            Update::decode_v2(&update)
        } else {
            Update::decode_v1(&update)
        };
        destination
            .transact_mut()
            .apply_update(decoded.unwrap())
            .unwrap();
        let txn = destination.transact();
        assert_eq!(
            destination_map.get(&txn, "small"),
            Some(yrs::Out::Any(Any::BigInt(1)))
        );
        assert_eq!(
            destination_map.get(&txn, "large"),
            Some(yrs::Out::Any(Any::BigInt(i64::MAX)))
        );
    }
}

#[test]
fn integer_float_boundaries_reject_saturation() {
    assert_eq!(Number::Int(i64::MAX).as_f64(), None);
    assert_eq!(Number::Float(i64::MAX as f64).as_i64(), None);
    assert_eq!(Number::Float(i64::MIN as f64).as_i64(), Some(i64::MIN));
    assert_ne!(Number::Int(i64::MAX), Number::Float(i64::MAX as f64));
    assert_eq!(Any::from((1i64 << 53) + 1), Any::BigInt((1i64 << 53) + 1));
}
