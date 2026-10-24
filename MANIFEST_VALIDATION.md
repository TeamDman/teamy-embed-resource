# Manifest validation qualification

The default `manifest-discovery` feature uses the owned Facet TOML event
parser. `toml_parser` 1.0.10 validates syntax and event order but leaves
duplicate keys and table redefinitions to its caller. The local
`vendor/teamy-facet-toml/src/semantic.rs` checks those rules over the existing
event stream before Facet exposes it. It tracks decoded paths, implicit,
dotted and explicit tables, inline tables, and individual array-table
elements. No second TOML parse is used in the normal dependency graph.

The dev-only upstream `toml` 1.0.7 dependency is retained as a differential
comparator. The committed tests cover duplicate scalar and dotted keys,
quoted and escaped keys, table redefinition, inline tables and arrays, valid
implicit parent reopening, and array-table element identity. A generated
390,625-document matrix of four-statement combinations found and resolved
edge cases involving dotted keys, implicit parents and the latest array-table
element. The local
739-document corpus then had zero validity divergences when comparing the
raw Facet parser to upstream `toml` and zero validity or top-level `bin`
divergences through `contains_top_level_key`; 737 corpus documents were valid.
The generated matrix and local corpus paths are not published here.

On 24 October 2026, `cargo tree -e normal` showed 30 unique packages with
default features, 7 with `--no-default-features`, and 14 in the upstream
checkout. The lean graph contained no Facet, TOML or serde package. Separate
cold Linux `cargo check --lib` samples took 4.045 seconds default, 0.525
seconds lean, and 1.867 seconds upstream. These are one-run observations with
separate target directories, not a benchmark or a Windows speed claim.

Set `EMBED_RESOURCE_MANIFEST_CORPUS_LIST` to a newline-separated list of TOML
file paths to run the committed corpus comparison test. The test asserts both
document validity and top-level `bin` detection. This is Linux source and
parser qualification; Windows resource compiler behavior still needs separate
qualification.

The published `teamy-facet-toml` package does not yet exist. This local
source subset depends on the compatible published Teamy Facet stack. A future
crates.io release of this fork requires publishing a compatible owned parser
package or revisiting the packaging arrangement; that publication is not
authorized here.
