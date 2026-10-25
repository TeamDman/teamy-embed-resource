# teamy-facet-toml

This is the manifest-reading subset of the owned Facet TOML source used by
`embed-resource`. It uses the published, compatible Teamy Facet packages and
validates TOML tokens and table definitions before exposing Facet events.
The existing `toml_edit` dependency validates date-time scalar ranges only.

Its package archive can be built and checked independently. The
`teamy-embed-resource` default feature requires this registry package; publish
and verify version 0.50.0-rc.7 before releasing the root crate.

See [UPSTREAM.md](UPSTREAM.md) for the exact parser source revision and
licensing. The parser source and local adapter use MIT OR Apache-2.0; both
license texts are included in this package.
