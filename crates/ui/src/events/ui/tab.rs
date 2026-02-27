use iced::Task;
use kernel::{BrowserCommand, Commandable, TabId};

use crate::core::{Application, Event};

pub(crate) fn create_new_tab(application: &mut Application) -> Task<Event> {
    let browser = application.browser.clone();

    Task::perform(
        async move {
            let mut lock = browser.lock().await;
            lock.execute(BrowserCommand::AddTab).await
        },
        |result| match result {
            Ok(task) => Event::Browser(task),
            Err(_) => Event::None,
        },
    )
}

pub(crate) fn close_tab(application: &mut Application, tab_id: TabId) -> Task<Event> {
    let browser = application.browser.clone();

    Task::perform(
        async move {
            let mut lock = browser.lock().await;
            lock.execute(BrowserCommand::CloseTab { tab_id }).await
        },
        |result| match result {
            Ok(task) => Event::Browser(task),
            Err(_) => Event::None,
        },
    )
}

pub(crate) fn change_active_tab(application: &mut Application, tab_id: TabId) -> Task<Event> {
    let browser = application.browser.clone();

    Task::perform(
        async move {
            let mut lock = browser.lock().await;
            lock.execute(BrowserCommand::ChangeActiveTab { tab_id })
                .await
        },
        |result| match result {
            Ok(task) => Event::Browser(task),
            Err(_) => Event::None,
        },
    )
}
