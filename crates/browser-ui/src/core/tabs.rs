mod devtools;
mod handler;
pub mod manager;
pub mod page;
pub mod tab;

pub use devtools::{Devtools, DevtoolsContext, DevtoolsPage};
pub use page::Page;
pub use tab::{Tab, TabId};
