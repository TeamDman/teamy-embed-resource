//! Deserialization subset of TeamDman's Facet TOML source.

extern crate alloc;

mod parser;

pub use facet_format::DeserializeError;
pub use parser::TomlParser;

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
