use std::collections::HashMap;

use html_dom::{Collector, HtmlTag, NodeId, Tag, TagInfo};

#[derive(Default)]
pub struct TabCollector {
    /// The title of the tab, if available.
    pub title: Option<String>,

    /// The URLs of images found in the document.
    pub images: HashMap<String, Vec<NodeId>>,
}

impl Collector for TabCollector {
    fn collect(&mut self, tag: &TagInfo) {
        if *tag.tag == Tag::Html(HtmlTag::Img)
            && let Some(src) = tag.attributes.as_ref().and_then(|attrs| attrs.get("src"))
        {
            self.images
                .entry(src.clone())
                .or_default()
                .push(tag.node_id);
        }

        if *tag.tag == Tag::Html(HtmlTag::Title)
            && self.title.is_none()
            && let Some(data) = tag.data
            && !data.trim().is_empty()
        {
            self.title = Some(data.clone());
        }
    }

    fn into_result(self) -> Self {
        self
    }
}
