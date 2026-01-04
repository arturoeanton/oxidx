//! Syntax Definition Module
//!
//! Provides data structures for defining language syntax rules
//! for syntax highlighting in code editors.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Defines syntax rules for a programming language.
///
/// Can be loaded from JSON files to support multiple languages
/// without recompiling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxDefinition {
    /// Language name (e.g., "Rust", "JavaScript")
    pub name: String,

    /// File extensions (e.g., ["rs"], ["js", "jsx"])
    pub extensions: Vec<String>,

    /// Language keywords (e.g., ["fn", "let", "mut"])
    pub keywords: Vec<String>,

    /// Built-in types (e.g., ["String", "i32", "Vec"])
    pub types: Vec<String>,

    /// Line comment prefix (e.g., "//")
    pub comment_line: String,

    /// String delimiter characters (e.g., ["\"", "'"])
    pub string_delimiters: Vec<String>,

    /// Optional: block comment start (e.g., "/*")
    #[serde(default)]
    pub comment_block_start: Option<String>,

    /// Optional: block comment end (e.g., "*/")
    #[serde(default)]
    pub comment_block_end: Option<String>,
}

impl SyntaxDefinition {
    /// Creates a new empty SyntaxDefinition.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            extensions: Vec::new(),
            keywords: Vec::new(),
            types: Vec::new(),
            comment_line: "//".to_string(),
            string_delimiters: vec!["\"".to_string()],
            comment_block_start: None,
            comment_block_end: None,
        }
    }

    /// Loads a SyntaxDefinition from a JSON file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, SyntaxError> {
        let content =
            fs::read_to_string(path.as_ref()).map_err(|e| SyntaxError::IoError(e.to_string()))?;
        Self::from_json(&content)
    }

    /// Parses a SyntaxDefinition from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, SyntaxError> {
        serde_json::from_str(json).map_err(|e| SyntaxError::ParseError(e.to_string()))
    }

    /// Serializes the definition to JSON.
    pub fn to_json(&self) -> Result<String, SyntaxError> {
        serde_json::to_string_pretty(self).map_err(|e| SyntaxError::ParseError(e.to_string()))
    }

    /// Checks if this definition matches a file extension.
    pub fn matches_extension(&self, ext: &str) -> bool {
        let ext_lower = ext.to_lowercase();
        self.extensions
            .iter()
            .any(|e| e.to_lowercase() == ext_lower)
    }

    /// Checks if a word is a keyword in this language.
    pub fn is_keyword(&self, word: &str) -> bool {
        self.keywords.contains(&word.to_string())
    }

    /// Checks if a word is a type in this language.
    pub fn is_type(&self, word: &str) -> bool {
        self.types.contains(&word.to_string())
    }

    /// Returns true if the given character starts a string.
    pub fn is_string_delimiter(&self, ch: char) -> bool {
        self.string_delimiters.iter().any(|s| s.starts_with(ch))
    }

    /// Returns the default Rust syntax definition.
    pub fn rust() -> Self {
        Self {
            name: "Rust".to_string(),
            extensions: vec!["rs".to_string()],
            keywords: vec![
                "fn", "let", "mut", "pub", "struct", "impl", "use", "mod", "crate", "super",
                "self", "Self", "const", "static", "enum", "trait", "type", "where", "for",
                "while", "loop", "if", "else", "match", "return", "break", "continue", "as", "in",
                "ref", "move", "async", "await", "dyn", "unsafe", "extern", "true", "false",
                "Some", "None", "Ok", "Err",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            types: vec![
                "String", "str", "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32",
                "u64", "u128", "usize", "f32", "f64", "bool", "char", "Vec", "Option", "Result",
                "Box", "Rc", "Arc", "Cell", "RefCell", "HashMap", "HashSet", "BTreeMap",
                "BTreeSet",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            comment_line: "//".to_string(),
            string_delimiters: vec!["\"".to_string()],
            comment_block_start: Some("/*".to_string()),
            comment_block_end: Some("*/".to_string()),
        }
    }

    /// Returns the default JavaScript syntax definition.
    pub fn javascript() -> Self {
        Self {
            name: "JavaScript".to_string(),
            extensions: vec!["js".to_string(), "jsx".to_string(), "mjs".to_string()],
            keywords: vec![
                "function",
                "var",
                "let",
                "const",
                "if",
                "else",
                "for",
                "while",
                "do",
                "switch",
                "case",
                "default",
                "break",
                "continue",
                "return",
                "try",
                "catch",
                "finally",
                "throw",
                "new",
                "delete",
                "typeof",
                "instanceof",
                "in",
                "of",
                "this",
                "class",
                "extends",
                "super",
                "import",
                "export",
                "from",
                "as",
                "async",
                "await",
                "yield",
                "true",
                "false",
                "null",
                "undefined",
                "NaN",
                "Infinity",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            types: vec![
                "Array", "Object", "String", "Number", "Boolean", "Symbol", "BigInt", "Map", "Set",
                "WeakMap", "WeakSet", "Date", "RegExp", "Error", "Promise", "Proxy", "JSON",
                "Math", "console",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            comment_line: "//".to_string(),
            string_delimiters: vec!["\"".to_string(), "'".to_string(), "`".to_string()],
            comment_block_start: Some("/*".to_string()),
            comment_block_end: Some("*/".to_string()),
        }
    }
}

/// Errors that can occur when loading syntax definitions.
#[derive(Debug, Clone)]
pub enum SyntaxError {
    /// IO error (file not found, permission denied, etc.)
    IoError(String),
    /// JSON parsing error
    ParseError(String),
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntaxError::IoError(msg) => write!(f, "IO error: {}", msg),
            SyntaxError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for SyntaxError {}
