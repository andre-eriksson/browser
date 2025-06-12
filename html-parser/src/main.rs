use std::fs;

use html_parser::parser::{options::ParserOptions, streaming::StreamingParser};

fn main() {
    streaming();
}

fn streaming() {
    let val = &"book";
    let file = fs::read_to_string(format!("resources/html/{}.html", val));
    let content = match file {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };

    let mut parser =
        StreamingParser::new_with_options(content.as_bytes(), None, ParserOptions::default());

    let total_time = std::time::Instant::now();
    let result = parser.parse();

    match result {
        Ok(result) => {
            println!("Parsed DOM successfully.");
            println!("Total time taken: {:?}", total_time.elapsed());
            //println!("Parsed DOM: {:#?}", result.dom_tree);

            std::fs::write(
                format!("resources/html/output/{}.txt", val),
                format!("{:#?}", result.dom_tree),
            )
            .expect("Unable to write to file");
        }
        Err(e) => {
            eprintln!("Error parsing document: {}", e);
        }
    }
}
