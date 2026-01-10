use tracing::debug;

use crate::{
    events::BrowserEvent,
    navigation::NavigationContext,
    tab::{Tab, TabId, TabManager},
};

/// Adds a new tab to the browser and returns a `BrowserEvent` indicating the addition.
pub fn add_tab(tab_manager: &mut TabManager) -> BrowserEvent {
    let new_tab_id = TabId(tab_manager.next_tab_id());
    let new_tab = Tab::new(new_tab_id);
    tab_manager.add_tab(new_tab);
    debug!("Added new tab with ID {:?}", new_tab_id);

    BrowserEvent::TabAdded(new_tab_id)
}

/// Closes the tab with the specified `tab_id`. If the closed tab is the active tab,
pub fn close_tab(tab_manager: &mut TabManager, tab_id: TabId) -> Result<BrowserEvent, String> {
    debug!("Closing tab with ID {:?}", tab_id);
    tab_manager.close_tab(tab_id)?;

    if tab_manager.active_tab_id() == tab_id {
        tab_manager.change_to_any_tab()?;
    }

    Ok(BrowserEvent::TabClosed(tab_id))
}

/// Changes the active tab to the tab with the specified `tab_id`.
pub fn change_active_tab(
    navigation_context: &mut dyn NavigationContext,
    tab_id: TabId,
) -> Result<BrowserEvent, String> {
    debug!("Changing active tab to ID {:?}", tab_id);
    navigation_context.tab_manager().change_active_tab(tab_id)?;
    navigation_context.emit_event(BrowserEvent::ActiveTabChanged(tab_id));

    Ok(BrowserEvent::ActiveTabChanged(tab_id))
}
