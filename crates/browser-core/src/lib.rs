mod browser;
mod cli;
mod commands;
mod events;
mod navigation;
mod service;
mod tab;

pub use browser::Browser;
pub use cli::headless::HeadlessBrowser;
pub use events::{BrowserCommand, BrowserEvent, Commandable, Emitter};
pub use tab::tabs::{Tab, TabId};
