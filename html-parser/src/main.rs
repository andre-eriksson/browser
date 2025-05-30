use std::fs;

use html_parser::parser::Parser;

fn main() {
    let full_time = std::time::Instant::now();
    let val = &"easy";
    let file = fs::read_to_string(format!("resources/html/{}.html", val));
    let content = match file {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };

    let lexing_start_time = std::time::Instant::now();
    let mut parser = Parser::new(&content, Some(100_000 as usize));
    println!("Lexing time: {:?}", lexing_start_time.elapsed());

    // Initialize regexes
    let parser_start_time = std::time::Instant::now();
    let result = parser.parse_document();

    match result {
        Ok(dom) => {
            //println!("Parsed DOM: {:#?}", dom);

            println!("Parsing completed in: {:?}", parser_start_time.elapsed());
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

    println!("Total time: {:?}", full_time.elapsed());
}
