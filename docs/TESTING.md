# Testing

Use a current stable Rust toolchain. Python 3 is needed only to fetch upstream
test datasets; it is not a library dependency.

```sh
python3 scripts/fetch-test-assets.py
cargo test --locked --all-features
cargo package --locked
```

The first command fetches the upstream datasets used by the inherited editing
trace, compatibility and benchmark code. Downloads use a pinned upstream
commit and each file is checked against its recorded Git blob hash. Verified
files are reused. Assets are ignored by Git and excluded from the crate;
running the inherited dataset tests without fetching them fails visibly.

`tests/fork_regressions.rs` uses public library APIs and small synthetic values
to cover parent detachment, selective undo/redo, deterministic object encoding,
BigInt tags, v1/v2 document roundtrips and exact conversion boundaries. These
tests do not need external assets. The inherited library tests and doctests
provide wider behavioral coverage. `cargo package` verifies that the default
feature library builds from the actual package contents.

GitHub Actions runs the all-feature tests and package verification with an
explicit job timeout. Packaging is verification only; it does not publish to
crates.io.

## Explicit manual allocator qualification

Normal tests, including `--all-features`, do not compile or run the counting
allocator module. On Linux it can be enabled deliberately:

```sh
RUSTFLAGS="--cfg myoxus_memory_probes" cargo test --lib myoxus_ownership -- --test-threads=1
```

Run this diagnostic in a fresh isolated process. Compile outside the probe's
runtime limits; bound the resulting test process to a 90-second wall deadline,
60 CPU seconds and 1 GiB address space. The command above selects the tests
but does not install those process limits itself. Do not add it to normal CI.

The probe counts requested live allocations, not RSS or retained allocator
arenas. Weak-store checks cover actual parent and child store release. The
repeated materialization case uses 1 MiB synthetic text, two warm-up calls and
twelve measured calls, with at most 256 KiB extra retained bookkeeping memory.
It does not measure whole-application memory behavior.
