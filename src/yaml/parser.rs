use std::any::type_name;

use marked_yaml::{
    LoadError::{
        MappingKeyMustBeScalar, ScanError, TopLevelMustBeMapping, UnexpectedAnchor, UnexpectedTag,
    },
    Node, Span,
};
use tower_lsp::lsp_types::{MessageType, SemanticToken, SemanticTokenType};

use crate::{
    documents::{get_index_for_type, DocumentInfo, ImCompleteSemanticToken},
    errors::error_registry::SyntaxError,
    utilities::positions_and_ranges::{CustomPosition, CustomRange},
    Backend,
};

pub fn get_start_and_length_from_span(node: &Node, source: &str) -> (usize, usize) {
    let span = node.span();
    let start = span
        .start()
        .map(|x| CustomPosition::from_marker(x).subtract_line(1).to_offset(source))
        .unwrap_or(0) as usize;
    let end = span
        .end()
        .map(|x| CustomPosition::from_marker(x).subtract_line(1).to_offset(source))
        .unwrap_or(0) as usize;
    (start - 1, if end > start { end - start } else { 1 })
}

pub fn node_length(node: &Node) -> usize {
    match node {
        Node::Scalar(scalar) => scalar.len(),
        Node::Sequence(sequence) => sequence.len(),
        Node::Mapping(mapping) => mapping.len(),
    }
}

pub fn node_type(node: &Node) -> String {
    match node {
        Node::Scalar(_) => "Scalar".to_string(),
        Node::Sequence(_) => "Sequence".to_string(),
        Node::Mapping(_) => "Mapping".to_string(),
    }
}

pub fn visit(backend: &Backend, doc: &mut DocumentInfo, node: Node) {
    // visiting node {} with span {}...{}
    backend.log(
        MessageType::INFO,
        format!(
            "visiting node of type {} with span {:?}...{:?} with length {}.",
            node_type(&node),
            node.span().start(),
            node.span().end(),
            node_length(&node),
        ),
    );
    match node {
        // string
        Node::Scalar(node) => doc.semantic_tokens.push(ImCompleteSemanticToken {
            start: get_start_and_length_from_span(&Node::Scalar(node.clone()), &doc.source.to_string()).0,
            token_type: get_index_for_type(SemanticTokenType::STRING),
            length: node.len(),
        }),
        // key-value pair
        Node::Mapping(mut node) => {
            // highlight the keys as properties
            node.entries().for_each(|entry| {
                let key = entry.key();
                let value = entry.get();

                doc.semantic_tokens.push(ImCompleteSemanticToken {
                    start: get_start_and_length_from_span(&Node::Scalar(key.clone()), &doc.source.to_string()).0,
                    length: key.len(),
                    token_type: get_index_for_type(SemanticTokenType::PROPERTY),
                });
                visit(backend, doc, value.clone());
            })
        }
        // array
        Node::Sequence(mut mode) => {
            mode.iter().for_each(|node| {
                visit(backend, doc, node.clone());
            })
        }
    }
}

pub fn parse<'a>(backend: &'a Backend, mut doc: &'a mut DocumentInfo) -> &'a DocumentInfo {
    let source = &doc.source.to_string();
    let node = marked_yaml::parse_yaml(0, source);
    if let Err(e) = node {
        // struct is LoadError(Marker)
        let mut range_start = match e {
            TopLevelMustBeMapping(marker)
            | UnexpectedAnchor(marker)
            | MappingKeyMustBeScalar(marker)
            | UnexpectedTag(marker)
            | ScanError(marker, _) => CustomPosition::from_marker(&marker),
        };
        // subtract 1 line because it's 1-indexed
        // subtract 1 character because it's 1-indexed
        range_start.set_line(range_start.line - 1);
        // range_start.set_character(range_start.character - 1);

        let mut message = e.to_string();

        // the message contains a "line:column: " at the start, remove the first 2 colons and the space
        message.drain(0..message.find(':').unwrap() + 1);
        message.drain(0..message.find(':').unwrap() + 1);
        message.drain(0..message.find(' ').unwrap() + 1);

        doc.diagnostics.push(
            SyntaxError::new(
                CustomRange::new(range_start, range_start.add_offset(1, source)),
                message,
            )
            .to_error()
            .to_diagnostic(),
        );
        return doc;
    }
    let node = node.unwrap();
    visit(backend, &mut doc, node);

    doc
}

// pub fn parse(mut doc: DocumentInfo) {
//     let yaml = YamlLoader::load_from_str(&doc.source.to_string());

//     if let Err(e) = yaml {
//         let range_start = CustomPosition::from_yaml_rust_marker(e.marker());
//         doc.diagnostics.push(
//             SyntaxError::new(
//                 CustomRange::new(range_start, range_start.add_offset(1, &doc.source.to_string())),
//                 "YAML parsing error".to_string(),
//             )
//             .to_error()
//             .to_diagnostic(),
//         );
//         return;
//     };

//     let binding = yaml.unwrap();
//     let yaml = binding.first();
//     if yaml.is_none() {
//         return;
//     }
//     let yaml = yaml.unwrap();

//     // get range
//     yaml;
// }
