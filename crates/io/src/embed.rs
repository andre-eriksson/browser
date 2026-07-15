use std::borrow::Cow;

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../assets/"]
#[include = "**/*"]
#[exclude = ".gitignore"]
/// A struct representing embedded resources in the application,
/// using the `rust_embed` crate to include files from the specified folder.
pub(crate) struct EmbededResource;

/// Represents different types of embedded resources in the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmbeddedType<'path> {
    Icon(&'path str),
    Font(&'path str),
    Image(&'path str),
    Shader(&'path str),
    Browser(&'path str),
    Root(&'path str),
}

impl EmbeddedType<'_> {
    /// Returns the path of the embedded resource based on its type and name.
    #[must_use]
    pub fn path(&self) -> String {
        match self {
            EmbeddedType::Icon(name) => format!("icon/{name}"),
            EmbeddedType::Font(name) => format!("font/{name}"),
            EmbeddedType::Image(name) => format!("image/{name}"),
            EmbeddedType::Shader(name) => format!("shader/{name}"),
            EmbeddedType::Browser(name) => format!("browser/{name}"),
            EmbeddedType::Root(name) => name.to_string(),
        }
    }

    pub fn load(self) -> Cow<'static, [u8]> {
        let resource = EmbededResource::get(&self.path()).unwrap_or_else(|| {
            panic!("Embedded asset not found: {}", self.path());
        });

        resource.data
    }
}
