# Releases and crates.io

`myoxus-yrs` version [0.28.0-myoxus.1](https://crates.io/crates/myoxus-yrs/0.28.0-myoxus.1)
was published on 2026-10-17, based on Yrs 0.28.0. The suffix marks a Myoxus
prerelease. Consumers must opt into a prerelease explicitly. No registry upload
is performed by the test workflow.

The published archive comes from release commit
`e94cdc97212f719a2f4b2b8c1b8fee391c349ec8`, tagged `v0.28.0-myoxus.1`. Its SHA-256
is `a58aeb8433e47a73bc37f84f9ef40db7fe9d1cd413cb0e70303677372eac8de2`.
Later repository documentation edits do not change this immutable package.

## What registry publication adds

A crates.io release lets consumers install a versioned package through the
normal Cargo registry and discover it through Cargo search and crates.io.
docs.rs normally builds API documentation for published Rust packages.
Consumers can use:

```toml
[dependencies]
yrs = { package = "myoxus-yrs", version = "=0.28.0-myoxus.1" }
```

GitHub dependencies are already usable without a registry release. crates.io
adds distribution and discovery; it does not certify quality or compatibility,
and it does not automatically replace transitive dependencies on `yrs`.

## Publication requirements

Publishing needs an available crate name, a crates.io account with a verified
email address, and authorized registry credentials. Keep credentials outside
the repository.

Before the first upload, review the license, authors, package contents,
changelog and compatibility notes; run the documented tests and
`cargo publish --dry-run --locked`. Publish only the exact reviewed commit
and create a matching version tag.

Published version contents cannot be replaced. A correction needs a new
version; yanking prevents new resolution of a bad version while preserving
existing lockfile builds. The registry has a limited deletion policy, so do
not treat an upload as a disposable draft.

Upstream-based version numbers describe lineage. They do not promise the same
API as upstream. Each release must state its Yrs baseline and the changes to
the previous Myoxus release. Check prerelease resolution behavior before
moving to a stable release or changing the version scheme.

References: [Cargo publishing](https://doc.rust-lang.org/cargo/reference/publishing.html),
[package aliases](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#renaming-dependencies-in-cargotoml),
[docs.rs builds](https://docs.rs/about/builds).
