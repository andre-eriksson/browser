use html_dom::{Collector, HtmlTag, Tag, TagInfo};

#[derive(Default)]
pub struct TabCollector {
    /// Indicates whether the parser is currently within the `<head>` section of the HTML document.
    pub in_head: bool,

    /// Indicates whether the parser is currently within a `<title>` tag.
    pub in_title: bool,

    /// The title of the tab, if available.
    pub title: Option<String>,
}

impl Collector for TabCollector {
    fn collect(&mut self, tag: &TagInfo) {
        if *tag.tag == Tag::Html(HtmlTag::Head) {
            self.in_head = true;
            return;
        }

        if *tag.tag == Tag::Html(HtmlTag::Body) {
            self.in_head = false;
            return;
        }

        if !self.in_head {
            return;
        }

        if *tag.tag == Tag::Html(HtmlTag::Title) {
            self.in_title = true;
        }

        if self.in_title
            && let Some(title) = tag.data
        {
            self.title = Some(title.to_string());
            self.in_title = false;
        }
    }

    fn into_result(self) -> Self {
        Self {
            in_head: self.in_head,
            in_title: self.in_title,
            title: self.title,
        }
    }
}
