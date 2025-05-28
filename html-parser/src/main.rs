mod decode;
mod dom;
mod parser;
mod token;

use parser::Parser;

fn main() {
    let file = include_str!("..\\res\\3.html");
    let content = file.as_bytes();

    let parser = Parser::new(std::str::from_utf8(&content).unwrap_or(""));
    let result = parser.parse_document();
    match result {
        Ok(dom) => {
            println!("Parsed DOM: {:#?}", dom);
        }
        Err(e) => {
            eprintln!("Error parsing document: {}", e);
        }
    }
}
