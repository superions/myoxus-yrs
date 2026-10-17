# Releases and crates.io

The repository prepares `myoxus-yrs` version `0.28.0-myoxus.1`, based on Yrs
0.28.0. The suffix marks a Myoxus prerelease. Consumers must opt into a
prerelease explicitly. No registry upload is performed by the test workflow.

## What registry publication adds

A crates.io release lets consumers install a versioned package through the
normal Cargo registry and discover it through Cargo search and crates.io.
docs.rs normally builds API documentation for published Rust packages.
Consumers can use:

```toml
# After this version has actually been published:
[dependencies]
yrs = { package = "myoxus-yrs", version = "=0.28.0-myoxus.1" }
```

GitHub dependencies are already usable without a registry release. crates.io
adds distribution and discovery; it does not certify quality or compatibility,
and it does not automatically replace transitive dependencies on `yrs`.

## Publication requirements

Publishing needs an available crate name, a crates.io account with a verified
email address, and authorized registry credentials. Keep credentials outside
the repository. This preparation does not create a registry account or token.

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
