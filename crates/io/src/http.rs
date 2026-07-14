use bytes::Bytes;
use http_types::request::Request;
use storage::Directory;

use crate::{Entry, embeded::EmbededType, errors::ResourceError, loader::Loadable};

/// A list of allowed "about:" URLs that the browser can load.
/// This is a security measure to prevent loading potentially harmful or
/// unintended content through "about:" URLs. Only the URLs specified in
/// this list will be allowed to be loaded by the browser.
const ALLOWED_ABOUT_URLS: &[&str] = &["blank"];

impl Loadable for Request {
    type Output = Bytes;
    fn load_asset(self, dirs: &Directory, max_file_size: Option<u64>) -> Result<Self::Output, ResourceError> {
        let scheme = self.context.url.scheme();

        if scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https") {
            return Err(ResourceError::UnsupportedProtocol(
                "Use the async `fetch` function instead to load network".to_string(),
            ));
        }
        let path = self.context.url.path();

        match scheme {
            "file" => {
                let entry = Entry::absolute(path);

                entry.load_asset(dirs, max_file_size)
            }
            "about" => {
                let url = self.context.url.path();
                if ALLOWED_ABOUT_URLS.contains(&url) {
                    let data = EmbededType::Browser(path).load();

                    Ok(data.into_owned().into())
                } else {
                    Err(ResourceError::UnsupportedProtocol(format!("The 'about:{url}' URL is not allowed.")))
                }
            }
            _ => Err(ResourceError::UnsupportedProtocol(format!("The '{scheme}' protocol is not supported."))),
        }
    }
}
