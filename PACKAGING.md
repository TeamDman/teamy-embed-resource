# Registry packaging checkpoint

The owned Facet TOML source is a separate package at
`vendor/teamy-facet-toml`. Its manifest pins the compatible published Teamy
Facet crates, `toml_parser` 1.0.10 and `toml_edit` 0.23.7. Its archive includes
the MIT and Apache license texts and [source provenance](vendor/teamy-facet-toml/UPSTREAM.md).
The parser package has a repository, README and standalone source archive. Its
manifest permits a future crates.io submission after authorization and a
registry name/owner check. It is a packaging candidate, not a published crate.
No crate has been published from this fork.

The root `embed-resource` archive does **not** contain the nested parser
package. Cargo strips the local path from its normalized manifest, leaving a
versioned dependency on `teamy-facet-toml`. A normal offline `cargo package`
for the root currently fails because that package is absent from the registry.
The existing `embed-resource` name and version `3.0.12` are also already used
by upstream on crates.io. The root manifest has `publish = false`; a future
owned registry identity/version decision is needed. Keep the original upstream
history, MIT notice and MPL-2.0 Teamy terms intact if that decision is made.

Run `python3 tests/package_smoke.py` from this checkout to create both archives
offline, verify the parser archive, and check extracted lean and default
consumers. The script supplies a temporary local Cargo patch for the
unpublished parser. That patch proves source packaging and dependency wiring;
it does not stand in for registry publication. Use `--allow-dirty` only while
reviewing uncommitted source. The lean extracted consumer compiles without a
TOML or Facet package in its actual normal/build dependency graph.

Windows resource compilation and linker behavior still need qualification
before production integration. Publishing either crate or merging this work
to `main` requires a separate decision.
