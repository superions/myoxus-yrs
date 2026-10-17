# myoxus-yrs

A Myoxus-maintained fork of [Yrs](https://github.com/y-crdt/y-crdt), the Rust
implementation of the [Yjs](https://yjs.dev/) collaborative editing CRDT.

This fork starts from **Yrs 0.28.0** and keeps the changes needed by Myoxus for
document lifetime, editing history and persisted data. It is maintained by
**Daniel Scharf** and is an independent project. The original Yrs authors are
Kevin Jahns and Bartosz Sypytkowski.

## What changes, and why?

| Change | Benefit | Compatibility impact |
| --- | --- | --- |
| Break the parent/subdocument ownership cycle | Dropping a parent releases its store even when a child handle remains; the detached child remains usable. | Parent-dependent operations resolve the child under the matching parent transaction. |
| Honor selective undo/redo clearing | `clear_undo()` preserves redo history; `clear_redo()` preserves undo history. Clear notifications describe the stacks actually changed. | Restores the behavior described by the public methods. |
| Sort keys when encoding `Any::Map` | Equal object values have stable encoded key order, useful for content hashes and revision comparisons. | Encoded bytes can differ from upstream and older unsorted encodings. Existing encodings remain readable. This does not canonicalize every CRDT update. |
| Preserve `Any::BigInt(i64)` and lib0 tag 122 | A JavaScript BigInt stays distinguishable from a JavaScript Number when document values are decoded and re-encoded. | Adds an enum variant compared with upstream 0.28.0; exhaustive matches may need adaptation. |
| Check integer/float conversion boundaries | Reject conversions that would silently round `i64::MAX` to 2^63. | Conversion results can differ where upstream accepted an inexact result. |
| Use `compact_str` instead of `smallstr` | Removes the `smallstr` dependency while retaining inline string storage and the existing byte/UTF-16 operations. | Internal storage change; no new document wire format. No whole-application performance claim. |
| Apply focused compiler-warning fixes | Removes unused imports/bindings and makes elided lifetime notation explicit, without suppressing warnings. | Remaining dead-code and initialization warnings are documented. |

[Detailed changes and tradeoffs](docs/CHANGES.md) explain the boundaries of each
change. [Upstream provenance](docs/UPSTREAM.md) records the exact release and
checksum. This fork does not claim universal drop-in compatibility with Yrs.

## Use from GitHub

The package name is `myoxus-yrs`; the Rust library name remains `yrs`, so existing
`use yrs::...` imports and the upstream examples can keep their names.

```toml
[dependencies]
yrs = { package = "myoxus-yrs", git = "https://github.com/superions/myoxus-yrs", branch = "main" }
```

For reproducible application builds, select a reviewed commit with Cargo's
`rev` option and commit `Cargo.lock`. A different package name does not
automatically replace transitive dependencies on upstream `yrs`; both packages
can otherwise coexist with distinct Rust types.

```rust
use yrs::{Doc, GetString, Text, Transact};

let doc = Doc::new();
let text = doc.get_or_insert_text("body");
text.insert(&mut doc.transact_mut(), 0, "Hello from Myoxus");
assert_eq!(text.get_string(&doc.transact()), "Hello from Myoxus");
```

The manifest prepares version `0.28.0-myoxus.1`. This identifies the upstream
base and the first Myoxus prerelease; it is not a claim that the package has
been published on crates.io. GitHub availability and registry publication are
separate steps. See [release and registry notes](docs/RELEASING.md).

## Features and API documentation

The inherited features are `sync`, `weak` and `small-client`; the default
feature set is empty. `small-client` selects 32-bit client identifiers for
older peers. Select it according to your existing peer/protocol requirements.
`sync` requires observer callbacks to be `Send + Sync`; `weak` enables weak
references and quotations.

Build the fork's API documentation with `cargo doc --no-deps --all-features`.
Upstream documentation is useful background, but it does not describe all
fork-specific behavior.

## Development and maintenance

[Testing](docs/TESTING.md) covers the inherited tests, fork regressions and
checksum-verified upstream datasets. Allocator probes are an explicit manual
diagnostic and are excluded from normal tests, including `--all-features`.

Upstream updates must be reviewed against the documented changes and qualified
before adoption. Changes that become unnecessary should be removed. Maintainer
activity follows Myoxus needs; there is no promised release or support schedule.

## License and authorship

MIT, including the Myoxus adaptations. See [LICENSE](LICENSE).

- Original Yrs: Kevin Jahns and Bartosz Sypytkowski.
- Myoxus adaptations and maintenance: **Daniel Scharf**, copyright 2026.

The upstream copyright and permission notices remain intact. Test datasets
downloaded for qualification retain the attribution supplied with the datasets;
they are not bundled in the crate.
