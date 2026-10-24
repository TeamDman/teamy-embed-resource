# Facet TOML source

`src/parser.rs` is copied from `facet-toml/src/parser.rs` in
[`TeamDman/facet-format` at `642d268fc01eced0a861bded044342ff88733eec`](https://github.com/TeamDman/facet-format/tree/642d268fc01eced0a861bded044342ff88733eec/facet-toml),
which derives from `facet-rs/facet-format`. The local `lib.rs` exposes only the
deserialization entry point needed by `embed-resource`; serialization, Axum
integration, and the upstream crate's other dependencies are omitted.

The source remains licensed MIT OR Apache-2.0. Both upstream license texts are
included here. This local package is deliberately unpublished. If a future
crates.io release of `embed-resource` is wanted, publish a compatible owned
`teamy-facet-toml` first or revisit this packaging arrangement.
