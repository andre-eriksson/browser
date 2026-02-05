use css_cssom::CSSStyleSheet;

use crate::{service::network::service::NetworkService, tab::manager::TabManager};

/// A trait representing the ability to process CSS styles within the browser context.
pub trait StyleProcessor {
    /// Processes the given CSS and adds any stylesheets to the provided vector.
    fn process_css(&self, css: &str, stylesheets: &mut Vec<CSSStyleSheet>);
}

/// A trait representing the ability to execute scripts within the browser context.
pub trait ScriptExecutor {
    /// Executes the given script in the context of the browser.
    fn execute_script(&self, script: &str);
}

pub trait NavigationContext: Send {
    fn script_executor(&self) -> &dyn ScriptExecutor;
    fn style_processor(&self) -> &dyn StyleProcessor;
    fn network_service(&mut self) -> &mut NetworkService;
    fn tab_manager(&mut self) -> &mut TabManager;
}
