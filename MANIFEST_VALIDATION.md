# Manifest validation follow-up

The current `manifest-discovery` implementation is an **interim two-pass
path**. `toml_edit` 0.23.7 validates TOML semantics, then the owned Facet TOML
event parser checks whether the top-level `bin` key exists. Both are absent
when default features are disabled. `compile_for_everything()` never inspects
the manifest.

The second pass is needed because `toml_parser` 1.0.10 validates event order
and syntax, but explicitly leaves semantic checks such as duplicate keys to
its caller. A direct Facet event scan accepts repeated keys, repeated table
headers, scalar/table conflicts, duplicate inline-table keys, and some
array-table conflicts that the upstream `toml::Table` rejects. Conversely,
reopening an implicit parent table and using the same key in separate array
table elements are valid. A flat set of seen key names would break those
cases.

The focused single-pass correction belongs in the owned Facet TOML parser's
event intake. Build a semantic table state keyed by decoded TOML paths, with
explicit versus implicit and dotted table status and a distinct state for
each array-table element. Validate key assignments and table headers against
that state while consuming the existing `toml_parser` event stream. Return a
span-bearing error before exposing events. This keeps the existing resource
compiler and Facet event conversion intact. Remove `toml_edit` only after the
differential gate passes for all targeted cases and the local manifest corpus.

`vendor/teamy-facet-toml` has a committed differential test against upstream
`toml` 1.0.7 as a **dev-only** dependency. It always runs targeted table and
key cases. Set `EMBED_RESOURCE_MANIFEST_CORPUS_LIST` to a newline-separated
list of local TOML file paths to run a larger corpus; the 24 October 2026
cloud review compared 739 documents with zero validity or key-result
divergences. The source files in that corpus are not copied into this repo.

`toml_edit` 0.23.7 was already available in the cloud registry cache, but is
new to this fork's normal dependency graph; it was not in the upstream
`embed-resource` lock. The owned Facet-format workspace instead locks
`toml_edit` 0.25.13. Version 0.23.7 shares `toml_parser` 1.0.x with the Facet
source. It adds four unique normal-graph packages here: `toml_edit`,
`indexmap`, `toml_datetime`, and `winnow` 0.7. The current package graph has
35 packages with the default feature and 7 without it. A single cold Linux
`cargo check --lib` run took 3.94 seconds default, 0.54 seconds lean, and
1.85 seconds for upstream; these are local samples, not a speed guarantee.
