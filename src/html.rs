#![allow(dead_code)]
use std::collections::HashMap;

use crate::dom::{comment, element, text, AttrMap, AttrValue, DocumentData, Node, NodeType};

struct Parser<'a> {
    pos: usize,
    input: String,
    context: &'a mut DocumentData,
}

impl Parser<'_> {
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        cur_char
    }

    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(|c| c.is_whitespace());
    }

    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| c.is_ascii_alphanumeric())
    }

    fn parse_node(&mut self) -> Node {
        if self.starts_with("<!--") {
            return self.parse_comment();
        }

        match self.next_char() {
            '<' => self.parse_element(),
            _ => self.parse_text(),
        }
    }

    fn parse_text(&mut self) -> Node {
        text(self.consume_while(|c| c != '<'))
    }

    fn parse_element(&mut self) -> Node {
        // Opening tag.
        assert!(self.consume_char() == '<');
        let tag_name = self.parse_tag_name();
        let attrs = self.parse_attributes();

        // Optional self-closing
        let next_char = self.consume_char();
        if next_char == '/' {
            assert!(self.consume_char() == '>');
            return element(tag_name, attrs, vec![]);
        }
        assert!(next_char == '>');

        // Contents.
        let children = self.parse_nodes();

        if tag_name == "style" {
            let inner_node = children.first().unwrap();
            if let NodeType::Text(styling) = &inner_node.node_type {
                self.context.load_css(styling.clone());
            }
        }

        // Closing tag.
        assert!(self.consume_char() == '<');
        assert!(self.consume_char() == '/');
        assert!(self.parse_tag_name() == tag_name);
        assert!(self.consume_char() == '>');

        element(tag_name, attrs, children)
    }

    fn parse_comment(&mut self) -> Node {
        assert!(self.starts_with("<!--"));
        assert!(self.consume_char() == '<');
        assert!(self.consume_char() == '!');
        assert!(self.consume_char() == '-');
        assert!(self.consume_char() == '-');
        let mut result = String::new();
        loop {
            let c = self.consume_char();
            if c == '-' && self.starts_with("->") {
                break;
            }
            result.push(c);
        }
        assert!(self.consume_char() == '-');
        assert!(self.consume_char() == '>');
        comment(result)
    }

    fn parse_attributes(&mut self) -> AttrMap {
        let mut attributes = HashMap::new();
        loop {
            self.consume_whitespace();

            let next_char = self.next_char();
            if next_char == '>' || next_char == '/' {
                break;
            }

            let (name, value) = self.parse_attr();
            if value.is_empty() {
                attributes.insert(name, AttrValue::Text(value));
            } else {
                attributes.insert(name, AttrValue::Implicit);
            }
        }
        AttrMap(attributes)
    }

    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        assert!(self.consume_char() == open_quote);
        value
    }

    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_tag_name();
        if self.consume_char() == '=' {
            let value = self.parse_attr_value();
            return (name, value);
        }

        (name, String::new())
    }

    fn parse_nodes(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        nodes
    }
}

pub fn parse(input: String, context: &mut DocumentData) -> Node {
    let mut parser = Parser {
        pos: 0,
        input,
        context,
    };
    let mut nodes = parser.parse_nodes();

    if nodes.len() == 1 {
        nodes.swap_remove(0)
    } else {
        element("html".into(), AttrMap::default(), nodes)
    }
}
