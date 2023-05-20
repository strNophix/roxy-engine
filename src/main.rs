use std::{env, fs};

mod css;
mod dom;
mod html;

fn main() {
    let file_path = env::args().nth(1).unwrap();
    let contents = fs::read_to_string(file_path).unwrap();
    let node = dom::parse(contents);
    println!("{:#?}", node);
}
