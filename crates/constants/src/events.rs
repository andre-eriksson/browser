// === Asset Loading Events ===
/// Event logged when an asset loading operation starts
pub const EVENT_LOAD_ASSET: &str = "load_asset";
/// Event logged when an asset is found in cache
pub const EVENT_ASSET_CACHE_HIT: &str = "asset_cache_hit";
/// Event logged when an asset has been successfully loaded
pub const EVENT_ASSET_LOADED: &str = "asset_loaded";
/// Event logged when a requested asset could not be found
pub const EVENT_ASSET_NOT_FOUND: &str = "asset_not_found";

// === UI Events ===
/// Event logged when a new tab is created
pub const EVENT_NEW_TAB: &str = "new_tab";
/// Event logged when a tab is closed
pub const EVENT_TAB_CLOSED: &str = "tab_closed";
