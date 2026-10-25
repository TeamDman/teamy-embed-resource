//! Semantic table checks for the existing `toml_parser` event stream.
//!
//! `toml_parser` validates syntax, but leaves duplicate assignments and table
//! definitions to its caller. The event stream also accepts some malformed
//! scalar and key forms, so this pass is not full TOML validation. The
//! top-level manifest probe keeps a separate full-document validation gate.
//! The small private state types stay together because their transitions form
//! one validation pass and are reviewed as a unit.

use alloc::borrow::Cow;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use facet_format::{DeserializeErrorKind, ParseError};
use toml_parser::parser::{Event, EventKind};
use toml_parser::{Raw, Span};

#[derive(Clone, Copy, PartialEq, Eq)]
enum TableKind {
    Root,
    Implicit,
    Dotted,
    Explicit,
    ArrayElement,
    Inline,
}

enum Entry {
    Value,
    Table(Table),
    ArrayOfTables(Vec<Table>),
}

struct Table {
    kind: TableKind,
    entries: BTreeMap<String, Entry>,
}

impl Table {
    fn new(kind: TableKind) -> Self {
        Self {
            kind,
            entries: BTreeMap::new(),
        }
    }
}

struct Key {
    name: String,
    span: Span,
}

struct Step {
    name: String,
    array_index: Option<usize>,
}

fn error(key: &Key, message: &str) -> ParseError {
    ParseError::new(
        facet_reflect::Span::new(key.span.start(), key.span.len()),
        DeserializeErrorKind::InvalidValue {
            message: message.to_string().into(),
        },
    )
}

struct Cursor<'a> {
    input: &'a str,
    events: &'a [Event],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn skip_trivia(&mut self) {
        while self
            .events
            .get(self.pos)
            .is_some_and(|event| matches!(event.kind(), EventKind::Whitespace | EventKind::Comment | EventKind::Newline))
        {
            self.pos += 1;
        }
    }

    fn peek(&mut self) -> Option<&'a Event> {
        self.skip_trivia();
        self.events.get(self.pos)
    }

    fn next(&mut self) -> Option<&'a Event> {
        self.skip_trivia();
        let event = self.events.get(self.pos);
        if event.is_some() {
            self.pos += 1;
        }
        event
    }

    fn take(&mut self, kind: EventKind) {
        if self.peek().is_some_and(|event| event.kind() == kind) {
            self.next();
        }
    }

    fn key_path(&mut self) -> Vec<Key> {
        let mut path = Vec::new();
        while let Some(event) = self.peek() {
            match event.kind() {
                EventKind::SimpleKey => {
                    let span = event.span();
                    let raw = Raw::new_unchecked(&self.input[span.start()..span.end()], event.encoding(), span);
                    let mut decoded = Cow::Borrowed("");
                    raw.decode_key(&mut decoded, &mut ());
                    path.push(Key {
                        name: decoded.into_owned(),
                        span,
                    });
                    self.next();
                }
                EventKind::KeySep => {
                    self.next();
                }
                _ => break,
            }
        }
        path
    }

    fn value(&mut self, depth: usize) -> Result<(), ParseError> {
        // The syntax parser has already applied the same recursion limit.
        debug_assert!(depth <= 128);
        let Some(event) = self.next() else { return Ok(()) };
        match event.kind() {
            EventKind::Scalar => {}
            EventKind::ArrayOpen => loop {
                match self.peek().map(|event| event.kind()) {
                    Some(EventKind::ArrayClose) => {
                        self.next();
                        break;
                    }
                    Some(EventKind::ValueSep) => {
                        self.next();
                    }
                    Some(_) => self.value(depth + 1)?,
                    None => break,
                }
            },
            EventKind::InlineTableOpen => {
                let mut table = Table::new(TableKind::Inline);
                loop {
                    match self.peek().map(|event| event.kind()) {
                        Some(EventKind::InlineTableClose) => {
                            self.next();
                            break;
                        }
                        Some(EventKind::SimpleKey) => {
                            let path = self.key_path();
                            self.take(EventKind::KeyValSep);
                            self.value(depth + 1)?;
                            assign(&mut table, &path)?;
                        }
                        Some(_) => {
                            self.next();
                        }
                        None => break,
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

fn table_at<'a>(mut table: &'a mut Table, path: &[Step]) -> &'a mut Table {
    for step in path {
        table = match table.entries.get_mut(&step.name).expect("validated table path") {
            Entry::Table(child) => child,
            Entry::ArrayOfTables(elements) => &mut elements[step.array_index.expect("array element index")],
            Entry::Value => unreachable!("validated table path cannot enter a value"),
        };
    }
    table
}

fn assign(mut table: &mut Table, path: &[Key]) -> Result<(), ParseError> {
    let Some((last, prefixes)) = path.split_last() else {
        return Ok(());
    };
    for key in prefixes {
        let entry = table
            .entries
            .entry(key.name.clone())
            .or_insert_with(|| Entry::Table(Table::new(TableKind::Dotted)));
        table = match entry {
            Entry::Table(child) if matches!(child.kind, TableKind::Dotted | TableKind::Implicit) => {
                // A dotted assignment turns an implicit header parent into
                // a dotted table, so a later [table] cannot define it.
                if child.kind == TableKind::Implicit {
                    child.kind = TableKind::Dotted;
                }
                child
            }
            Entry::ArrayOfTables(elements) => elements.last_mut().expect("array table always has a current element"),
            _ => return Err(error(key, "duplicate key or value extended by dotted key")),
        };
    }
    // A dotted assignment may descend through the latest array-table element
    // only to define a nested dotted table. Assigning directly into that
    // element from a different table context would redefine its table form.
    if !prefixes.is_empty() && table.kind == TableKind::ArrayElement {
        return Err(error(last, "dotted key redefines an array-table element"));
    }
    if table.entries.contains_key(&last.name) {
        return Err(error(last, "duplicate key"));
    }
    table.entries.insert(last.name.clone(), Entry::Value);
    Ok(())
}

fn open_table(root: &mut Table, path: &[Key], is_array: bool) -> Result<Vec<Step>, ParseError> {
    let Some((last, prefixes)) = path.split_last() else {
        return Ok(Vec::new());
    };
    let mut table = root;
    let mut steps = Vec::with_capacity(path.len());
    for key in prefixes {
        let entry = table
            .entries
            .entry(key.name.clone())
            .or_insert_with(|| Entry::Table(Table::new(TableKind::Implicit)));
        table = match entry {
            Entry::Table(child) => {
                steps.push(Step {
                    name: key.name.clone(),
                    array_index: None,
                });
                child
            }
            Entry::ArrayOfTables(elements) => {
                let index = elements.len() - 1;
                steps.push(Step {
                    name: key.name.clone(),
                    array_index: Some(index),
                });
                &mut elements[index]
            }
            Entry::Value => return Err(error(key, "cannot extend a value with a table header")),
        };
    }

    let entry = table.entries.entry(last.name.clone());
    let array_index = if is_array {
        match entry {
            alloc::collections::btree_map::Entry::Vacant(slot) => {
                slot.insert(Entry::ArrayOfTables(vec![Table::new(TableKind::ArrayElement)]));
                Some(0)
            }
            alloc::collections::btree_map::Entry::Occupied(mut slot) => match slot.get_mut() {
                Entry::ArrayOfTables(elements) => {
                    let index = elements.len();
                    elements.push(Table::new(TableKind::ArrayElement));
                    Some(index)
                }
                _ => return Err(error(last, "duplicate table")),
            },
        }
    } else {
        match entry {
            alloc::collections::btree_map::Entry::Vacant(slot) => {
                slot.insert(Entry::Table(Table::new(TableKind::Explicit)));
            }
            alloc::collections::btree_map::Entry::Occupied(mut slot) => match slot.get_mut() {
                Entry::Table(child) if child.kind == TableKind::Implicit => {
                    child.kind = TableKind::Explicit;
                }
                _ => return Err(error(last, "duplicate table")),
            },
        }
        None
    };
    steps.push(Step {
        name: last.name.clone(),
        array_index,
    });
    Ok(steps)
}

pub(crate) fn validate(input: &str, events: &[Event]) -> Result<(), ParseError> {
    let mut cursor = Cursor { input, events, pos: 0 };
    let mut root = Table::new(TableKind::Root);
    let mut current = Vec::new();

    while let Some(kind) = cursor.peek().map(|event| event.kind()) {
        match kind {
            EventKind::StdTableOpen | EventKind::ArrayTableOpen => {
                let is_array = kind == EventKind::ArrayTableOpen;
                cursor.next();
                let path = cursor.key_path();
                cursor.take(if is_array { EventKind::ArrayTableClose } else { EventKind::StdTableClose });
                current = open_table(&mut root, &path, is_array)?;
            }
            EventKind::SimpleKey => {
                let path = cursor.key_path();
                cursor.take(EventKind::KeyValSep);
                cursor.value(0)?;
                assign(table_at(&mut root, &current), &path)?;
            }
            _ => {
                cursor.next();
            }
        }
    }
    Ok(())
}
