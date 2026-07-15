use crate::{
    Entry,
    entry::{AppDirectory, AppFile},
};

pub const PROFILE_CACHE_USER_AGENT: AppFile = AppFile(Entry::cache("stylesheets/useragent.bin", false));

pub const PROFILE_PREFERENCES: AppFile = AppFile(Entry::config("preferences.toml", false));

pub const GLOBAL_THEMES_DIRECTORY: AppDirectory = AppDirectory(Entry::user_data("themes/", true));
pub const PROFILE_THEMES_DIRECTORY: AppDirectory = AppDirectory(Entry::user_data("themes/", false));
