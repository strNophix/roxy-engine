use std::io;

use dom::NodeType;

mod css;
mod dom;
mod html;

fn main() {
    let mut input = String::new();
    let stdin = io::stdin();
    stdin.read_line(&mut input).unwrap();
    // let input = "<html><head><title>Document</title><style>h1{font-size: 14px;}</style></head><body><h1>Hello</h1></body></html>".into();

    let node = dom::parse(input);
    if let NodeType::DocumentNode(data) = node.node_type {
        println!("{}", data.stylesheets[0]);
    }
}
