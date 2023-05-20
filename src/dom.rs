use core::fmt;
use std::collections::HashMap;

use crate::{
    css::{self, StyleSheet},
    html,
};

#[derive(Debug, Clone)]
pub enum AttrValue {
    Text(String),
    Implicit,
}

#[derive(Debug, Default, Clone)]
pub struct AttrMap(pub HashMap<String, AttrValue>);

impl fmt::Display for AttrMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let i = self
            .0
            .iter()
            .map(|(key, value)| match &value {
                AttrValue::Text(text) => format!("{}=\"{}\"", key, text),
                AttrValue::Implicit => key.to_string(),
            })
            .collect::<Vec<String>>();
        write!(f, "{}", i.join(" "))
    }
}

#[derive(Debug, Clone)]
pub struct ElementData {
    tag_name: String,
    attributes: AttrMap,
    child_nodes: Vec<Node>,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Element(ElementData),
    Text(String),
    Comment(String),
    Document(DocumentData),
}

#[derive(Debug, Clone)]
pub struct Node {
    pub node_type: NodeType,
}

impl Node {
    fn pretty_print(&self, f: &mut fmt::Formatter<'_>, indent: usize) {
        let prepadding = "  ".repeat(indent);
        match &self.node_type {
            NodeType::Element(data) => {
                write!(f, "{}<{}", prepadding, data.tag_name).unwrap();

                if data.attributes.0.is_empty() {
                    write!(f, " {}", data.attributes).unwrap();
                }

                if data.child_nodes.is_empty() {
                    writeln!(f, "></{}>", data.tag_name).unwrap();
                    return;
                }

                writeln!(f, ">").unwrap();

                let _ = &data
                    .child_nodes
                    .iter()
                    .for_each(|node| node.pretty_print(f, indent + 1));

                writeln!(f, "{}</{}>", prepadding, data.tag_name).unwrap();
            }
            NodeType::Text(text) => {
                writeln!(f, "{}{}", prepadding, text).unwrap();
            }
            NodeType::Comment(text) => writeln!(f, "{}<!-- {} -->", prepadding, text).unwrap(),
            NodeType::Document(_) => {}
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pretty_print(f, 0);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DocumentData {
    pub root: Box<Option<Node>>,
    pub stylesheets: Vec<StyleSheet>,
}

impl DocumentData {
    pub fn load_css(&mut self, styling: String) {
        self.stylesheets.push(css::parse(styling));
    }

    pub fn load_document(&mut self, document: String) {
        let node = html::parse(document, self);
        _ = self.root.insert(node);
    }

    pub fn new() -> Self {
        Self {
            root: Box::new(None),
            stylesheets: vec![],
        }
    }
}

pub fn text(data: String) -> Node {
    Node {
        node_type: NodeType::Text(data),
    }
}

pub fn element(name: String, attrs: AttrMap, children: Vec<Node>) -> Node {
    Node {
        node_type: NodeType::Element(ElementData {
            tag_name: name,
            attributes: attrs,
            child_nodes: children,
        }),
    }
}

pub fn comment(text: String) -> Node {
    Node {
        node_type: NodeType::Comment(text),
    }
}

pub fn parse(document: String) -> Node {
    let mut context = DocumentData::new();
    context.load_document(document);
    Node {
        node_type: NodeType::Document(context),
    }
}
