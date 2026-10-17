# Upstream source and attribution

- Project: [Yrs / y-crdt](https://github.com/y-crdt/y-crdt).
- Base package: `yrs` 0.28.0.
- Published archive: <https://static.crates.io/crates/yrs/yrs-0.28.0.crate>.
- Archive SHA-256: `52c70dc8beca8666c77612a96889106ca3cd65318609721f464624ff79685da9`.
- Commit recorded by the archive:
  `23b7f5693bbf9e7d26340c521ee8647f79bdfba2`.
- Original license: MIT.
- Original authors: Kevin Jahns and Bartosz Sypytkowski.
- Myoxus adaptations: Daniel Scharf, copyright 2026, also under MIT.

`upstream/yrs-0.28.0.toml` and `upstream/yrs-0.28.0-vcs.json` retain the
unmodified manifest and VCS record from the published package. They describe
the upstream baseline, not the current package identity.

The repository begins with a source-only import of that checksum-verified
archive, followed by the maintained library changes and standalone packaging.
It does not import the private Myoxus Git history. The original MIT notice was
retained from the previously vendored Yrs source because the 0.28.0 archive
does not include a license file. Its authors' notice is preserved in LICENSE.

The additional test datasets are fetched from the exact upstream commit.
`upstream/test-assets.json` records each expected Git blob identifier, verified
by the fetcher before a downloaded file is used. Associated dataset attribution
is fetched with the datasets. These large files stay outside version control
and the published crate.

For each future upstream update, record the release archive checksum, review
the source delta, remove superseded fixes where supported by regressions, and
rerun the affected standalone tests. A new upstream release does not by itself
prove that a fork-specific fix is obsolete.
