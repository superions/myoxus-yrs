# Changelog

## 0.28.0-myoxus.1 — 2026-10-17

Initial standalone Myoxus fork based on Yrs 0.28.0.

- Break parent/subdocument ownership cycles while keeping orphan children usable.
- Preserve the other history stack during selective undo/redo clearing.
- Sort `Any::Map` keys during binary encoding.
- Preserve distinct BigInt values and reject inexact numeric boundary conversions.
- Replace `smallstr` with `compact_str` 0.10.0.
- Carry focused warning fixes without suppressing remaining diagnostics.
- Add the `myoxus-yrs` package identity, provenance, MIT attribution and standalone CI.

See [detailed changes](docs/CHANGES.md) for compatibility implications.
