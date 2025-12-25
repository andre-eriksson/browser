use url::Url;

pub struct Tab {
    pub id: usize,

    pub current_url: Option<Url>,

    pub document: Option<String>,
}

impl Tab {
    pub fn new(id: usize, url: Option<Url>) -> Self {
        Tab {
            id,
            current_url: url,
            document: None,
        }
    }
}
