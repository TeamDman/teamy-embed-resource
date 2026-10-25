# Registry packaging and qualification

The owned Facet TOML source is a separate package at
`vendor/teamy-facet-toml`. Its manifest pins the compatible published Teamy
Facet crates, `toml_parser` 1.0.10 and `toml_edit` 0.23.7. Its archive includes
the MIT and Apache license texts and [source provenance](vendor/teamy-facet-toml/UPSTREAM.md).
The parser package has a repository, README and standalone source archive.
Publish and verify `teamy-facet-toml` 0.50.0-rc.7 before the root crate so
default-feature consumers can resolve its registry dependency.

The root `teamy-embed-resource` archive does **not** contain the nested parser
package. Cargo strips the local path from its normalized manifest, leaving a
versioned dependency on `teamy-facet-toml`. The root package has the owned
identity and initial version
`teamy-embed-resource` `0.1.0`, with `[lib] name = "embed_resource"` for source
compatibility. Cargo consumers can declare the dependency as
`embed-resource = { package = "teamy-embed-resource", version = "0.1.0" }`
after its release. The root manifest permits crates.io only. Upload remains
conditional on Windows qualification and a resolvable parser registry release.

Run `python3 tests/package_smoke.py` from this checkout to create both archives,
verify the parser archive, and check extracted lean and default consumers. The
script supplies a temporary local Cargo patch for the parser, so it proves
source packaging and dependency wiring rather than registry availability. Use
`--allow-dirty` only while reviewing uncommitted source. The lean extracted
consumer should compile without a TOML or Facet package in its actual
normal/build dependency graph.

After publication, verify each registry checksum and compile a fresh consumer
against the downloaded packages without a local patch. Qualify Windows resource
compilation and linker behavior before template adoption. Repository branch
publication and merges are separate decisions.
