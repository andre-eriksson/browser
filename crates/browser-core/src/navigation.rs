use css_cssom::CSSStyleSheet;
use network::http::client::HttpClient;

use crate::{BrowserEvent, tab::TabManager};

/// A trait representing the ability to process CSS styles within the browser context.
pub trait StyleProcessor {
    /// Processes the given CSS and adds any stylesheets to the provided vector.
    fn process_css(&mut self, css: &str, stylesheets: &mut Vec<CSSStyleSheet>);
}

/// A trait representing the ability to execute scripts within the browser context.
pub trait ScriptExecutor {
    /// Executes the given script in the context of the browser.
    fn execute_script(&mut self, script: &str);
}

/// A trait representing the context required for navigation operations within the browser.
pub trait NavigationContext: Send {
    /// Returns a reference to the HTTP client used for making requests.
    fn http_client(&self) -> &dyn HttpClient;

    /// Returns a mutable reference to the tab manager.
    fn tab_manager(&mut self) -> &mut TabManager;

    /// Returns the default stylesheet, if any.
    fn default_stylesheet(&self) -> Option<&CSSStyleSheet>;

    /// Emits the specified browser event.
    fn emit_event(&self, event: BrowserEvent);

    /// Processes the given CSS and adds any stylesheets to the provided vector.
    fn process_css(&mut self, css: &str, stylesheets: &mut Vec<CSSStyleSheet>);

    /// Executes the given script in the context of the browser.
    fn execute_script(&mut self, script: &str);
}
