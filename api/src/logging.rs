pub const URL: &str = "url";
pub const DURATION: &str = "duration";
pub const STATUS_CODE: &str = "status_code";
pub const EVENT: &str = "event";
pub const TAG_TYPE: &str = "tag_type";

// === Generic Events ===
pub const EVENT_NETWORK_THREAD_STARTED: &str = "network_thread_started";
pub const EVENT_NETWORK_THREAD_STOPPED: &str = "network_thread_stopped";
pub const EVENT_NEW_TAB: &str = "new_tab";
pub const EVENT_TAB_CLOSED: &str = "tab_closed";
pub const EVENT_HTML_PARSED: &str = "html_parsed_complete";
pub const EVENT_PAGE_RETRIEVED: &str = "page_retrieved";
pub const EVENT_PAGE_LOADED: &str = "page_loaded";
pub const EVENT_FETCH_CONTENT: &str = "fetch_content";

// === Asset Events ===
pub const EVENT_LOAD_ASSET: &str = "load_asset";
pub const EVENT_ASSET_CACHE_HIT: &str = "asset_cache_hit";
pub const EVENT_ASSET_LOADED: &str = "asset_loaded";
pub const EVENT_ASSET_NOT_FOUND: &str = "asset_not_found";
