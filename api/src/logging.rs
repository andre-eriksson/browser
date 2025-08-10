/// URL field key for logging events
pub const URL: &str = "url";
/// Duration field key for timing measurements in logging events
pub const DURATION: &str = "duration";
/// HTTP status code field key for response logging
pub const STATUS_CODE: &str = "status_code";
/// Event type field key for categorizing log entries
pub const EVENT: &str = "event";
/// Tag type field key for additional event categorization
pub const TAG_TYPE: &str = "tag_type";

// === Generic Events ===
/// Event logged when a new tab is created
pub const EVENT_NEW_TAB: &str = "new_tab";
/// Event logged when a tab is closed
pub const EVENT_TAB_CLOSED: &str = "tab_closed";
/// Event logged when HTML parsing is completed
pub const EVENT_HTML_PARSED: &str = "html_parsed_complete";
/// Event logged when a page is successfully retrieved
pub const EVENT_PAGE_RETRIEVED: &str = "page_retrieved";
/// Event logged when content fetching begins
pub const EVENT_FETCH_CONTENT: &str = "fetch_content";

// === Asset Events ===
/// Event logged when an asset loading operation starts
pub const EVENT_LOAD_ASSET: &str = "load_asset";
/// Event logged when an asset is found in cache
pub const EVENT_ASSET_CACHE_HIT: &str = "asset_cache_hit";
/// Event logged when an asset has been successfully loaded
pub const EVENT_ASSET_LOADED: &str = "asset_loaded";
/// Event logged when a requested asset could not be found
pub const EVENT_ASSET_NOT_FOUND: &str = "asset_not_found";
