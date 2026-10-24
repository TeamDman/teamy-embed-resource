# Facet TOML source

`src/parser.rs` is copied from `facet-toml/src/parser.rs` in
[`TeamDman/facet-format` at `642d268fc01eced0a861bded044342ff88733eec`](https://github.com/TeamDman/facet-format/tree/642d268fc01eced0a861bded044342ff88733eec/facet-toml),
which derives from `facet-rs/facet-format`. The local `lib.rs` retains the
deserialization entry point for the copied parser tests and adds a top-level
key probe for `embed-resource`. Serialization, Axum integration, and the
upstream crate's other dependencies are omitted.

The owned `TeamDman/facet-format` tree contains this parser source, but its
`facet-toml` package has not been published as `teamy-facet-toml` and its
workspace dependencies point to a sibling `../facet` checkout. A standalone
Git dependency on that tree would not be self-contained. The published
upstream `facet-toml` uses upstream Facet package identities, which would add
an incompatible second reflection stack. This local subset depends only on
the compatible published Teamy Facet packages and `toml_parser`. The local
`src/semantic.rs` validates duplicate keys and table definitions against the
same event stream that Facet consumes; Facet events supply the top-level key
probe. This parser is confined to the default-on `manifest-discovery` feature
of `embed-resource`. The upstream `toml` crate is a dev-only differential
comparator, not a normal dependency.

The copied source and local adapter glue remain licensed MIT OR Apache-2.0.
Both upstream license texts are included here. This local package is
deliberately unpublished. If a future crates.io release of `embed-resource` is
wanted, publish a compatible owned
`teamy-facet-toml` first or revisit this packaging arrangement.
