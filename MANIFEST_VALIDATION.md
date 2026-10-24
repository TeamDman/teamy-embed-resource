# Manifest validation qualification

The default `manifest-discovery` feature uses the owned Facet TOML event
parser. `toml_parser` 1.0.10 checks event structure; the local intake rejects
error events, decodes every key, scalar, comment and newline with an error
sink, and checks duplicate keys and table definitions. The raw decoder
classifies dates and times without checking calendar ranges, so the existing
`toml_edit` 0.23.7 dependency validates **date-time scalar values only**.
This is one document parse plus isolated date-time value validation. It is
still an interim dependency, and the default graph remains larger than
upstream's.

The earlier full-document `toml_edit` gate rejected an upstream-valid nested
case (`[a.b.c]`, then `[a]`, then `b.a=1`). Restricting it to date-time
scalars preserves that table behavior while rejecting invalid dates such as
`2024-13-01`. No new dependency was added. `toml` 1.0.7 is a dev-only
differential comparator.

Committed tests cover duplicate and dotted keys, explicit and implicit
tables, inline table sealing, separate array-table elements, escaped keys,
malformed values and controls, and top-level `bin` detection. The opt-in
390,625-document matrix combines four statements from a fixed set of shallow
scalar and table forms; it found table-state edge cases but does not exercise
arbitrary TOML values. A local 739-document corpus matched upstream for
validity and top-level `bin`; 737 documents were valid. Additional local
50,000-case scalar and 12,299-case date-time differential probes found no
validity divergence. These are bounded samples, not a proof for every TOML
document. The corpus paths and private review material are not published.

Set `EMBED_RESOURCE_MANIFEST_CORPUS_LIST` to a newline-separated list of TOML
files to run the corpus test, and `EMBED_RESOURCE_SEMANTIC_MATRIX=1` to run the
generated matrix. Windows resource compilation still needs separate
qualification.

On 24 October 2026, `cargo tree -e normal` showed 34 unique packages with
default features and 7 with `--no-default-features`; upstream had 14. The
lean graph contained no Facet, TOML or serde package. The root lock retains
dev-only comparator packages; inspect the normal graph for build dependency
claims. Separate cold Linux `cargo check --lib` samples took 3.981 seconds
default, 0.542 seconds lean, and 1.867 seconds upstream. These are single
observations, not a speed benchmark or a Windows claim. A future crates.io
release requires publishing a compatible owned
`teamy-facet-toml` package or revisiting this local source arrangement. No
package publication is authorized here.
