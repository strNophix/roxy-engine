use core::fmt;
use std::io;

mod dom;
mod html;

struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        return result;
    }

    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        return cur_char;
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(|c| c.is_whitespace());
    }

    fn parse_single_selector(&mut self) -> SingleSelector {
        let mut selector = SingleSelector::default();
        while !self.eof() {
            self.consume_whitespace();
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    assert!(self.next_char().is_ascii_alphanumeric());
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    assert!(self.next_char().is_ascii_alphanumeric());
                    selector.classes.push(self.parse_identifier());
                }
                '*' => {
                    self.consume_char();
                    assert!(self.next_char().is_ascii_whitespace());
                }
                '{' => {
                    break;
                }
                c => {
                    if c.is_ascii_alphanumeric() == false {
                        break;
                    }
                    selector.tag_name = Some(self.parse_identifier());
                }
            }
        }
        return selector;
    }

    fn parse_identifier(&mut self) -> String {
        self.consume_while(|c| (c.is_ascii_alphanumeric() || c == '-'))
    }

    fn parse_rule(&mut self) -> Rule {
        Rule {
            selectors: self.parse_selectors(),
            declarations: self.parse_declarations(),
        }
    }

    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(Selector::Single(self.parse_single_selector()));
            self.consume_whitespace();
            match self.next_char() {
                ',' => {
                    self.consume_char();
                    self.consume_whitespace();
                }
                '{' => break,
                c => panic!("Unexpected character {} in selector list", c),
            }
        }
        return selectors;
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        assert!(self.consume_char() == '{');
        let mut result = Vec::new();
        while !self.eof() {
            self.consume_whitespace();
            if self.next_char() == '}' {
                self.consume_char();
                break;
            }
            let identifier = self.parse_identifier();
            self.consume_whitespace();
            assert!(self.consume_char() == ':');
            self.consume_whitespace();
            let value = self.parse_declaration_value();
            result.push(Declaration {
                name: identifier,
                value: value,
            });
            self.consume_whitespace();
            assert!(self.consume_char() == ';');
        }
        return result;
    }

    fn parse_declaration_value(&mut self) -> Value {
        match self.next_char() {
            '0'..='9' => self.parse_length(),
            '#' => self.parse_color(),
            _ => Value::Keyword(self.parse_identifier()),
        }
    }

    fn parse_hex_pair(&mut self) -> u8 {
        let s = &self.input[self.pos..self.pos + 2];
        self.pos += 2;
        u8::from_str_radix(s, 16).unwrap()
    }

    fn parse_color(&mut self) -> Value {
        assert_eq!(self.consume_char(), '#');
        Value::Color(ColorValue::RGBA(
            self.parse_hex_pair(),
            self.parse_hex_pair(),
            self.parse_hex_pair(),
            255,
        ))
    }

    fn parse_length(&mut self) -> Value {
        Value::Length(self.parse_float(), self.parse_unit())
    }

    fn parse_float(&mut self) -> f32 {
        let s = self.consume_while(|c| match c {
            '0'..='9' | '.' => true,
            _ => false,
        });
        s.parse().unwrap()
    }

    fn parse_unit(&mut self) -> Unit {
        match &*self.parse_identifier().to_ascii_lowercase() {
            "px" => Unit::Px,
            _ => panic!("unrecognized unit"),
        }
    }

    fn parse_rules(&mut self) -> Vec<Rule> {
        let mut rules = Vec::new();
        while !self.eof() {
            rules.push(self.parse_rule());
            self.consume_whitespace();
        }
        rules
    }
}

fn parse(input: String) -> StyleSheet {
    let mut parser = Parser {
        pos: 0,
        input: input,
    };
    StyleSheet {
        rules: parser.parse_rules(),
    }
}

#[derive(Default, Debug)]
struct SingleSelector {
    tag_name: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
}

#[derive(Debug)]
enum Selector {
    Single(SingleSelector),
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Single(selector) => {
                if let Some(tag_name) = &selector.tag_name {
                    write!(f, "{}", tag_name).unwrap();
                }
                if selector.classes.len() > 0 {
                    write!(f, ".{}", selector.classes.join(".")).unwrap();
                }
                if let Some(id) = &selector.id {
                    write!(f, "#{}", id).unwrap();
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
enum Value {
    Keyword(String),
    Length(f32, Unit),
    Color(ColorValue),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Keyword(keyword) => {
                write!(f, "{}", keyword)
            }
            Self::Color(color) => {
                write!(f, "{}", color)
            }
            Self::Length(amount, unit) => {
                write!(f, "{}{}", amount, unit)
            }
        }
    }
}

#[derive(Debug)]
enum ColorValue {
    RGBA(u8, u8, u8, u8),
}

impl fmt::Display for ColorValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::RGBA(r, g, b, a) => write!(f, "rgba({}, {}, {}, {})", r, g, b, a),
        }
    }
}

#[derive(Debug)]
enum Unit {
    Px,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Px => write!(f, "px"),
        }
    }
}

#[derive(Debug)]
struct Declaration {
    name: String,
    value: Value,
}

impl fmt::Display for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {};", self.name, self.value)
    }
}

#[derive(Debug)]
struct Rule {
    selectors: Vec<Selector>,
    declarations: Vec<Declaration>,
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prepadding = "  ";
        let selectors = self
            .selectors
            .iter()
            .map(|d| format!("{d}"))
            .collect::<Vec<String>>();
        writeln!(f, "{} {{", selectors.join(", ")).unwrap();
        let declarations = self
            .declarations
            .iter()
            .map(|d| format!("{d}"))
            .collect::<Vec<String>>();
        writeln!(f, "{}{}", prepadding, declarations.join("")).unwrap();
        writeln!(f, "}}")
    }
}

#[derive(Debug)]
struct StyleSheet {
    rules: Vec<Rule>,
}

impl fmt::Display for StyleSheet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.rules.len() > 0 {
            let rules = self
                .rules
                .iter()
                .map(|r| format!("{r}"))
                .collect::<Vec<String>>();
            writeln!(f, "{}", rules.join("\n")).unwrap()
        }
        Ok(())
    }
}

fn main() {
    let mut input = String::new();
    let stdin = io::stdin();
    stdin.read_line(&mut input).unwrap();
    // let nodes = html::parse(input);
    let nodes = parse(input);
    println!("{nodes}");
}
