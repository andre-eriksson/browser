mod browser;
mod cli;
mod commands;
pub mod errors;
mod events;
mod header;
mod headless;
mod navigation;
mod tab;

pub use browser::Browser;
pub use cli::headless::HeadlessBrowser;
pub use events::{Commandable, Emitter, KernelCommand, KernelResponse};
pub use headless::HeadlessEngine;
pub use tab::collector::TabCollector;
pub use tab::page::{DevtoolsPage, Page};
pub use tab::tabs::{HistoryState, Tab, TabId};
