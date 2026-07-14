mod database;
mod paths;

use browser_args::BrowserArgs;
use browser_config::BrowserConfig;
use cookies::CookieJar;
use http_cache::http::HttpCache;
use tracing::{trace, warn};

use crate::profile::{database::Databases, paths::ProfilePaths};

#[derive(Debug, Clone)]
pub enum ProfileKind {
    Persistent { id: Option<String> },
    Temporary { custom_suffix: Option<String> },
}

#[derive(Debug)]
pub struct Profile {
    config: BrowserConfig,
    databases: Databases,
    dirs: ProfilePaths,
}

impl Profile {
    pub fn new(args: &BrowserArgs) -> Self {
        let config = BrowserConfig::new(args);

        let profile_kind = if args.incognito {
            ProfileKind::Temporary {
                custom_suffix: None,
            }
        } else {
            ProfileKind::Persistent {
                id: args.profile.clone(),
            }
        };

        trace!("Initializing profile with kind: {:?}", profile_kind);

        let dirs = ProfilePaths::new(profile_kind);
        if dirs.is_degraded() {
            warn!(
                "Profile directories are degraded. Some features may not work as expected. Please check your file system permissions and available disk space."
            )
        }

        trace!("Profile directories initialized: {:?}", dirs);

        let databases = Databases::init(&dirs).expect("Failed to initialize databases, which is required for the browser to function. Please ensure you have enough disk space and permissions to create necessary files.");

        Self {
            dirs,
            config,
            databases,
        }
    }

    pub fn config(&self) -> &BrowserConfig {
        &self.config
    }

    pub const fn http_cache(&self) -> &HttpCache {
        &self.databases.http_cache
    }

    pub const fn cookie_jar(&self) -> &CookieJar {
        &self.databases.cookie_jar
    }

    pub fn dirs(&self) -> &ProfilePaths {
        &self.dirs
    }
}
