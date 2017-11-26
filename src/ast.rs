#[cfg(feature="no_position")]
use serde::ser::{Serialize, Serializer, SerializeMap};

/**
 * Element types used in the abstract syntax tree (AST).
 *
 * Each element must keep track of its position in the original
 * input document. After parsing, the document tree can be serialized by serde.
 */
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(tag="type", rename_all="lowercase", deny_unknown_fields)]
pub enum Element {
    Document {position: Span, content: Vec<Element>},
    Heading {position: Span, depth: usize, caption: Vec<Element>, content: Vec<Element>},
    Text {position: Span, text: String},
    Formatted {position: Span, markup: MarkupType, content: Vec<Element>},
    Paragraph {position: Span, content: Vec<Element>},
    Template {position: Span, name: String, content: Vec<Element>},
    TemplateArgument {position: Span, name: String, value: Vec<Element>},
    InternalReference {position: Span, target: Vec<Element>, options: Vec<Vec<Element>>, caption: Vec<Element>},
    ExternalReference {position: Span, target: String, caption: Vec<Element>},
    ListItem {position: Span, depth: usize, kind: ListItemKind, content: Vec<Element>},
    List {position: Span, content: Vec<Element>},
    Table {position: Span, attributes: Vec<TagAttribute>, caption: Vec<Element>, caption_attributes: Vec<TagAttribute>, rows: Vec<Element>},
    TableRow {position: Span, attributes: Vec<TagAttribute>, cells: Vec<Element>},
    TableCell {position: Span, header: bool, attributes: Vec<TagAttribute>, content: Vec<Element>},
    Comment {position: Span, text: String},
    HtmlTag {position: Span, name: String, attributes: Vec<TagAttribute>, content: Vec<Element>},
}

/// Types of markup a section of text may have.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all="lowercase")]
pub enum MarkupType {
    NoWiki,
    Bold,
    Italic,
    Math,
    StrikeThrough,
    Underline,
    Code,
    Blockquote,
    Preformatted,
}


/// Types of markup a section of text may have.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all="lowercase")]
pub enum ListItemKind {
    Unordered,
    Definition,
    DefinitionTerm,
    Ordered
}

/**
 * Represents a position in the source document.
 *
 * The PartialEq implementation allows for a "any" position (all zero), which is
 * equal to any other position. This is used to reduce clutter in tests, where
 * a default Position ("{}") can be used where the actual representation is irrelevant.
 */
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all="lowercase", default="Position::any_position", deny_unknown_fields)]
pub struct Position {
    pub offset: usize,
    pub line: usize,
    pub col: usize,
}

/// Holds position information (start and end) for one element
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[cfg_attr(not(feature="no_position"), derive(Serialize))]
#[serde(rename_all="lowercase", default="Span::any", deny_unknown_fields)]
pub struct Span {
    pub start: Position,
    pub end: Position
}

/// Represents a pair of html tag attribute and value.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all="lowercase", deny_unknown_fields)]
pub struct TagAttribute {
    pub position: Span,
    pub key: String,
    pub value: String,
}

/// Position of a source line of code.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SourceLine<'input> {
    pub start: usize,
    pub content: &'input str,
    pub end: usize,
}

/// Match an HTML tag name to it's markup type
pub fn get_markup_by_tag_name(tag: &str) -> MarkupType {
    match &tag.to_lowercase()[..] {
        "math" => MarkupType::Math,
        "del" => MarkupType::StrikeThrough,
        "s" => MarkupType::StrikeThrough,
        "nowiki" => MarkupType::NoWiki,
        "u" => MarkupType::Underline,
        "ins" => MarkupType::Underline,
        "code" => MarkupType::Code,
        "blockquote" => MarkupType::Blockquote,
        "pre" => MarkupType::Preformatted,
        _ => panic!("markup type lookup not implemented for {}!", tag),
    }
}

/** Compiles a list of start and end positions of the input source lines.
 *
 * This representation is used to calculate line and column position from the input offset.
 */
pub fn get_source_lines<'input>(source: &'input str) -> Vec<SourceLine> {

    let mut pos = 0;
    let mut result = Vec::new();

    for line in source.split("\n") {
        result.push( SourceLine {
            start: pos,
            content: line,
            end: pos + line.len() + 1,
        });
        pos += line.len() + 1;
    }
    result
}

impl Position {
    pub fn new(offset: usize, slocs: &Vec<SourceLine>) -> Self {
        for (i, sloc) in slocs.iter().enumerate() {
            if offset >= sloc.start && offset < sloc.end {
                return Position {
                    offset: offset,
                    line: i + 1,
                    col: sloc.content[0..offset - sloc.start].chars().count() + 1,
                }
            }
        }
        Position {offset: offset, line: slocs.len() + 1, col: 0}
    }

    pub fn any_position() -> Self {
        Position {
            offset: 0,
            line: 0,
            col: 0,
        }
    }
}

impl Span {
    pub fn any() -> Self {
        Span {
            start: Position::any_position(),
            end: Position::any_position(),
        }
    }

    pub fn new(posl: usize, posr: usize, source_lines: &Vec<SourceLine>) -> Self {
        Span {
            start: Position::new(posl, source_lines),
            end: Position::new(posr, source_lines),
        }
    }
}

#[cfg(feature="no_position")]
impl Serialize for Span {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        let map = serializer.serialize_map(None)?;
        map.end()
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Position) -> bool {
        // comparing with "any" position is always true
        if (other.offset == 0 && other.line == 0 && other.col == 0) ||
           (self.offset == 0 && self.line == 0 && self.col == 0) {
            return true;
        }

        return self.offset == other.offset && self.line == other.line && self.col == other.col;
    }

    fn ne(&self, other: &Position) -> bool {!self.eq(other)}
}

impl TagAttribute {
    pub fn new(position: Span, key: String, value: String) -> Self {
        TagAttribute {
            position: position,
            key: key,
            value: value,
        }
    }
}
