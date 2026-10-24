//! Deserialization subset of TeamDman's Facet TOML source.

extern crate alloc;

mod parser;
mod semantic;

pub use facet_format::DeserializeError;
pub use parser::TomlParser;

/// Check for a top-level key while validating the complete TOML document.
///
/// This preserves the value-shape-agnostic behavior of `toml::Table::contains_key`.
pub fn contains_top_level_key(input: &str, searched: &str) -> Result<bool, String> {
    use facet_format::{FormatParser, ParseEventKind};

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
    use super::{TomlParser, contains_top_level_key};
    use facet_format::FormatParser;

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
            "a={b.c=1,b.d=2}",
            "a={b.c=1,b.c=2}",
            "a=[{x=1,x=2}]",
            "[a.b]\nx=1\n[a.c]\ny=2",
            "[[a]]\n[a.b]\nc=1",
            "[[b.a]]\n[b]\na.b.c=1",
            "[[b.a]]\n[b]\na.c=1",
            "[[a.b]]\n[a]\nb.a=1",
            "a.b.c=1\n[a.b]\nd=2",
            "a.b=1\na.c=2",
            "\"a\\u002eb\"=1\n\"a.b\"=2",
            "[a.b.c]\n[a]\nb.a=1",
            "[a.b.c]\n[a]\nb.a=1\n[a.b]",
            "[[a.b.c]]\n[a]\nb.a=1\n[a.b]",
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

    #[test]
    fn generated_semantic_matrix_matches_upstream_when_requested() {
        if std::env::var("EMBED_RESOURCE_SEMANTIC_MATRIX").as_deref() != Ok("1") {
            return;
        }
        let statements = [
            "a=1",
            "a.b=1",
            "a.b.c=1",
            "a.c=1",
            "b=1",
            "b.a=1",
            "a={b=1}",
            "a={b.c=1,b.d=2}",
            "a={b=1,b=2}",
            "a=[{b=1,b=2}]",
            "a=[]",
            "a=[1]",
            "a=[[1]]",
            "\"a\"=1",
            "\"a\\u002eb\"=1",
            "[a]",
            "[a.b]",
            "[a.b.c]",
            "[b]",
            "[b.a]",
            "[[a]]",
            "[[a.b]]",
            "[[a.b.c]]",
            "[[b]]",
            "[[b.a]]",
        ];
        let mut compared = 0usize;
        for first in statements {
            for second in statements {
                for third in statements {
                    for fourth in statements {
                        let input = format!("{first}\n{second}\n{third}\n{fourth}\n");
                        let upstream_valid = input.parse::<toml::Table>().is_ok();
                        let facet_valid = TomlParser::new(&input)
                            .and_then(|mut parser| {
                                while parser.next_event()?.is_some() {}
                                Ok(())
                            })
                            .is_ok();
                        assert_eq!(upstream_valid, facet_valid, "validity mismatch for {input:?}");
                        compared += 1;
                    }
                }
            }
        }
        assert_eq!(compared, 390_625);
    }
}
