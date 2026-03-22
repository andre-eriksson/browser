use crate::errors::KernelError;
use tracing::trace;

use crate::{
    Tab,
    events::KernelResponse,
    tab::{manager::TabManager, tabs::TabId},
};

/// Adds a new tab to the browser and returns a `BrowserEvent` indicating the addition.
pub(crate) fn add_tab(tab_manager: &mut TabManager) -> KernelResponse {
    let new_tab_id = TabId(tab_manager.next_tab_id());
    let new_tab = Tab::new(new_tab_id);
    tab_manager.add_tab(new_tab);
    trace!("Added new tab with ID {:?}", new_tab_id);

    KernelResponse::TabAdded(new_tab_id)
}

/// Closes the tab with the specified `tab_id`. If the closed tab is the active tab,
pub(crate) fn close_tab(tab_manager: &mut TabManager, tab_id: TabId) -> Result<KernelResponse, KernelError> {
    trace!("Closing tab with ID {:?}", tab_id);
    tab_manager.close_tab(tab_id)?;

    if tab_manager.active_tab_id() == tab_id {
        let new_tab_id = tab_manager.change_to_any_tab()?;
        return Ok(KernelResponse::TabClosed(tab_id, Some(new_tab_id)));
    }

    Ok(KernelResponse::TabClosed(tab_id, None))
}

/// Changes the active tab to the tab with the specified `tab_id`.
pub(crate) fn change_active_tab(tab_manager: &mut TabManager, tab_id: TabId) -> Result<KernelResponse, KernelError> {
    trace!("Changing active tab to ID {:?}", tab_id);

    tab_manager.change_active_tab(tab_id)?;

    Ok(KernelResponse::ActiveTabChanged(tab_id))
}
