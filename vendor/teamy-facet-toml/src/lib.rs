//! Deserialization subset of TeamDman's Facet TOML source.

extern crate alloc;

mod parser;

pub use facet_format::DeserializeError;
pub use parser::TomlParser;

/// Check for a top-level key while validating the complete TOML document.
///
/// This preserves the value-shape-agnostic behavior of `toml::Table::contains_key`.
pub fn contains_top_level_key(input: &str, searched: &str) -> Result<bool, String> {
    use facet_format::{FormatParser, ParseEventKind};

    // Interim: Facet's event parser accepts some TOML semantic errors,
    // including duplicate keys. Validate first with toml_edit's serde-free
    // parser until semantic checks move into the Facet event intake.
    let _: toml_edit::DocumentMut = input.parse().map_err(|e: toml_edit::TomlError| e.to_string())?;
    let mut parser = TomlParser::new(input).map_err(|e| e.to_string())?;
    let mut depth = 0usize;
    let mut found = false;
    while let Some(event) = parser.next_event().map_err(|e| e.to_string())? {
        match event.kind {
            ParseEventKind::StructStart(_) | ParseEventKind::SequenceStart(_) => depth += 1,
            ParseEventKind::StructEnd | ParseEventKind::SequenceEnd => depth -= 1,
            ParseEventKind::FieldKey(key) if depth == 1 && key.name().is_some_and(|name| name == searched) => found = true,
            _ => {}
        }
    }
    Ok(found)
}

/// Deserialize a TOML document into an owned Facet type.
pub fn from_str<T>(input: &str) -> Result<T, DeserializeError>
where
    T: facet_core::Facet<'static>,
{
    use facet_format::FormatDeserializer;

    let mut parser = TomlParser::new(input)?;
    let mut deserializer = FormatDeserializer::new_owned(&mut parser);
    deserializer.deserialize_deferred()
}

#[cfg(test)]
mod differential_tests {
    use super::contains_top_level_key;

    fn assert_matches_upstream(input: &str) {
        let upstream = input.parse::<toml::Table>();
        let facet = contains_top_level_key(input, "bin");
        assert_eq!(upstream.is_ok(), facet.is_ok(), "validity mismatch for {input:?}");
        if let (Ok(table), Ok(found)) = (upstream, facet) {
            assert_eq!(table.contains_key("bin"), found, "key mismatch for {input:?}");
        }
    }

    #[test]
    fn table_and_key_semantics_match_upstream() {
        for input in [
            "bin=[]\nbin=[]",
            "[package]\nname=\"a\"\nname=\"b\"",
            "[a]\nx=1\n[a]\ny=2",
            "[a.b]\nx=1\n[a]\ny=2",
            "a=1\n[a]\nx=2",
            "[a]\nx=1\na=2",
            "a.b=1\n[a]\nc=2",
            "[a]\nx=1\na.b=2",
            "[[a]]\nx=1\n[[a]]\nx=2",
            "[[a]]\nx=1\nx=2",
            "a={x=1,x=2}",
            "a.b=1\na.b=2",
            "\"a\"=1\na=2",
            "[[a]]\nx=1\n[a]\ny=2",
            "[a]\nx=1\n[[a]]\ny=2",
            "bin = [{ name = \"tool\" }]",
            "\"bin\" = [1]",
            "[bin]\nname = \"tool\"",
            "bin.name = \"tool\"",
            "bin = \"tool\"",
            "[package.bin]\nname = \"nested\"",
        ] {
            assert_matches_upstream(input);
        }
    }

    #[test]
    fn local_manifest_corpus_matches_upstream_when_provided() {
        let Ok(list_path) = std::env::var("EMBED_RESOURCE_MANIFEST_CORPUS_LIST") else {
            return;
        };
        let paths = std::fs::read_to_string(list_path).unwrap();
        let mut count = 0usize;
        for path in paths.lines().filter(|line| !line.is_empty()) {
            let source = std::fs::read_to_string(path).unwrap();
            assert_matches_upstream(&source);
            count += 1;
        }
        assert!(count > 0, "corpus list was empty");
        eprintln!("compared {count} TOML documents with upstream");
    }
}
