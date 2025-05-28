mod decode;
mod parser;
mod token;

use std::fs;

use parser::Parser;

fn main() {
    let val = 3;
    let file = fs::read_to_string(format!("resources/html/{}.html", val));
    let content = match file {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };

    let parser = Parser::new(&content);
    let result = parser.parse_document();
    match result {
        Ok(dom) => {
            println!("Parsed DOM: {:#?}", dom);
            // Optionally, write the DOM to a file
            std::fs::write(
                format!("resources/html/output/{}.txt", val),
                format!("{:#?}", dom),
            )
            .expect("Unable to write to file");
        }
        Err(e) => {
            eprintln!("Error parsing document: {}", e);
        }
    }
}
