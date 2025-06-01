use std::fs;

use html_parser::parser::streaming::StreamingParser;

fn main() {
    streaming();
}

fn streaming() {
    let val = &"edge_cases";
    let file = fs::read_to_string(format!("resources/html/{}.html", val));
    let content = match file {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };

    let total_time = std::time::Instant::now();
    let mut parser = StreamingParser::new(content.as_bytes(), None);
    let result = parser.parse();

    match result {
        Ok(dom) => {
            //println!("Parsed DOM: {:#?}", dom);
            println!("Parsing completed in: {:?}", total_time.elapsed());
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
