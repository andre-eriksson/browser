use crate::{Page, PageMetadata};

/// Maximum number of pages to cache in the forward and backward navigation caches, allowing for quick retrieval of recently visited pages without reloading from the network.
const MAX_BFCACHE_SIZE: usize = 3;

/// Manages the navigation history for a browser tab, including caching of recently visited pages and their metadata.
#[derive(Debug, Clone)]
pub struct History {
    /// Cache for recently visited pages when navigating forward, allowing for quick retrieval without reloading from the network.
    f_cache: Vec<Page>,

    /// Cache for recently visited pages when navigating back, allowing for quick retrieval without reloading from the network.
    b_cache: Vec<Page>,

    /// Metadata for pages in the forward navigation history, used to reconstruct page state when navigating forward, or used
    /// to retrieve the URL and do a fresh navigation if the cached page is no longer available.
    forward: Vec<PageMetadata>,

    /// Metadata for pages in the backward navigation history, used to reconstruct page state when navigating back, or used
    /// to retrieve the URL and do a fresh navigation if the cached page is no longer available.
    backward: Vec<PageMetadata>,
}

impl History {
    /// Creates a new `History` instance with empty caches and metadata lists.
    pub fn new() -> Self {
        Self {
            f_cache: Vec::with_capacity(MAX_BFCACHE_SIZE),
            b_cache: Vec::with_capacity(MAX_BFCACHE_SIZE),

            forward: Vec::with_capacity(10),
            backward: Vec::with_capacity(10),
        }
    }

    /// Adds a new page to the forward history, caching the page for quick retrieval when navigating forward.
    pub fn add_back(&mut self, page: Page, metadata: PageMetadata) {
        self.push_back_entry(page, metadata);
        self.clear_forward_history();
    }

    /// Adds a new page to the forward history, caching the page for quick retrieval when navigating forward.
    fn push_forward_entry(&mut self, page: Page, metadata: PageMetadata) {
        Self::push_cached_page(&mut self.f_cache, page);
        self.forward.push(metadata);
    }

    /// Adds a new page to the backward history, caching the page for quick retrieval when navigating back.
    fn push_back_entry(&mut self, page: Page, metadata: PageMetadata) {
        Self::push_cached_page(&mut self.b_cache, page);
        self.backward.push(metadata);
    }

    /// Pushes a page into the specified cache, ensuring that the cache does not exceed the defined maximum size by removing the oldest entry if necessary.
    fn push_cached_page(cache: &mut Vec<Page>, page: Page) {
        if cache.len() >= MAX_BFCACHE_SIZE {
            cache.remove(0);
        }
        cache.push(page);
    }

    /// Clears the forward history, including both the cached pages and their associated metadata, to ensure that navigating to a new page
    /// from the current position resets the forward navigation state.
    fn clear_forward_history(&mut self) {
        self.f_cache.clear();
        self.forward.clear();
    }

    /// Checks if there are entries available in the backward history, indicating that the user can navigate back to a previous page.
    pub const fn can_go_back(&self) -> bool {
        !self.backward.is_empty()
    }

    /// Checks if there are entries available in the forward history, indicating that the user can navigate forward to a previously visited page.
    pub const fn can_go_forward(&self) -> bool {
        !self.forward.is_empty()
    }

    /// Navigates back to the previous page in the history, returning the cached page if available along with its metadata.
    /// If the cache is exhausted, it returns `None` for the page, and the metadata can be used to perform a fresh navigation
    /// to the previous URL.
    pub fn go_back(&mut self, page: Page, metadata: PageMetadata) -> (Option<Page>, PageMetadata) {
        let previous_metadata = self
            .backward
            .pop()
            .expect("There should be page metadata available when going back");
        let cached_page = self.b_cache.pop();

        self.push_forward_entry(page, metadata);
        (cached_page, previous_metadata)
    }

    /// Navigates forward to the next page in the history, returning the cached page if available along with its metadata.
    /// If the cache is exhausted, it returns `None` for the page, and the metadata can be used to perform a fresh navigation
    /// to the next URL.
    pub fn go_forward(&mut self, page: Page, metadata: PageMetadata) -> (Option<Page>, PageMetadata) {
        let previous_metadata = self
            .forward
            .pop()
            .expect("There should be page metadata available when going forward");
        let cached_page = self.f_cache.pop();

        self.push_back_entry(page, metadata);
        (cached_page, previous_metadata)
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use io::DocumentPolicy;
    use url::Url;

    use super::*;

    fn page_metadata(index: usize) -> PageMetadata {
        PageMetadata {
            url: Url::parse(&format!("http://example.com/page{index}")).unwrap(),
            title: format!("Page {index}"),
            favicon: None,
            policies: DocumentPolicy::default(),
        }
    }

    #[test]
    fn test_history_navigation() {
        let mut history = History::new();

        let page1 = Page::blank();
        let metadata1 = page_metadata(1);

        let page2 = Page::blank();
        let metadata2 = page_metadata(2);

        history.add_back(page1.clone(), metadata1.clone());

        assert!(history.can_go_back());
        assert!(!history.can_go_forward());

        let (back_page, back_metadata) = history.go_back(page2.clone(), metadata2.clone());
        assert!(back_page.is_some());
        assert_eq!(back_metadata.url.as_str(), "http://example.com/page1");
        assert_eq!(back_metadata.title, "Page 1");

        assert!(!history.can_go_back());
        assert!(history.can_go_forward());

        let (forward_page, forward_metadata) = history.go_forward(page1.clone(), metadata1.clone());
        assert!(forward_page.is_some());
        assert_eq!(forward_metadata.url.as_str(), "http://example.com/page2");
        assert_eq!(forward_metadata.title, "Page 2");
    }

    #[test]
    fn test_new_navigation_clears_forward_history() {
        let mut history = History::new();

        history.add_back(Page::blank(), page_metadata(1));
        history.add_back(Page::blank(), page_metadata(2));

        let _ = history.go_back(Page::blank(), page_metadata(3));
        assert!(history.can_go_forward());

        history.add_back(Page::blank(), page_metadata(2));
        assert!(!history.can_go_forward());
    }

    #[test]
    fn test_go_forward_preserves_remaining_forward_entries() {
        let mut history = History::new();

        history.add_back(Page::blank(), page_metadata(1));
        history.add_back(Page::blank(), page_metadata(2));
        history.add_back(Page::blank(), page_metadata(3));

        let _ = history.go_back(Page::blank(), page_metadata(4));
        let _ = history.go_back(Page::blank(), page_metadata(3));

        let (cached_page, metadata) = history.go_forward(Page::blank(), page_metadata(2));
        assert!(cached_page.is_some());
        assert_eq!(metadata.url.as_str(), "http://example.com/page3");
        assert!(history.can_go_forward());

        let (cached_page, metadata) = history.go_forward(Page::blank(), page_metadata(3));
        assert!(cached_page.is_some());
        assert_eq!(metadata.url.as_str(), "http://example.com/page4");
        assert!(!history.can_go_forward());
    }

    #[test]
    fn test_history_navigation_beyond_cache_size_uses_metadata_fallback() {
        let mut history = History::new();
        let total_history_entries = MAX_BFCACHE_SIZE + 3;

        for idx in 0..total_history_entries {
            history.add_back(Page::blank(), page_metadata(idx));
        }

        for step in 0..total_history_entries {
            let expected_index = total_history_entries - 1 - step;
            let synthetic_current_page = 1_000 + step;

            let (cached_page, metadata) = history.go_back(Page::blank(), page_metadata(synthetic_current_page));
            assert_eq!(metadata.url.as_str(), format!("http://example.com/page{expected_index}"));

            if step < MAX_BFCACHE_SIZE {
                assert!(cached_page.is_some());
            } else {
                assert!(cached_page.is_none());
            }
        }

        assert!(!history.can_go_back());
        assert!(history.can_go_forward());

        for step in 0..total_history_entries {
            let expected_index = 1_000 + (total_history_entries - 1 - step);
            let synthetic_current_page = 2_000 + step;

            let (cached_page, metadata) = history.go_forward(Page::blank(), page_metadata(synthetic_current_page));
            assert_eq!(metadata.url.as_str(), format!("http://example.com/page{expected_index}"));

            if step < MAX_BFCACHE_SIZE {
                assert!(cached_page.is_some());
            } else {
                assert!(cached_page.is_none());
            }
        }

        assert!(!history.can_go_forward());
    }
}
