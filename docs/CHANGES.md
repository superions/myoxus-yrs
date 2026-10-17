# Changes from Yrs 0.28.0

This page describes the maintained library delta. Packaging, repository links
and standalone test setup are separate from changes to document behavior.

## Parent/subdocument ownership

Upstream 0.28.0 retains a parent document through its subdocument ownership
graph. A strong parent reference in that graph can keep the parent and child
stores alive after their external handles are dropped.

The fork makes `ItemContent::Doc` own only the child. The child's parent link
uses a weak store reference and a copied item identifier. It never retains a
raw parent item pointer. Operations that need a parent item resolve that ID
under a transaction on the matching parent store; unrelated transactions do
not acquire authority over the child.

This allows parent-first and child-first release, and allows an externally
held child to remain usable after the parent is gone. Normal deletion,
subdocument updates and events retain their existing roles. Review
`src/block.rs`, `src/doc.rs`, `src/store.rs` and `src/transaction.rs` together
when updating this implementation. `src/branch.rs` adapts the content match.

## Selective undo and redo clearing

Upstream `clear_internal` ignores its `clear_undo` and `clear_redo` arguments
and clears both stacks. The fork honors the flags. Clearing one stack keeps
the other usable, clearing both remains supported, and notifications report
only stacks that actually contained entries. Repeated clearing of an empty
stack does not invent a change notification.

This is a behavior fix in `src/undo.rs`, not a new history model.

## Deterministic object encoding

`Any::Map` writes object entries in sorted key order rather than randomized
`HashMap` iteration order. This applies recursively when objects contain
objects. It avoids changes in encoded object bytes caused only by a different
hash-map iteration order.

Older unsorted object bytes are still accepted. Consumers comparing raw bytes
must expect newly encoded output to differ. The change does not sort CRDT map
operations, normalize client IDs, or make every semantically equal document
have a single canonical update representation.

## Numeric values and BigInt

Upstream 0.28.0 introduces `Number::{Int, Float}` and decodes lib0 BigInt tag
122 into `Number::Int`. This loses the original JavaScript Number/BigInt type
distinction, even for values such as `1n` that fit in a small integer.

The fork adopts the new `Number` API while retaining `Any::BigInt(i64)`:

- Tag 122 decodes to `BigInt` and `BigInt` re-encodes with tag 122.
- Integer constructors outside JavaScript's safe-integer range retain the
  earlier BigInt behavior used by Myoxus.
- Serde and primitive conversions support that variant.
- Exact integer/float conversion checks reject saturation and rounding at
  the signed 64-bit upper boundary.

This is intentionally different from the upstream 0.28.0 public enum. Callers
must review exhaustive matches and their own JSON/Serde representation. JSON
does not provide a general lossless BigInt type. Preservation here concerns
the supported lib0/Yjs binary values; it does not add arbitrary-size integers.
Ordinary numbers otherwise retain the 0.28.0 numeric encoding choices.

## String storage

`compact_str` 0.10.0 replaces `smallstr` in `SplittableString` and preliminary
text buffers. Inline storage, independent mutable clones, string splitting,
merging and byte/UTF-16 offset semantics remain the intended contract.
This removes `smallstr` from the dependency graph. It does not establish a
performance improvement for every workload.

## Compiler warnings

The carried cleanup removes unused imports and unnecessary mutable bindings,
gates weak-only bindings with their consumers and spells elided return
lifetimes as `'_`. It adds no warning suppression.

The Myoxus server configuration observed 31 library warnings after the 0.28
migration, compared with the preceding 56: 30 dead-code diagnostics and one
overwritten initialization remain. Counts depend on compiler and features;
this is a recorded configuration-specific result, not a warning-free claim.

## Standalone packaging

The public package and repository are named `myoxus-yrs`; the Rust library is
still `yrs`. Original source examples can retain their imports. The manifest
and license credit Daniel Scharf for the Myoxus adaptations and retain the
original authors.

API docs identify this fork and no longer load the upstream logo. Test dataset
paths are relative to this package instead of the original monorepo. Allocator
instrumentation requires the explicit `myoxus_memory_probes` cfg. Ordinary
`--all-features` qualification does not activate it.

The inherited awareness-summary test uses the same fixed clock for both peers:
its state equality includes receive timestamps, which are local metadata and
are not synchronized over the protocol. This removes dependence on completing
both operations in the same wall-clock millisecond; protocol assertions remain.
