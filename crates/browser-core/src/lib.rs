mod browser;
mod cli;
mod commands;
mod events;
mod headless;
mod navigation;
mod service;
mod tab;

pub use browser::Browser;
pub use cli::headless::HeadlessBrowser;
pub use events::{BrowserCommand, BrowserEvent, Commandable, Emitter};
pub use headless::HeadlessEngine;
pub use tab::collector::TabCollector;
pub use tab::tabs::{Tab, TabId};
