use cookie::Cookie;

#[derive(Debug)]
pub enum BrowserCommand {
    Navigate { tab_id: usize, url: String },
    Reload(usize),
    Stop(usize),

    // Cookie management
    AddCookie(usize, Cookie<'static>),
    GetCookies(usize),
}
