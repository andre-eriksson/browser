#[derive(Debug, PartialEq)]
pub enum SourceType {
    Frame,
    Script,
    Style,
    Image,
    Font,
    Media,
    Worker,
    Manifest,
    Fetch,
}

pub fn get_source_from_tag(tag_name: &str) -> SourceType {
    match tag_name {
        "frame" => SourceType::Frame,
        "script" => SourceType::Script,
        "style" => SourceType::Style,
        "img" => SourceType::Image,
        "font" => SourceType::Font,
        "media" => SourceType::Media,
        "worker" => SourceType::Worker,
        "manifest" => SourceType::Manifest,
        _ => SourceType::Fetch, // Default to Fetch for unrecognized tags
    }
}
