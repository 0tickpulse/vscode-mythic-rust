use marked_yaml::Node::{self, Scalar};

use crate::documents::DocumentInfo;

pub trait YamlSchema {
    fn get_description(&self) -> String;
    fn validate(&self, doc: &mut DocumentInfo, node: &Node) -> bool {
        true
    }
}

pub struct YamlSchemaString {
    literal: Option<String>,
}

impl YamlSchemaString {
    pub fn new(literal: Option<String>) -> Self {
        Self { literal }
    }
}

impl YamlSchema for YamlSchemaString {
    fn get_description(&self) -> String {
        match &self.literal {
            Some(literal) => format!("\"{}\"", literal),
            None => "string".to_string(),
        }
    }
    fn validate(&self, doc: &mut DocumentInfo, node: &Node) -> bool {
        match node {
            Scalar(scalar) => {
                if let Some(literal) = &self.literal {
                    if scalar.to_string() != *literal {
                        return false;
                    }
                }
            }
            _ => return false,
        }
        true
    }
}
