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
pub use events::{BrowserCommand, BrowserEvent, Commandable, Emitter};
pub use headless::HeadlessEngine;
pub use tab::collector::TabCollector;
pub use tab::page::Page;
pub use tab::tabs::{Tab, TabId};
