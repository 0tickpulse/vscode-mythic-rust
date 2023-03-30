use marked_yaml::Node;
use ropey::Rope;
use tower_lsp::lsp_types::{Diagnostic, Hover, SemanticToken, SemanticTokenType};

use crate::{utilities::positions_and_ranges::CustomRange, Backend};

pub const LEGEND_TYPE: &[SemanticTokenType] = &[
    SemanticTokenType::NAMESPACE,
    SemanticTokenType::TYPE,
    SemanticTokenType::CLASS,
    SemanticTokenType::ENUM,
    SemanticTokenType::INTERFACE,
    SemanticTokenType::STRUCT,
    SemanticTokenType::TYPE_PARAMETER,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::PROPERTY,
    SemanticTokenType::ENUM_MEMBER,
    SemanticTokenType::EVENT,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::METHOD,
    SemanticTokenType::MACRO,
    SemanticTokenType::KEYWORD,
    SemanticTokenType::MODIFIER,
    SemanticTokenType::COMMENT,
    SemanticTokenType::STRING,
    SemanticTokenType::NUMBER,
    SemanticTokenType::REGEXP,
    SemanticTokenType::OPERATOR,
];

pub fn get_index_for_type(token_type: SemanticTokenType) -> u32 {
    LEGEND_TYPE
        .iter()
        .position(|x| x == &token_type)
        .unwrap()
        .try_into()
        .unwrap()
}

#[derive(Debug, Clone)]
pub struct ImCompleteSemanticToken {
    pub start: usize,
    pub length: usize,
    pub token_type: u32,
}

/// Represents a cached document.
#[derive(Debug, Clone)]
pub struct DocumentInfo {
    pub source: Rope,
    pub yaml: Option<Node>,
    pub hovers: Vec<Hover>,
    pub diagnostics: Vec<Diagnostic>,
    pub semantic_tokens: Vec<ImCompleteSemanticToken>,
}

impl DocumentInfo {
    pub fn new(source: Rope, yaml: Option<Node>) -> Self {
        Self {
            source,
            yaml,
            hovers: Vec::new(),
            diagnostics: Vec::new(),
            semantic_tokens: Vec::new(),
        }
    }
}
