# teamy-facet-toml

This is the manifest-reading subset of the owned Facet TOML source used by
`embed-resource`. It uses the published, compatible Teamy Facet packages and
validates TOML tokens and table definitions before exposing Facet events.
The existing `toml_edit` dependency validates date-time scalar ranges only.

Its package archive can be built and checked locally. The manifest permits a
future crates.io submission, but no registry release is authorized yet and
the registry name/owner must still be confirmed at publication time.
The root `embed-resource` package requires a registry release of this crate
before its own registry package can resolve without a local patch.

See [UPSTREAM.md](UPSTREAM.md) for the exact parser source revision and
licensing. The parser source and local adapter use MIT OR Apache-2.0; both
license texts are included in this package.
