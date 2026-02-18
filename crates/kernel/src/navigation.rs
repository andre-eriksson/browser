use std::sync::{Arc, Mutex};

use cookies::CookieJar;
use network::{HeaderMap, client::HttpClient};

use crate::tab::manager::TabManager;

/// A trait representing the ability to execute scripts within the browser context.
pub trait ScriptExecutor {
    /// Executes the given script in the context of the browser.
    fn execute_script(&self, script: &str);
}

pub trait NavigationContext: Send {
    fn script_executor(&self) -> &dyn ScriptExecutor;
    fn http_client(&self) -> &dyn HttpClient;
    fn cookie_jar(&mut self) -> &mut Arc<Mutex<CookieJar>>;
    fn headers(&self) -> &Arc<HeaderMap>;
    fn tab_manager(&mut self) -> &mut TabManager;
}
