use std::fmt::Display;

use strum::AsRefStr;

/// Represents an HTML tag, which can be either a known tag or an unknown tag.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tag {
    /// Represents a known HTML tag defined in the `Self` enum.
    Html(HtmlTag),

    /// Represents a known SVG tag defined in the `SvgTag` enum.
    Svg(SvgTag),

    /// Represents a known MathML tag defined in the `MathTag` enum.
    MathML(MathTag),

    /// Represents an unknown tag, where the string is the tag name, for instance custom tags like `<yt-thumbnail-view-model>`.
    Unknown(String),
}

impl Tag {
    #[must_use]
    pub fn from_str_insensitive(s: &str) -> Self {
        HtmlTag::from_str_insensitive(s).map_or_else(
            || SvgTag::from_str_insensitive(s).map_or_else(|| Self::Unknown(s.to_string()), Self::Svg),
            Self::Html,
        )
    }

    /// Checks if the tag is a void element.
    #[must_use]
    pub const fn is_void_element(&self) -> bool {
        match self {
            Self::Html(html_tag) => html_tag.is_void_element(),
            Self::Svg(svg_tag) => svg_tag.is_void_element(),
            Self::MathML(_) | Self::Unknown(_) => false,
        }
    }

    /// Determines if the current tag should automatically close based on the new tag being encountered.
    #[must_use]
    pub const fn should_auto_close(&self, new_tag: &Self) -> bool {
        match self {
            Self::Html(html_tag) => html_tag.should_auto_close(new_tag),
            Self::Svg(_) | Self::MathML(_) | Self::Unknown(_) => false,
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Html(html_tag) => html_tag.as_ref(),
            Self::Svg(svg_tag) => svg_tag.as_ref(),
            Self::MathML(math_tag) => math_tag.as_ref(),
            Self::Unknown(name) => name.as_str(),
        }
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Html(html_tag) => html_tag.to_string(),
                Self::Svg(svg_tag) => svg_tag.to_string(),
                Self::MathML(math_tag) => math_tag.to_string(),
                Self::Unknown(name) => name.clone(),
            }
        )
    }
}

impl From<HtmlTag> for Tag {
    fn from(value: HtmlTag) -> Self {
        Self::Html(value)
    }
}

impl From<SvgTag> for Tag {
    fn from(value: SvgTag) -> Self {
        Self::Svg(value)
    }
}

/// Represents known HTML tags as an enum.
///
/// This enum includes common HTML tags that are recognized by the parser.
///
/// <https://html.spec.whatwg.org/multipage/#toc-semantics>
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum HtmlTag {
    A,
    Abbr,
    Address,
    Area,
    Article,
    Aside,
    Audio,
    B,
    Base,
    Bdi,
    Bdo,
    Blockquote,
    Body,
    Br,
    Button,
    Canvas,
    Caption,
    Cite,
    Code,
    Col,
    Colgroup,
    Data,
    Datalist,
    Dd,
    Del,
    Details,
    Dfn,
    Dialog,
    Div,
    Dl,
    Dt,
    Em,
    Embed,
    Fieldset,
    Figcaption,
    Figure,
    Footer,
    Form,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    Head,
    Header,
    HGroup,
    Hr,
    Html,
    I,
    Iframe,
    Img,
    Input,
    Ins,
    Kbd,
    Label,
    Legend,
    Li,
    Link,
    Main,
    Map,
    Mark,
    Math,
    Menu,
    Meta,
    Meter,
    Nav,
    Noscript,
    Object,
    Ol,
    Optgroup,
    Option,
    Output,
    P,
    Picture,
    Pre,
    Progress,
    Q,
    Rp,
    Rt,
    Ruby,
    S,
    Samp,
    Script,
    Search,
    Section,
    Select,
    Selectedcontent,
    Slot,
    Small,
    Source,
    Span,
    Strong,
    Style,
    Sub,
    Summary,
    Sup,
    Svg,
    Table,
    Tbody,
    Td,
    Template,
    Textarea,
    Tfoot,
    Th,
    Thead,
    Time,
    Title,
    Tr,
    Track,
    U,
    Ul,
    Var,
    Video,
    Wbr,
}

impl HtmlTag {
    #[must_use]
    pub fn from_str_insensitive(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        match bytes.len() {
            1 => match [bytes[0].to_ascii_lowercase()] {
                [b'p'] => Some(Self::P),
                [b'a'] => Some(Self::A),
                [b's'] => Some(Self::S),
                [b'q'] => Some(Self::Q),
                [b'i'] => Some(Self::I),
                [b'b'] => Some(Self::B),
                [b'u'] => Some(Self::U),
                _ => None,
            },
            2 => match [bytes[0].to_ascii_lowercase(), bytes[1].to_ascii_lowercase()] {
                [b'h', b'1'] => Some(Self::H1),
                [b'h', b'2'] => Some(Self::H2),
                [b'h', b'3'] => Some(Self::H3),
                [b'h', b'4'] => Some(Self::H4),
                [b'h', b'5'] => Some(Self::H5),
                [b'h', b'6'] => Some(Self::H6),
                [b'h', b'r'] => Some(Self::Hr),
                [b'o', b'l'] => Some(Self::Ol),
                [b'u', b'l'] => Some(Self::Ul),
                [b'l', b'i'] => Some(Self::Li),
                [b'd', b'l'] => Some(Self::Dl),
                [b'd', b't'] => Some(Self::Dt),
                [b'd', b'd'] => Some(Self::Dd),
                [b'e', b'm'] => Some(Self::Em),
                [b'r', b't'] => Some(Self::Rt),
                [b'r', b'p'] => Some(Self::Rp),
                [b'b', b'r'] => Some(Self::Br),
                [b't', b'r'] => Some(Self::Tr),
                [b't', b'd'] => Some(Self::Td),
                [b't', b'h'] => Some(Self::Th),
                _ => None,
            },
            3 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                ] {
                    [b'n', b'a', b'v'] => Some(Self::Nav),
                    [b'p', b'r', b'e'] => Some(Self::Pre),
                    [b'd', b'i', b'v'] => Some(Self::Div),
                    [b'd', b'f', b'n'] => Some(Self::Dfn),
                    [b'v', b'a', b'r'] => Some(Self::Var),
                    [b'k', b'b', b'd'] => Some(Self::Kbd),
                    [b's', b'u', b'b'] => Some(Self::Sub),
                    [b's', b'u', b'p'] => Some(Self::Sup),
                    [b'b', b'd', b'i'] => Some(Self::Bdi),
                    [b'b', b'd', b'o'] => Some(Self::Bdo),
                    [b'w', b'b', b'r'] => Some(Self::Wbr),
                    [b'i', b'n', b's'] => Some(Self::Ins),
                    [b'd', b'e', b'l'] => Some(Self::Del),
                    [b'i', b'm', b'g'] => Some(Self::Img),
                    [b'm', b'a', b'p'] => Some(Self::Map),
                    [b's', b'v', b'g'] => Some(Self::Svg),
                    [b'c', b'o', b'l'] => Some(Self::Col),
                    _ => None,
                }
            }
            4 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                ] {
                    [b'h', b't', b'm', b'l'] => Some(Self::Html),
                    [b'h', b'e', b'a', b'd'] => Some(Self::Head),
                    [b'b', b'a', b's', b'e'] => Some(Self::Base),
                    [b'l', b'i', b'n', b'k'] => Some(Self::Link),
                    [b'm', b'e', b't', b'a'] => Some(Self::Meta),
                    [b'b', b'o', b'd', b'y'] => Some(Self::Body),
                    [b'm', b'e', b'n', b'u'] => Some(Self::Menu),
                    [b'm', b'a', b'i', b'n'] => Some(Self::Main),
                    [b'c', b'i', b't', b'e'] => Some(Self::Cite),
                    [b'a', b'b', b'b', b'r'] => Some(Self::Abbr),
                    [b'r', b'u', b'b', b'y'] => Some(Self::Ruby),
                    [b'd', b'a', b't', b'a'] => Some(Self::Data),
                    [b't', b'i', b'm', b'e'] => Some(Self::Time),
                    [b'c', b'o', b'd', b'e'] => Some(Self::Code),
                    [b's', b'a', b'm', b'p'] => Some(Self::Samp),
                    [b'm', b'a', b'r', b'k'] => Some(Self::Mark),
                    [b's', b'p', b'a', b'n'] => Some(Self::Span),
                    [b'a', b'r', b'e', b'a'] => Some(Self::Area),
                    [b'm', b'a', b't', b'h'] => Some(Self::Math),
                    [b'f', b'o', b'r', b'm'] => Some(Self::Form),
                    [b's', b'l', b'o', b't'] => Some(Self::Slot),
                    _ => None,
                }
            }
            5 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                ] {
                    [b't', b'i', b't', b'l', b'e'] => Some(Self::Title),
                    [b's', b't', b'y', b'l', b'e'] => Some(Self::Style),
                    [b'a', b's', b'i', b'd', b'e'] => Some(Self::Aside),
                    [b's', b'm', b'a', b'l', b'l'] => Some(Self::Small),
                    [b'e', b'm', b'b', b'e', b'd'] => Some(Self::Embed),
                    [b'v', b'i', b'd', b'e', b'o'] => Some(Self::Video),
                    [b'a', b'u', b'd', b'i', b'o'] => Some(Self::Audio),
                    [b't', b'r', b'a', b'c', b'k'] => Some(Self::Track),
                    [b't', b'a', b'b', b'l', b'e'] => Some(Self::Table),
                    [b't', b'b', b'o', b'd', b'y'] => Some(Self::Tbody),
                    [b't', b'h', b'e', b'a', b'd'] => Some(Self::Thead),
                    [b't', b'f', b'o', b'o', b't'] => Some(Self::Tfoot),
                    [b'l', b'a', b'b', b'e', b'l'] => Some(Self::Label),
                    [b'i', b'n', b'p', b'u', b't'] => Some(Self::Input),
                    [b'm', b'e', b't', b'e', b'r'] => Some(Self::Meter),
                    _ => None,
                }
            }
            6 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                ] {
                    [b'h', b'g', b'r', b'o', b'u', b'p'] => Some(Self::HGroup),
                    [b'h', b'e', b'a', b'd', b'e', b'r'] => Some(Self::Header),
                    [b'f', b'o', b'o', b't', b'e', b'r'] => Some(Self::Footer),
                    [b'f', b'i', b'g', b'u', b'r', b'e'] => Some(Self::Figure),
                    [b's', b'e', b'a', b'r', b'c', b'h'] => Some(Self::Search),
                    [b's', b't', b'r', b'o', b'n', b'g'] => Some(Self::Strong),
                    [b's', b'o', b'u', b'r', b'c', b'e'] => Some(Self::Source),
                    [b'i', b'f', b'r', b'a', b'm', b'e'] => Some(Self::Iframe),
                    [b'o', b'b', b'j', b'e', b'c', b't'] => Some(Self::Object),
                    [b'b', b'u', b't', b't', b'o', b'n'] => Some(Self::Button),
                    [b's', b'e', b'l', b'e', b'c', b't'] => Some(Self::Select),
                    [b'o', b'p', b't', b'i', b'o', b'n'] => Some(Self::Option),
                    [b'o', b'u', b't', b'p', b'u', b't'] => Some(Self::Output),
                    [b'l', b'e', b'g', b'e', b'n', b'd'] => Some(Self::Legend),
                    [b'd', b'i', b'a', b'l', b'o', b'g'] => Some(Self::Dialog),
                    [b's', b'c', b'r', b'i', b'p', b't'] => Some(Self::Script),
                    [b'c', b'a', b'n', b'v', b'a', b's'] => Some(Self::Canvas),
                    _ => None,
                }
            }
            7 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                ] {
                    [b'a', b'r', b't', b'i', b'c', b'l', b'e'] => Some(Self::Article),
                    [b's', b'e', b'c', b't', b'i', b'o', b'n'] => Some(Self::Section),
                    [b'a', b'd', b'd', b'r', b'e', b's', b's'] => Some(Self::Address),
                    [b'p', b'i', b'c', b't', b'u', b'r', b'e'] => Some(Self::Picture),
                    [b'c', b'a', b'p', b't', b'i', b'o', b'n'] => Some(Self::Caption),
                    [b'd', b'e', b't', b'a', b'i', b'l', b's'] => Some(Self::Details),
                    [b's', b'u', b'm', b'm', b'a', b'r', b'y'] => Some(Self::Summary),
                    _ => None,
                }
            }
            8 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                ] {
                    [b'c', b'o', b'l', b'g', b'r', b'o', b'u', b'p'] => Some(Self::Colgroup),
                    [b'd', b'a', b't', b'a', b'l', b'i', b's', b't'] => Some(Self::Datalist),
                    [b'o', b'p', b't', b'g', b'r', b'o', b'u', b'p'] => Some(Self::Optgroup),
                    [b't', b'e', b'x', b't', b'a', b'r', b'e', b'a'] => Some(Self::Textarea),
                    [b'p', b'r', b'o', b'g', b'r', b'e', b's', b's'] => Some(Self::Progress),
                    [b'f', b'i', b'e', b'l', b'd', b's', b'e', b't'] => Some(Self::Fieldset),
                    [b'n', b'o', b's', b'c', b'r', b'i', b'p', b't'] => Some(Self::Noscript),
                    [b't', b'e', b'm', b'p', b'l', b'a', b't', b'e'] => Some(Self::Template),
                    _ => None,
                }
            }
            10 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                ] {
                    [b'b', b'l', b'o', b'c', b'k', b'q', b'u', b'o', b't', b'e'] => Some(Self::Blockquote),
                    [b'f', b'i', b'g', b'c', b'a', b'p', b't', b'i', b'o', b'n'] => Some(Self::Figcaption),
                    _ => None,
                }
            }
            15 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                    bytes[10].to_ascii_lowercase(),
                    bytes[11].to_ascii_lowercase(),
                    bytes[12].to_ascii_lowercase(),
                    bytes[13].to_ascii_lowercase(),
                    bytes[14].to_ascii_lowercase(),
                ] {
                    [
                        b's',
                        b'e',
                        b'l',
                        b'e',
                        b'c',
                        b't',
                        b'e',
                        b'd',
                        b'c',
                        b'o',
                        b'n',
                        b't',
                        b'e',
                        b'n',
                        b't',
                    ] => Some(Self::Selectedcontent),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Checks if the given tag is a void element.
    /// Void elements are those that do not have any content and do not require a closing tag.
    ///
    /// # Arguments
    /// * `tag` - The HTML tag to check.
    ///
    /// # Returns
    /// `true` if the tag is a void element, `false` otherwise.
    #[must_use]
    pub const fn is_void_element(&self) -> bool {
        matches!(
            self,
            Self::Area
                | Self::Base
                | Self::Br
                | Self::Col
                | Self::Embed
                | Self::Hr
                | Self::Img
                | Self::Input
                | Self::Link
                | Self::Meta
                | Self::Source
                | Self::Track
                | Self::Wbr
        )
    }

    /// A utility function to determine if a tag should automatically close based on the current tag and the new tag being encountered.
    ///
    /// # Arguments
    /// * `current_tag` - The name of the current tag that is open.
    /// * `new_tag` - The name of the new tag that is being encountered.
    ///
    /// # Returns
    /// A boolean indicating whether the current tag should be automatically closed when the new tag is encountered.
    #[must_use]
    pub const fn should_auto_close(&self, new_tag: &Tag) -> bool {
        if let Tag::Html(new_known) = new_tag {
            return self.should_auto_close_known(*new_known);
        }

        false
    }

    /// Determines if a known tag should automatically close based on the current and new known tags.
    #[must_use]
    const fn should_auto_close_known(self, new: Self) -> bool {
        matches!(
            (self, new),
            (
                Self::P,
                Self::Div
                    | Self::P
                    | Self::H1
                    | Self::H2
                    | Self::H3
                    | Self::H4
                    | Self::H5
                    | Self::H6
                    | Self::Ul
                    | Self::Ol
                    | Self::Li
                    | Self::Dl
                    | Self::Dt
                    | Self::Dd
                    | Self::Blockquote
                    | Self::Pre
                    | Self::Form
                    | Self::Table
                    | Self::Section
                    | Self::Article
                    | Self::Aside
                    | Self::Header
                    | Self::Footer
                    | Self::Nav
                    | Self::Main
                    | Self::Figure
                    | Self::Hr
            ) | (Self::Li, Self::Li)
                | (Self::Dd, Self::Dd)
                | (Self::Dt, Self::Dt)
                | (Self::Option, Self::Option | Self::Optgroup)
                | (Self::Tr, Self::Tr)
                | (Self::Td | Self::Th, Self::Td | Self::Th | Self::Tr)
                | (Self::Head, Self::Body)
        )
    }
}

impl Display for HtmlTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Html => "html",
                Self::Head => "head",
                Self::Title => "title",
                Self::Base => "base",
                Self::Link => "link",
                Self::Meta => "meta",
                Self::Style => "style",
                Self::Body => "body",
                Self::Article => "article",
                Self::Section => "section",
                Self::Nav => "nav",
                Self::Aside => "aside",
                Self::H1 => "h1",
                Self::H2 => "h2",
                Self::H3 => "h3",
                Self::H4 => "h4",
                Self::H5 => "h5",
                Self::H6 => "h6",
                Self::HGroup => "hgroup",
                Self::Header => "header",
                Self::Footer => "footer",
                Self::Address => "address",
                Self::P => "p",
                Self::Hr => "hr",
                Self::Pre => "pre",
                Self::Blockquote => "blockquote",
                Self::Ol => "ol",
                Self::Ul => "ul",
                Self::Menu => "menu",
                Self::Li => "li",
                Self::Dl => "dl",
                Self::Dt => "dt",
                Self::Dd => "dd",
                Self::Figure => "figure",
                Self::Figcaption => "figcaption",
                Self::Main => "main",
                Self::Search => "search",
                Self::Div => "div",
                Self::A => "a",
                Self::Em => "em",
                Self::Strong => "strong",
                Self::Small => "small",
                Self::S => "s",
                Self::Cite => "cite",
                Self::Q => "q",
                Self::Dfn => "dfn",
                Self::Abbr => "abbr",
                Self::Ruby => "ruby",
                Self::Rt => "rt",
                Self::Rp => "rp",
                Self::Data => "data",
                Self::Time => "time",
                Self::Code => "code",
                Self::Var => "var",
                Self::Samp => "samp",
                Self::Kbd => "kbd",
                Self::Sub => "sub",
                Self::Sup => "sup",
                Self::I => "i",
                Self::B => "b",
                Self::U => "u",
                Self::Mark => "mark",
                Self::Bdi => "bdi",
                Self::Bdo => "bdo",
                Self::Span => "span",
                Self::Br => "br",
                Self::Wbr => "wbr",
                Self::Ins => "ins",
                Self::Del => "del",
                Self::Picture => "picture",
                Self::Source => "source",
                Self::Img => "img",
                Self::Iframe => "iframe",
                Self::Embed => "embed",
                Self::Object => "object",
                Self::Video => "video",
                Self::Audio => "audio",
                Self::Track => "track",
                Self::Map => "map",
                Self::Area => "area",
                Self::Math => "math",
                Self::Svg => "svg",
                Self::Table => "table",
                Self::Caption => "caption",
                Self::Colgroup => "colgroup",
                Self::Col => "col",
                Self::Tbody => "tbody",
                Self::Thead => "thead",
                Self::Tfoot => "tfoot",
                Self::Tr => "tr",
                Self::Td => "td",
                Self::Th => "th",
                Self::Form => "form",
                Self::Label => "label",
                Self::Input => "input",
                Self::Button => "button",
                Self::Select => "select",
                Self::Datalist => "datalist",
                Self::Optgroup => "optgroup",
                Self::Option => "option",
                Self::Textarea => "textarea",
                Self::Output => "output",
                Self::Progress => "progress",
                Self::Meter => "meter",
                Self::Fieldset => "fieldset",
                Self::Legend => "legend",
                Self::Selectedcontent => "selectedcontent",
                Self::Details => "details",
                Self::Summary => "summary",
                Self::Dialog => "dialog",
                Self::Script => "script",
                Self::Noscript => "noscript",
                Self::Template => "template",
                Self::Slot => "slot",
                Self::Canvas => "canvas",
            }
        )
    }
}

/// Represents SVG tags as an enum.
///
/// This enum includes common SVG tags that are recognized by the parser.
///
/// <https://developer.mozilla.org/en-US/docs/Web/SVG/Reference/Element>
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum SvgTag {
    A,
    Animate,
    AnimateMotion,
    AnimateTransform,
    Circle,
    ClipPath,
    Defs,
    Desc,
    Ellipse,
    FeBlend,
    FeColorMatrix,
    FeComponentTransfer,
    FeComposite,
    FeConvolveMatrix,
    FeDiffuseLighting,
    FeDisplacementMap,
    FeDistantLight,
    FeDropShadow,
    FeFlood,
    FeFuncA,
    FeFuncB,
    FeFuncG,
    FeFuncR,
    FeGaussianBlur,
    FeImage,
    FeMerge,
    FeMergeNode,
    FeMorphology,
    FeOffset,
    FePointLight,
    FeSpecularLighting,
    FeSpotLight,
    FeTile,
    FeTurbulence,
    Filter,
    ForeignObject,
    G,
    Image,
    Line,
    LinearGradient,
    Marker,
    Mask,
    Metadata,
    Mpath,
    Path,
    Pattern,
    Polygon,
    Polyline,
    RadialGradient,
    Rect,
    Script,
    Set,
    Stop,
    Svg,
    Switch,
    Symbol,
    Text,
    TextPath,
    Title,
    Tspan,
    Use,
    View,
}

impl SvgTag {
    #[must_use]
    pub fn from_str_insensitive(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        match bytes.len() {
            1 => match [bytes[0].to_ascii_lowercase()] {
                [b'a'] => Some(Self::A),
                [b'g'] => Some(Self::G),
                _ => None,
            },
            3 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                ] {
                    [b's', b'e', b't'] => Some(Self::Set),
                    [b's', b'v', b'g'] => Some(Self::Svg),
                    [b'u', b's', b'e'] => Some(Self::Use),
                    _ => None,
                }
            }
            4 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                ] {
                    [b'd', b'e', b'f', b's'] => Some(Self::Defs),
                    [b'd', b'e', b's', b'c'] => Some(Self::Desc),
                    [b'l', b'i', b'n', b'e'] => Some(Self::Line),
                    [b'm', b'a', b's', b'k'] => Some(Self::Mask),
                    [b'p', b'a', b't', b'h'] => Some(Self::Path),
                    [b'r', b'e', b'c', b't'] => Some(Self::Rect),
                    [b's', b't', b'o', b'p'] => Some(Self::Stop),
                    [b't', b'e', b'x', b't'] => Some(Self::Text),
                    [b'v', b'i', b'e', b'w'] => Some(Self::View),
                    _ => None,
                }
            }
            5 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                ] {
                    [b'i', b'm', b'a', b'g', b'e'] => Some(Self::Image),
                    [b'm', b'p', b'a', b't', b'h'] => Some(Self::Mpath),
                    [b't', b'i', b't', b'l', b'e'] => Some(Self::Title),
                    [b't', b's', b'p', b'a', b'n'] => Some(Self::Tspan),
                    _ => None,
                }
            }
            6 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                ] {
                    [b'c', b'i', b'r', b'c', b'l', b'e'] => Some(Self::Circle),
                    [b'f', b'e', b't', b'i', b'l', b'e'] => Some(Self::FeTile),
                    [b'f', b'i', b'l', b't', b'e', b'r'] => Some(Self::Filter),
                    [b'm', b'a', b'r', b'k', b'e', b'r'] => Some(Self::Marker),
                    [b's', b'c', b'r', b'i', b'p', b't'] => Some(Self::Script),
                    [b's', b'w', b'i', b't', b'c', b'h'] => Some(Self::Switch),
                    [b's', b'y', b'm', b'b', b'o', b'l'] => Some(Self::Symbol),
                    _ => None,
                }
            }
            7 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                ] {
                    [b'a', b'n', b'i', b'm', b'a', b't', b'e'] => Some(Self::Animate),
                    [b'e', b'l', b'l', b'i', b'p', b's', b'e'] => Some(Self::Ellipse),
                    [b'f', b'e', b'b', b'l', b'e', b'n', b'd'] => Some(Self::FeBlend),
                    [b'f', b'e', b'f', b'l', b'o', b'o', b'd'] => Some(Self::FeFlood),
                    [b'f', b'e', b'f', b'u', b'n', b'c', b'a'] => Some(Self::FeFuncA),
                    [b'f', b'e', b'f', b'u', b'n', b'c', b'b'] => Some(Self::FeFuncB),
                    [b'f', b'e', b'f', b'u', b'n', b'c', b'g'] => Some(Self::FeFuncG),
                    [b'f', b'e', b'f', b'u', b'n', b'c', b'r'] => Some(Self::FeFuncR),
                    [b'f', b'e', b'i', b'm', b'a', b'g', b'e'] => Some(Self::FeImage),
                    [b'f', b'e', b'm', b'e', b'r', b'g', b'e'] => Some(Self::FeMerge),
                    [b'p', b'a', b't', b't', b'e', b'r', b'n'] => Some(Self::Pattern),
                    [b'p', b'o', b'l', b'y', b'g', b'o', b'n'] => Some(Self::Polygon),
                    _ => None,
                }
            }
            8 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                ] {
                    [b'c', b'l', b'i', b'p', b'p', b'a', b't', b'h'] => Some(Self::ClipPath),
                    [b'f', b'e', b'o', b'f', b'f', b's', b'e', b't'] => Some(Self::FeOffset),
                    [b'm', b'e', b't', b'a', b'd', b'a', b't', b'a'] => Some(Self::Metadata),
                    [b'p', b'o', b'l', b'y', b'l', b'i', b'n', b'e'] => Some(Self::Polyline),
                    [b't', b'e', b'x', b't', b'p', b'a', b't', b'h'] => Some(Self::TextPath),
                    _ => None,
                }
            }
            11 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                    bytes[10].to_ascii_lowercase(),
                ] {
                    [
                        b'f',
                        b'e',
                        b'c',
                        b'o',
                        b'm',
                        b'p',
                        b'o',
                        b's',
                        b'i',
                        b't',
                        b'e',
                    ] => Some(Self::FeComposite),
                    [
                        b'f',
                        b'e',
                        b'm',
                        b'e',
                        b'r',
                        b'g',
                        b'e',
                        b'n',
                        b'o',
                        b'd',
                        b'e',
                    ] => Some(Self::FeMergeNode),
                    [
                        b'f',
                        b'e',
                        b's',
                        b'p',
                        b'o',
                        b't',
                        b'l',
                        b'i',
                        b'g',
                        b'h',
                        b't',
                    ] => Some(Self::FeSpotLight),
                    _ => None,
                }
            }
            12 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                    bytes[10].to_ascii_lowercase(),
                    bytes[11].to_ascii_lowercase(),
                ] {
                    [
                        b'f',
                        b'e',
                        b'd',
                        b'r',
                        b'o',
                        b'p',
                        b's',
                        b'h',
                        b'a',
                        b'd',
                        b'o',
                        b'w',
                    ] => Some(Self::FeDropShadow),
                    [
                        b'f',
                        b'e',
                        b'm',
                        b'o',
                        b'r',
                        b'p',
                        b'h',
                        b'o',
                        b'l',
                        b'o',
                        b'g',
                        b'y',
                    ] => Some(Self::FeMorphology),
                    [
                        b'f',
                        b'e',
                        b'p',
                        b'o',
                        b'i',
                        b'n',
                        b't',
                        b'l',
                        b'i',
                        b'g',
                        b'h',
                        b't',
                    ] => Some(Self::FePointLight),
                    [
                        b'f',
                        b'e',
                        b't',
                        b'u',
                        b'r',
                        b'b',
                        b'u',
                        b'l',
                        b'e',
                        b'n',
                        b'c',
                        b'e',
                    ] => Some(Self::FeTurbulence),
                    _ => None,
                }
            }
            13 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                    bytes[10].to_ascii_lowercase(),
                    bytes[11].to_ascii_lowercase(),
                    bytes[12].to_ascii_lowercase(),
                ] {
                    [
                        b'a',
                        b'n',
                        b'i',
                        b'm',
                        b'a',
                        b't',
                        b'e',
                        b'm',
                        b'o',
                        b't',
                        b'i',
                        b'o',
                        b'n',
                    ] => Some(Self::AnimateMotion),
                    [
                        b'f',
                        b'e',
                        b'c',
                        b'o',
                        b'l',
                        b'o',
                        b'r',
                        b'm',
                        b'a',
                        b't',
                        b'r',
                        b'i',
                        b'x',
                    ] => Some(Self::FeColorMatrix),
                    [
                        b'f',
                        b'o',
                        b'r',
                        b'e',
                        b'i',
                        b'g',
                        b'n',
                        b'o',
                        b'b',
                        b'j',
                        b'e',
                        b'c',
                        b't',
                    ] => Some(Self::ForeignObject),
                    _ => None,
                }
            }
            14 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                    bytes[10].to_ascii_lowercase(),
                    bytes[11].to_ascii_lowercase(),
                    bytes[12].to_ascii_lowercase(),
                    bytes[13].to_ascii_lowercase(),
                ] {
                    [
                        b'f',
                        b'e',
                        b'd',
                        b'i',
                        b's',
                        b't',
                        b'a',
                        b'n',
                        b't',
                        b'l',
                        b'i',
                        b'g',
                        b'h',
                        b't',
                    ] => Some(Self::FeDistantLight),
                    [
                        b'f',
                        b'e',
                        b'g',
                        b'a',
                        b'u',
                        b's',
                        b's',
                        b'i',
                        b'a',
                        b'n',
                        b'b',
                        b'l',
                        b'u',
                        b'r',
                    ] => Some(Self::FeGaussianBlur),
                    [
                        b'l',
                        b'i',
                        b'n',
                        b'e',
                        b'a',
                        b'r',
                        b'g',
                        b'r',
                        b'a',
                        b'd',
                        b'i',
                        b'e',
                        b'n',
                        b't',
                    ] => Some(Self::LinearGradient),
                    [
                        b'r',
                        b'a',
                        b'd',
                        b'i',
                        b'a',
                        b'l',
                        b'g',
                        b'r',
                        b'a',
                        b'd',
                        b'i',
                        b'e',
                        b'n',
                        b't',
                    ] => Some(Self::RadialGradient),
                    _ => None,
                }
            }
            16 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                    bytes[10].to_ascii_lowercase(),
                    bytes[11].to_ascii_lowercase(),
                    bytes[12].to_ascii_lowercase(),
                    bytes[13].to_ascii_lowercase(),
                    bytes[14].to_ascii_lowercase(),
                    bytes[15].to_ascii_lowercase(),
                ] {
                    [
                        b'a',
                        b'n',
                        b'i',
                        b'm',
                        b'a',
                        b't',
                        b'e',
                        b't',
                        b'r',
                        b'a',
                        b'n',
                        b's',
                        b'f',
                        b'o',
                        b'r',
                        b'm',
                    ] => Some(Self::AnimateTransform),
                    [
                        b'f',
                        b'e',
                        b'c',
                        b'o',
                        b'n',
                        b'v',
                        b'o',
                        b'l',
                        b'v',
                        b'e',
                        b'm',
                        b'a',
                        b't',
                        b'r',
                        b'i',
                        b'x',
                    ] => Some(Self::FeConvolveMatrix),
                    _ => None,
                }
            }
            17 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                    bytes[10].to_ascii_lowercase(),
                    bytes[11].to_ascii_lowercase(),
                    bytes[12].to_ascii_lowercase(),
                    bytes[13].to_ascii_lowercase(),
                    bytes[14].to_ascii_lowercase(),
                    bytes[15].to_ascii_lowercase(),
                    bytes[16].to_ascii_lowercase(),
                ] {
                    [
                        b'f',
                        b'e',
                        b'd',
                        b'i',
                        b'f',
                        b'f',
                        b'u',
                        b's',
                        b'e',
                        b'l',
                        b'i',
                        b'g',
                        b'h',
                        b't',
                        b'i',
                        b'n',
                        b'g',
                    ] => Some(Self::FeDiffuseLighting),
                    [
                        b'f',
                        b'e',
                        b'd',
                        b'i',
                        b's',
                        b'p',
                        b'l',
                        b'a',
                        b'c',
                        b'e',
                        b'm',
                        b'e',
                        b'n',
                        b't',
                        b'm',
                        b'a',
                        b'p',
                    ] => Some(Self::FeDisplacementMap),
                    _ => None,
                }
            }
            18 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                    bytes[10].to_ascii_lowercase(),
                    bytes[11].to_ascii_lowercase(),
                    bytes[12].to_ascii_lowercase(),
                    bytes[13].to_ascii_lowercase(),
                    bytes[14].to_ascii_lowercase(),
                    bytes[15].to_ascii_lowercase(),
                    bytes[16].to_ascii_lowercase(),
                    bytes[17].to_ascii_lowercase(),
                ] {
                    [
                        b'f',
                        b'e',
                        b's',
                        b'p',
                        b'e',
                        b'c',
                        b'u',
                        b'l',
                        b'a',
                        b'r',
                        b'l',
                        b'i',
                        b'g',
                        b'h',
                        b't',
                        b'i',
                        b'n',
                        b'g',
                    ] => Some(Self::FeSpecularLighting),
                    _ => None,
                }
            }
            19 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                    bytes[10].to_ascii_lowercase(),
                    bytes[11].to_ascii_lowercase(),
                    bytes[12].to_ascii_lowercase(),
                    bytes[13].to_ascii_lowercase(),
                    bytes[14].to_ascii_lowercase(),
                    bytes[15].to_ascii_lowercase(),
                    bytes[16].to_ascii_lowercase(),
                    bytes[17].to_ascii_lowercase(),
                    bytes[18].to_ascii_lowercase(),
                ] {
                    [
                        b'f',
                        b'e',
                        b'c',
                        b'o',
                        b'm',
                        b'p',
                        b'o',
                        b'n',
                        b'e',
                        b'n',
                        b't',
                        b't',
                        b'r',
                        b'a',
                        b'n',
                        b's',
                        b'f',
                        b'e',
                        b'r',
                    ] => Some(Self::FeComponentTransfer),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_void_element(self) -> bool {
        matches!(
            self,
            Self::Circle
                | Self::Ellipse
                | Self::Line
                | Self::Path
                | Self::Polygon
                | Self::Polyline
                | Self::Rect
                | Self::Stop
                | Self::Use
                | Self::Image
                | Self::FeBlend
                | Self::FeColorMatrix
                | Self::FeComponentTransfer
                | Self::FeComposite
                | Self::FeConvolveMatrix
                | Self::FeDiffuseLighting
                | Self::FeDisplacementMap
                | Self::FeDistantLight
                | Self::FeDropShadow
                | Self::FeFlood
                | Self::FeFuncA
                | Self::FeFuncB
                | Self::FeFuncG
                | Self::FeFuncR
                | Self::FeGaussianBlur
                | Self::FeImage
                | Self::FeMergeNode
                | Self::FeMorphology
                | Self::FeOffset
                | Self::FePointLight
                | Self::FeSpecularLighting
                | Self::FeSpotLight
                | Self::FeTile
                | Self::FeTurbulence
                | Self::Animate
                | Self::AnimateMotion
                | Self::AnimateTransform
                | Self::Mpath
                | Self::Set
        )
    }
}

impl Display for SvgTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::A => "a",
                Self::Animate => "animate",
                Self::AnimateMotion => "animateMotion",
                Self::AnimateTransform => "animateTransform",
                Self::Circle => "circle",
                Self::ClipPath => "clipPath",
                Self::Defs => "defs",
                Self::Desc => "desc",
                Self::Ellipse => "ellipse",
                Self::FeBlend => "feBlend",
                Self::FeColorMatrix => "feColorMatrix",
                Self::FeComponentTransfer => "feComponentTransfer",
                Self::FeComposite => "feComposite",
                Self::FeConvolveMatrix => "feConvolveMatrix",
                Self::FeDiffuseLighting => "feDiffuseLighting",
                Self::FeDisplacementMap => "feDisplacementMap",
                Self::FeDistantLight => "feDistantLight",
                Self::FeDropShadow => "feDropShadow",
                Self::FeFlood => "feFlood",
                Self::FeFuncA => "feFuncA",
                Self::FeFuncB => "feFuncB",
                Self::FeFuncG => "feFuncG",
                Self::FeFuncR => "feFuncR",
                Self::FeGaussianBlur => "feGaussianBlur",
                Self::FeImage => "feImage",
                Self::FeMerge => "feMerge",
                Self::FeMergeNode => "feMergeNode",
                Self::FeMorphology => "feMorphology",
                Self::FeOffset => "feOffset",
                Self::FePointLight => "fePointLight",
                Self::FeSpecularLighting => "feSpecularLighting",
                Self::FeSpotLight => "feSpotLight",
                Self::FeTile => "feTile",
                Self::FeTurbulence => "feTurbulence",
                Self::Filter => "filter",
                Self::ForeignObject => "foreignObject",
                Self::G => "g",
                Self::Image => "image",
                Self::Line => "line",
                Self::LinearGradient => "linearGradient",
                Self::Marker => "marker",
                Self::Mask => "mask",
                Self::Metadata => "metadata",
                Self::Mpath => "mpath",
                Self::Path => "path",
                Self::Pattern => "pattern",
                Self::Polygon => "polygon",
                Self::Polyline => "polyline",
                Self::RadialGradient => "radialGradient",
                Self::Rect => "rect",
                Self::Script => "script",
                Self::Set => "set",
                Self::Stop => "stop",
                Self::Svg => "svg",
                Self::Switch => "switch",
                Self::Symbol => "symbol",
                Self::Text => "text",
                Self::TextPath => "textPath",
                Self::Title => "title",
                Self::Tspan => "tspan",
                Self::Use => "use",
                Self::View => "view",
            }
        )
    }
}

/// Represents the tag name of a MathML tag
///
/// <https://w3c.github.io/mathml-core/#mathml-elements-and-attributes>
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum MathTag {
    A,
    Annotation,
    AnnotationXML,
    Maction,
    Math,
    Merror,
    Mfrac,
    Mi,
    Mmultiscripts,
    Mn,
    Mo,
    Mover,
    Mpadded,
    Mphantom,
    Mprescripts,
    Mroot,
    Mrow,
    Ms,
    Mspace,
    Msqrt,
    Mstyle,
    Msub,
    Msubsup,
    Msup,
    Mtable,
    Mtd,
    Mtext,
    Mtr,
    Munder,
    Munderover,
    Semantics,
}

impl MathTag {
    pub fn from_str_insensitive(s: &str) -> Option<Self> {
        {
            let bytes = s.as_bytes();
            match bytes.len() {
                2 => match [bytes[0].to_ascii_lowercase(), bytes[1].to_ascii_lowercase()] {
                    [b'm', b'i'] => Some(MathTag::Mi),
                    [b'm', b'o'] => Some(MathTag::Mo),
                    [b'm', b'n'] => Some(MathTag::Mn),
                    [b'm', b's'] => Some(MathTag::Ms),
                    _ => None,
                },
                3 => {
                    match [
                        bytes[0].to_ascii_lowercase(),
                        bytes[1].to_ascii_lowercase(),
                        bytes[2].to_ascii_lowercase(),
                    ] {
                        [b'm', b't', b'd'] => Some(MathTag::Mtd),
                        [b'm', b't', b'r'] => Some(MathTag::Mtr),
                        _ => None,
                    }
                }
                4 => {
                    match [
                        bytes[0].to_ascii_lowercase(),
                        bytes[1].to_ascii_lowercase(),
                        bytes[2].to_ascii_lowercase(),
                        bytes[3].to_ascii_lowercase(),
                    ] {
                        [b'm', b'r', b'o', b'w'] => Some(MathTag::Mrow),
                        [b'm', b's', b'u', b'b'] => Some(MathTag::Msub),
                        [b'm', b's', b'u', b'p'] => Some(MathTag::Msup),
                        _ => None,
                    }
                }
                5 => {
                    match [
                        bytes[0].to_ascii_lowercase(),
                        bytes[1].to_ascii_lowercase(),
                        bytes[2].to_ascii_lowercase(),
                        bytes[3].to_ascii_lowercase(),
                        bytes[4].to_ascii_lowercase(),
                    ] {
                        [b'm', b'f', b'r', b'a', b'c'] => Some(MathTag::Mfrac),
                        [b'm', b'o', b'v', b'e', b'r'] => Some(MathTag::Mover),
                        [b'm', b'r', b'o', b'o', b't'] => Some(MathTag::Mroot),
                        [b'm', b's', b'q', b'r', b't'] => Some(MathTag::Msqrt),
                        [b'm', b't', b'e', b'x', b't'] => Some(MathTag::Mtext),
                        _ => None,
                    }
                }
                6 => {
                    match [
                        bytes[0].to_ascii_lowercase(),
                        bytes[1].to_ascii_lowercase(),
                        bytes[2].to_ascii_lowercase(),
                        bytes[3].to_ascii_lowercase(),
                        bytes[4].to_ascii_lowercase(),
                        bytes[5].to_ascii_lowercase(),
                    ] {
                        [b'm', b's', b'p', b'a', b'c', b'e'] => Some(MathTag::Mspace),
                        [b'm', b's', b't', b'y', b'l', b'e'] => Some(MathTag::Mstyle),
                        [b'm', b't', b'a', b'b', b'l', b'e'] => Some(MathTag::Mtable),
                        [b'm', b'u', b'n', b'd', b'e', b'r'] => Some(MathTag::Munder),
                        [b'm', b'e', b'r', b'r', b'o', b'r'] => Some(MathTag::Merror),
                        _ => None,
                    }
                }
                7 => {
                    match [
                        bytes[0].to_ascii_lowercase(),
                        bytes[1].to_ascii_lowercase(),
                        bytes[2].to_ascii_lowercase(),
                        bytes[3].to_ascii_lowercase(),
                        bytes[4].to_ascii_lowercase(),
                        bytes[5].to_ascii_lowercase(),
                        bytes[6].to_ascii_lowercase(),
                    ] {
                        [b'm', b'p', b'a', b'd', b'd', b'e', b'd'] => Some(MathTag::Mpadded),
                        [b'm', b's', b'u', b'b', b's', b'u', b'p'] => Some(MathTag::Msubsup),
                        _ => None,
                    }
                }
                8 => {
                    match [
                        bytes[0].to_ascii_lowercase(),
                        bytes[1].to_ascii_lowercase(),
                        bytes[2].to_ascii_lowercase(),
                        bytes[3].to_ascii_lowercase(),
                        bytes[4].to_ascii_lowercase(),
                        bytes[5].to_ascii_lowercase(),
                        bytes[6].to_ascii_lowercase(),
                        bytes[7].to_ascii_lowercase(),
                    ] {
                        [b'm', b'p', b'h', b'a', b'n', b't', b'o', b'm'] => Some(MathTag::Mphantom),
                        _ => None,
                    }
                }
                9 => {
                    match [
                        bytes[0].to_ascii_lowercase(),
                        bytes[1].to_ascii_lowercase(),
                        bytes[2].to_ascii_lowercase(),
                        bytes[3].to_ascii_lowercase(),
                        bytes[4].to_ascii_lowercase(),
                        bytes[5].to_ascii_lowercase(),
                        bytes[6].to_ascii_lowercase(),
                        bytes[7].to_ascii_lowercase(),
                        bytes[8].to_ascii_lowercase(),
                    ] {
                        [b's', b'e', b'm', b'a', b'n', b't', b'i', b'c', b's'] => Some(MathTag::Semantics),
                        _ => None,
                    }
                }
                10 => {
                    match [
                        bytes[0].to_ascii_lowercase(),
                        bytes[1].to_ascii_lowercase(),
                        bytes[2].to_ascii_lowercase(),
                        bytes[3].to_ascii_lowercase(),
                        bytes[4].to_ascii_lowercase(),
                        bytes[5].to_ascii_lowercase(),
                        bytes[6].to_ascii_lowercase(),
                        bytes[7].to_ascii_lowercase(),
                        bytes[8].to_ascii_lowercase(),
                        bytes[9].to_ascii_lowercase(),
                    ] {
                        [b'a', b'n', b'n', b'o', b't', b'a', b't', b'i', b'o', b'n'] => Some(MathTag::Annotation),
                        [b'm', b'u', b'n', b'd', b'e', b'r', b'o', b'v', b'e', b'r'] => Some(MathTag::Munderover),
                        _ => None,
                    }
                }
                11 => {
                    match [
                        bytes[0].to_ascii_lowercase(),
                        bytes[1].to_ascii_lowercase(),
                        bytes[2].to_ascii_lowercase(),
                        bytes[3].to_ascii_lowercase(),
                        bytes[4].to_ascii_lowercase(),
                        bytes[5].to_ascii_lowercase(),
                        bytes[6].to_ascii_lowercase(),
                        bytes[7].to_ascii_lowercase(),
                        bytes[8].to_ascii_lowercase(),
                        bytes[9].to_ascii_lowercase(),
                        bytes[10].to_ascii_lowercase(),
                    ] {
                        [
                            b'm',
                            b'p',
                            b'r',
                            b'e',
                            b's',
                            b'c',
                            b'r',
                            b'i',
                            b'p',
                            b't',
                            b's',
                        ] => Some(MathTag::Mprescripts),
                        _ => None,
                    }
                }
                13 => {
                    match [
                        bytes[0].to_ascii_lowercase(),
                        bytes[1].to_ascii_lowercase(),
                        bytes[2].to_ascii_lowercase(),
                        bytes[3].to_ascii_lowercase(),
                        bytes[4].to_ascii_lowercase(),
                        bytes[5].to_ascii_lowercase(),
                        bytes[6].to_ascii_lowercase(),
                        bytes[7].to_ascii_lowercase(),
                        bytes[8].to_ascii_lowercase(),
                        bytes[9].to_ascii_lowercase(),
                        bytes[10].to_ascii_lowercase(),
                        bytes[11].to_ascii_lowercase(),
                        bytes[12].to_ascii_lowercase(),
                    ] {
                        [
                            b'a',
                            b'n',
                            b'n',
                            b'o',
                            b't',
                            b'a',
                            b't',
                            b'i',
                            b'o',
                            b'n',
                            b'x',
                            b'm',
                            b'l',
                        ] => Some(MathTag::AnnotationXML),
                        [
                            b'm',
                            b'm',
                            b'u',
                            b'l',
                            b't',
                            b'i',
                            b's',
                            b'c',
                            b'r',
                            b'i',
                            b'p',
                            b't',
                            b's',
                        ] => Some(MathTag::Mmultiscripts),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
    }
}

impl Display for MathTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MathTag::A => "a",
                MathTag::Annotation => "annotation",
                MathTag::AnnotationXML => "annotationxml",
                MathTag::Maction => "maction",
                MathTag::Math => "math",
                MathTag::Merror => "merror",
                MathTag::Mfrac => "mfrac",
                MathTag::Mi => "mi",
                MathTag::Mmultiscripts => "mmultiscripts",
                MathTag::Mn => "mn",
                MathTag::Mo => "mo",
                MathTag::Mover => "mover",
                MathTag::Mpadded => "mpadded",
                MathTag::Mphantom => "mphantom",
                MathTag::Mprescripts => "mprescripts",
                MathTag::Mroot => "mroot",
                MathTag::Mrow => "mrow",
                MathTag::Ms => "ms",
                MathTag::Mspace => "mspace",
                MathTag::Msqrt => "msqrt",
                MathTag::Mstyle => "mstyle",
                MathTag::Msub => "msub",
                MathTag::Msubsup => "msubsup",
                MathTag::Msup => "msup",
                MathTag::Mtable => "mtable",
                MathTag::Mtd => "mtd",
                MathTag::Mtext => "mtext",
                MathTag::Mtr => "mtr",
                MathTag::Munder => "munder",
                MathTag::Munderover => "munderover",
                MathTag::Semantics => "semantics",
            }
        )
    }
}
