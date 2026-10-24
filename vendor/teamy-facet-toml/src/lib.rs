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

    // Facet's event parser accepts some TOML semantic errors, including
    // duplicate keys. Validate first with toml_edit's serde-free parser.
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
