use std::fmt::Display;

/// Represents an HTML tag, which can be either a known tag or an unknown tag.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tag {
    /// Represents a known HTML tag defined in the `HtmlTag` enum.
    Html(HtmlTag),

    /// Represents a known SVG tag defined in the `SvgTag` enum.
    Svg(SvgTag),

    /// Represents an unknown tag, where the string is the tag name, for instance custom tags like `<yt-thumbnail-view-model>`.
    Unknown(String),
}

impl Tag {
    pub fn from_str_insensitive(s: &str) -> Self {
        if let Some(html_tag) = HtmlTag::from_str_insensitive(s) {
            Tag::Html(html_tag)
        } else if let Some(svg_tag) = SvgTag::from_str_insensitive(s) {
            Tag::Svg(svg_tag)
        } else {
            Tag::Unknown(s.to_string())
        }
    }

    /// Checks if the tag is a void element.
    pub fn is_void_element(&self) -> bool {
        match self {
            Tag::Html(html_tag) => html_tag.is_void_element(),
            Tag::Svg(svg_tag) => svg_tag.is_void_element(),
            Tag::Unknown(_) => false,
        }
    }

    /// Determines if the current tag should automatically close based on the new tag being encountered.
    pub fn should_auto_close(&self, new_tag: &Tag) -> bool {
        match self {
            Tag::Html(html_tag) => html_tag.should_auto_close(new_tag),
            Tag::Svg(_) => false,
            Tag::Unknown(_) => false,
        }
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Tag::Html(html_tag) => html_tag.to_string(),
                Tag::Svg(svg_tag) => svg_tag.to_string(),
                Tag::Unknown(name) => name.clone(),
            }
        )
    }
}

/// Represents known HTML tags as an enum.
///
/// This enum includes common HTML tags that are recognized by the parser.
///
/// <https://html.spec.whatwg.org/multipage/#toc-semantics>
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum HtmlTag {
    Html,
    Head,
    Title,
    Base,
    Link,
    Meta,
    Style,
    Body,
    Article,
    Section,
    Nav,
    Aside,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    HGroup,
    Header,
    Footer,
    Address,
    P,
    Hr,
    Pre,
    Blockquote,
    Ol,
    Ul,
    Menu,
    Li,
    Dl,
    Dt,
    Dd,
    Figure,
    Figcaption,
    Main,
    Search,
    Div,
    A,
    Em,
    Strong,
    Small,
    S,
    Cite,
    Q,
    Dfn,
    Abbr,
    Ruby,
    Rt,
    Rp,
    Data,
    Time,
    Code,
    Var,
    Samp,
    Kbd,
    Sub,
    Sup,
    I,
    B,
    U,
    Mark,
    Bdi,
    Bdo,
    Span,
    Br,
    Wbr,
    Ins,
    Del,
    Picture,
    Source,
    Img,
    Iframe,
    Embed,
    Object,
    Video,
    Audio,
    Track,
    Map,
    Area,
    Math,
    Annotation,
    AnnotationXML,
    Mi,
    Mo,
    Mn,
    Ms,
    Mfrac,
    Mmultiscripts,
    Mover,
    Mpadded,
    Mphantom,
    Mprescripts,
    Mroot,
    Mrow,
    Mspace,
    Msqrt,
    Mstyle,
    Msub,
    Msubsup,
    Msup,
    Mtable,
    Mtd,
    Mtr,
    Munder,
    Munderover,
    Semantics,
    Mtext,
    Merror,
    Svg,
    Table,
    Caption,
    Colgroup,
    Col,
    Tbody,
    Thead,
    Tfoot,
    Tr,
    Td,
    Th,
    Form,
    Label,
    Input,
    Button,
    Select,
    Datalist,
    Optgroup,
    Option,
    Textarea,
    Output,
    Progress,
    Meter,
    Fieldset,
    Legend,
    Selectedcontent,
    Details,
    Summary,
    Dialog,
    Script,
    Noscript,
    Template,
    Slot,
    Canvas,
}

impl HtmlTag {
    pub fn from_str_insensitive(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        match bytes.len() {
            1 => match [bytes[0].to_ascii_lowercase()] {
                [b'p'] => Some(HtmlTag::P),
                [b'a'] => Some(HtmlTag::A),
                [b's'] => Some(HtmlTag::S),
                [b'q'] => Some(HtmlTag::Q),
                [b'i'] => Some(HtmlTag::I),
                [b'b'] => Some(HtmlTag::B),
                [b'u'] => Some(HtmlTag::U),
                _ => None,
            },
            2 => match [bytes[0].to_ascii_lowercase(), bytes[1].to_ascii_lowercase()] {
                [b'h', b'1'] => Some(HtmlTag::H1),
                [b'h', b'2'] => Some(HtmlTag::H2),
                [b'h', b'3'] => Some(HtmlTag::H3),
                [b'h', b'4'] => Some(HtmlTag::H4),
                [b'h', b'5'] => Some(HtmlTag::H5),
                [b'h', b'6'] => Some(HtmlTag::H6),
                [b'h', b'r'] => Some(HtmlTag::Hr),
                [b'o', b'l'] => Some(HtmlTag::Ol),
                [b'u', b'l'] => Some(HtmlTag::Ul),
                [b'l', b'i'] => Some(HtmlTag::Li),
                [b'd', b'l'] => Some(HtmlTag::Dl),
                [b'd', b't'] => Some(HtmlTag::Dt),
                [b'd', b'd'] => Some(HtmlTag::Dd),
                [b'e', b'm'] => Some(HtmlTag::Em),
                [b'r', b't'] => Some(HtmlTag::Rt),
                [b'r', b'p'] => Some(HtmlTag::Rp),
                [b'b', b'r'] => Some(HtmlTag::Br),
                [b'm', b'i'] => Some(HtmlTag::Mi),
                [b'm', b'o'] => Some(HtmlTag::Mo),
                [b'm', b'n'] => Some(HtmlTag::Mn),
                [b'm', b's'] => Some(HtmlTag::Ms),
                [b't', b'r'] => Some(HtmlTag::Tr),
                [b't', b'd'] => Some(HtmlTag::Td),
                [b't', b'h'] => Some(HtmlTag::Th),
                _ => None,
            },
            3 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                ] {
                    [b'n', b'a', b'v'] => Some(HtmlTag::Nav),
                    [b'p', b'r', b'e'] => Some(HtmlTag::Pre),
                    [b'd', b'i', b'v'] => Some(HtmlTag::Div),
                    [b'd', b'f', b'n'] => Some(HtmlTag::Dfn),
                    [b'v', b'a', b'r'] => Some(HtmlTag::Var),
                    [b'k', b'b', b'd'] => Some(HtmlTag::Kbd),
                    [b's', b'u', b'b'] => Some(HtmlTag::Sub),
                    [b's', b'u', b'p'] => Some(HtmlTag::Sup),
                    [b'b', b'd', b'i'] => Some(HtmlTag::Bdi),
                    [b'b', b'd', b'o'] => Some(HtmlTag::Bdo),
                    [b'w', b'b', b'r'] => Some(HtmlTag::Wbr),
                    [b'i', b'n', b's'] => Some(HtmlTag::Ins),
                    [b'd', b'e', b'l'] => Some(HtmlTag::Del),
                    [b'i', b'm', b'g'] => Some(HtmlTag::Img),
                    [b'm', b'a', b'p'] => Some(HtmlTag::Map),
                    [b'm', b't', b'd'] => Some(HtmlTag::Mtd),
                    [b'm', b't', b'r'] => Some(HtmlTag::Mtr),
                    [b's', b'v', b'g'] => Some(HtmlTag::Svg),
                    [b'c', b'o', b'l'] => Some(HtmlTag::Col),
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
                    [b'h', b't', b'm', b'l'] => Some(HtmlTag::Html),
                    [b'h', b'e', b'a', b'd'] => Some(HtmlTag::Head),
                    [b'b', b'a', b's', b'e'] => Some(HtmlTag::Base),
                    [b'l', b'i', b'n', b'k'] => Some(HtmlTag::Link),
                    [b'm', b'e', b't', b'a'] => Some(HtmlTag::Meta),
                    [b'b', b'o', b'd', b'y'] => Some(HtmlTag::Body),
                    [b'm', b'e', b'n', b'u'] => Some(HtmlTag::Menu),
                    [b'm', b'a', b'i', b'n'] => Some(HtmlTag::Main),
                    [b'c', b'i', b't', b'e'] => Some(HtmlTag::Cite),
                    [b'a', b'b', b'b', b'r'] => Some(HtmlTag::Abbr),
                    [b'r', b'u', b'b', b'y'] => Some(HtmlTag::Ruby),
                    [b'd', b'a', b't', b'a'] => Some(HtmlTag::Data),
                    [b't', b'i', b'm', b'e'] => Some(HtmlTag::Time),
                    [b'c', b'o', b'd', b'e'] => Some(HtmlTag::Code),
                    [b's', b'a', b'm', b'p'] => Some(HtmlTag::Samp),
                    [b'm', b'a', b'r', b'k'] => Some(HtmlTag::Mark),
                    [b's', b'p', b'a', b'n'] => Some(HtmlTag::Span),
                    [b'a', b'r', b'e', b'a'] => Some(HtmlTag::Area),
                    [b'm', b'a', b't', b'h'] => Some(HtmlTag::Math),
                    [b'm', b'r', b'o', b'w'] => Some(HtmlTag::Mrow),
                    [b'm', b's', b'u', b'b'] => Some(HtmlTag::Msub),
                    [b'm', b's', b'u', b'p'] => Some(HtmlTag::Msup),
                    [b'f', b'o', b'r', b'm'] => Some(HtmlTag::Form),
                    [b's', b'l', b'o', b't'] => Some(HtmlTag::Slot),
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
                    [b't', b'i', b't', b'l', b'e'] => Some(HtmlTag::Title),
                    [b's', b't', b'y', b'l', b'e'] => Some(HtmlTag::Style),
                    [b'a', b's', b'i', b'd', b'e'] => Some(HtmlTag::Aside),
                    [b's', b'm', b'a', b'l', b'l'] => Some(HtmlTag::Small),
                    [b'e', b'm', b'b', b'e', b'd'] => Some(HtmlTag::Embed),
                    [b'v', b'i', b'd', b'e', b'o'] => Some(HtmlTag::Video),
                    [b'a', b'u', b'd', b'i', b'o'] => Some(HtmlTag::Audio),
                    [b't', b'r', b'a', b'c', b'k'] => Some(HtmlTag::Track),
                    [b'm', b'f', b'r', b'a', b'c'] => Some(HtmlTag::Mfrac),
                    [b'm', b'o', b'v', b'e', b'r'] => Some(HtmlTag::Mover),
                    [b'm', b'r', b'o', b'o', b't'] => Some(HtmlTag::Mroot),
                    [b'm', b's', b'q', b'r', b't'] => Some(HtmlTag::Msqrt),
                    [b'm', b't', b'e', b'x', b't'] => Some(HtmlTag::Mtext),
                    [b't', b'a', b'b', b'l', b'e'] => Some(HtmlTag::Table),
                    [b't', b'b', b'o', b'd', b'y'] => Some(HtmlTag::Tbody),
                    [b't', b'h', b'e', b'a', b'd'] => Some(HtmlTag::Thead),
                    [b't', b'f', b'o', b'o', b't'] => Some(HtmlTag::Tfoot),
                    [b'l', b'a', b'b', b'e', b'l'] => Some(HtmlTag::Label),
                    [b'i', b'n', b'p', b'u', b't'] => Some(HtmlTag::Input),
                    [b'm', b'e', b't', b'e', b'r'] => Some(HtmlTag::Meter),
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
                    [b'h', b'g', b'r', b'o', b'u', b'p'] => Some(HtmlTag::HGroup),
                    [b'h', b'e', b'a', b'd', b'e', b'r'] => Some(HtmlTag::Header),
                    [b'f', b'o', b'o', b't', b'e', b'r'] => Some(HtmlTag::Footer),
                    [b'f', b'i', b'g', b'u', b'r', b'e'] => Some(HtmlTag::Figure),
                    [b's', b'e', b'a', b'r', b'c', b'h'] => Some(HtmlTag::Search),
                    [b's', b't', b'r', b'o', b'n', b'g'] => Some(HtmlTag::Strong),
                    [b's', b'o', b'u', b'r', b'c', b'e'] => Some(HtmlTag::Source),
                    [b'i', b'f', b'r', b'a', b'm', b'e'] => Some(HtmlTag::Iframe),
                    [b'o', b'b', b'j', b'e', b'c', b't'] => Some(HtmlTag::Object),
                    [b'm', b's', b'p', b'a', b'c', b'e'] => Some(HtmlTag::Mspace),
                    [b'm', b's', b't', b'y', b'l', b'e'] => Some(HtmlTag::Mstyle),
                    [b'm', b't', b'a', b'b', b'l', b'e'] => Some(HtmlTag::Mtable),
                    [b'm', b'u', b'n', b'd', b'e', b'r'] => Some(HtmlTag::Munder),
                    [b'm', b'e', b'r', b'r', b'o', b'r'] => Some(HtmlTag::Merror),
                    [b'b', b'u', b't', b't', b'o', b'n'] => Some(HtmlTag::Button),
                    [b's', b'e', b'l', b'e', b'c', b't'] => Some(HtmlTag::Select),
                    [b'o', b'p', b't', b'i', b'o', b'n'] => Some(HtmlTag::Option),
                    [b'o', b'u', b't', b'p', b'u', b't'] => Some(HtmlTag::Output),
                    [b'l', b'e', b'g', b'e', b'n', b'd'] => Some(HtmlTag::Legend),
                    [b'd', b'i', b'a', b'l', b'o', b'g'] => Some(HtmlTag::Dialog),
                    [b's', b'c', b'r', b'i', b'p', b't'] => Some(HtmlTag::Script),
                    [b'c', b'a', b'n', b'v', b'a', b's'] => Some(HtmlTag::Canvas),
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
                    [b'a', b'r', b't', b'i', b'c', b'l', b'e'] => Some(HtmlTag::Article),
                    [b's', b'e', b'c', b't', b'i', b'o', b'n'] => Some(HtmlTag::Section),
                    [b'a', b'd', b'd', b'r', b'e', b's', b's'] => Some(HtmlTag::Address),
                    [b'p', b'i', b'c', b't', b'u', b'r', b'e'] => Some(HtmlTag::Picture),
                    [b'm', b'p', b'a', b'd', b'd', b'e', b'd'] => Some(HtmlTag::Mpadded),
                    [b'm', b's', b'u', b'b', b's', b'u', b'p'] => Some(HtmlTag::Msubsup),
                    [b'c', b'a', b'p', b't', b'i', b'o', b'n'] => Some(HtmlTag::Caption),
                    [b'd', b'e', b't', b'a', b'i', b'l', b's'] => Some(HtmlTag::Details),
                    [b's', b'u', b'm', b'm', b'a', b'r', b'y'] => Some(HtmlTag::Summary),
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
                    [b'm', b'p', b'h', b'a', b'n', b't', b'o', b'm'] => Some(HtmlTag::Mphantom),
                    [b'c', b'o', b'l', b'g', b'r', b'o', b'u', b'p'] => Some(HtmlTag::Colgroup),
                    [b'd', b'a', b't', b'a', b'l', b'i', b's', b't'] => Some(HtmlTag::Datalist),
                    [b'o', b'p', b't', b'g', b'r', b'o', b'u', b'p'] => Some(HtmlTag::Optgroup),
                    [b't', b'e', b'x', b't', b'a', b'r', b'e', b'a'] => Some(HtmlTag::Textarea),
                    [b'p', b'r', b'o', b'g', b'r', b'e', b's', b's'] => Some(HtmlTag::Progress),
                    [b'f', b'i', b'e', b'l', b'd', b's', b'e', b't'] => Some(HtmlTag::Fieldset),
                    [b'n', b'o', b's', b'c', b'r', b'i', b'p', b't'] => Some(HtmlTag::Noscript),
                    [b't', b'e', b'm', b'p', b'l', b'a', b't', b'e'] => Some(HtmlTag::Template),
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
                    [b's', b'e', b'm', b'a', b'n', b't', b'i', b'c', b's'] => Some(HtmlTag::Semantics),
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
                    [b'b', b'l', b'o', b'c', b'k', b'q', b'u', b'o', b't', b'e'] => Some(HtmlTag::Blockquote),
                    [b'f', b'i', b'g', b'c', b'a', b'p', b't', b'i', b'o', b'n'] => Some(HtmlTag::Figcaption),
                    [b'a', b'n', b'n', b'o', b't', b'a', b't', b'i', b'o', b'n'] => Some(HtmlTag::Annotation),
                    [b'm', b'u', b'n', b'd', b'e', b'r', b'o', b'v', b'e', b'r'] => Some(HtmlTag::Munderover),
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
                    ] => Some(HtmlTag::Mprescripts),
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
                    ] => Some(HtmlTag::AnnotationXML),
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
                    ] => Some(HtmlTag::Mmultiscripts),
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
                    ] => Some(HtmlTag::Selectedcontent),
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
    pub fn is_void_element(&self) -> bool {
        matches!(
            self,
            HtmlTag::Area
                | HtmlTag::Base
                | HtmlTag::Br
                | HtmlTag::Col
                | HtmlTag::Embed
                | HtmlTag::Hr
                | HtmlTag::Img
                | HtmlTag::Input
                | HtmlTag::Link
                | HtmlTag::Meta
                | HtmlTag::Source
                | HtmlTag::Track
                | HtmlTag::Wbr
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
    pub fn should_auto_close(&self, new_tag: &Tag) -> bool {
        if let Tag::Html(new_known) = new_tag {
            return self.should_auto_close_known(new_known);
        }

        false
    }

    /// Determines if a known tag should automatically close based on the current and new known tags.
    fn should_auto_close_known(&self, new: &HtmlTag) -> bool {
        matches!(
            (self, new),
            (HtmlTag::P, HtmlTag::Div)
                | (HtmlTag::P, HtmlTag::P)
                | (HtmlTag::P, HtmlTag::H1)
                | (HtmlTag::P, HtmlTag::H2)
                | (HtmlTag::P, HtmlTag::H3)
                | (HtmlTag::P, HtmlTag::H4)
                | (HtmlTag::P, HtmlTag::H5)
                | (HtmlTag::P, HtmlTag::H6)
                | (HtmlTag::P, HtmlTag::Ul)
                | (HtmlTag::P, HtmlTag::Ol)
                | (HtmlTag::P, HtmlTag::Li)
                | (HtmlTag::P, HtmlTag::Dl)
                | (HtmlTag::P, HtmlTag::Dt)
                | (HtmlTag::P, HtmlTag::Dd)
                | (HtmlTag::P, HtmlTag::Blockquote)
                | (HtmlTag::P, HtmlTag::Pre)
                | (HtmlTag::P, HtmlTag::Form)
                | (HtmlTag::P, HtmlTag::Table)
                | (HtmlTag::P, HtmlTag::Section)
                | (HtmlTag::P, HtmlTag::Article)
                | (HtmlTag::P, HtmlTag::Aside)
                | (HtmlTag::P, HtmlTag::Header)
                | (HtmlTag::P, HtmlTag::Footer)
                | (HtmlTag::P, HtmlTag::Nav)
                | (HtmlTag::P, HtmlTag::Main)
                | (HtmlTag::P, HtmlTag::Figure)
                | (HtmlTag::P, HtmlTag::Hr)
                | (HtmlTag::Li, HtmlTag::Li)
                | (HtmlTag::Dd, HtmlTag::Dd)
                | (HtmlTag::Dt, HtmlTag::Dt)
                | (HtmlTag::Option, HtmlTag::Option)
                | (HtmlTag::Option, HtmlTag::Optgroup)
                | (HtmlTag::Tr, HtmlTag::Tr)
                | (HtmlTag::Td, HtmlTag::Td)
                | (HtmlTag::Td, HtmlTag::Th)
                | (HtmlTag::Td, HtmlTag::Tr)
                | (HtmlTag::Th, HtmlTag::Th)
                | (HtmlTag::Th, HtmlTag::Td)
                | (HtmlTag::Th, HtmlTag::Tr)
                | (HtmlTag::Head, HtmlTag::Body)
        )
    }
}

impl Display for HtmlTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                HtmlTag::Html => "html",
                HtmlTag::Head => "head",
                HtmlTag::Title => "title",
                HtmlTag::Base => "base",
                HtmlTag::Link => "link",
                HtmlTag::Meta => "meta",
                HtmlTag::Style => "style",
                HtmlTag::Body => "body",
                HtmlTag::Article => "article",
                HtmlTag::Section => "section",
                HtmlTag::Nav => "nav",
                HtmlTag::Aside => "aside",
                HtmlTag::H1 => "h1",
                HtmlTag::H2 => "h2",
                HtmlTag::H3 => "h3",
                HtmlTag::H4 => "h4",
                HtmlTag::H5 => "h5",
                HtmlTag::H6 => "h6",
                HtmlTag::HGroup => "hgroup",
                HtmlTag::Header => "header",
                HtmlTag::Footer => "footer",
                HtmlTag::Address => "address",
                HtmlTag::P => "p",
                HtmlTag::Hr => "hr",
                HtmlTag::Pre => "pre",
                HtmlTag::Blockquote => "blockquote",
                HtmlTag::Ol => "ol",
                HtmlTag::Ul => "ul",
                HtmlTag::Menu => "menu",
                HtmlTag::Li => "li",
                HtmlTag::Dl => "dl",
                HtmlTag::Dt => "dt",
                HtmlTag::Dd => "dd",
                HtmlTag::Figure => "figure",
                HtmlTag::Figcaption => "figcaption",
                HtmlTag::Main => "main",
                HtmlTag::Search => "search",
                HtmlTag::Div => "div",
                HtmlTag::A => "a",
                HtmlTag::Em => "em",
                HtmlTag::Strong => "strong",
                HtmlTag::Small => "small",
                HtmlTag::S => "s",
                HtmlTag::Cite => "cite",
                HtmlTag::Q => "q",
                HtmlTag::Dfn => "dfn",
                HtmlTag::Abbr => "abbr",
                HtmlTag::Ruby => "ruby",
                HtmlTag::Rt => "rt",
                HtmlTag::Rp => "rp",
                HtmlTag::Data => "data",
                HtmlTag::Time => "time",
                HtmlTag::Code => "code",
                HtmlTag::Var => "var",
                HtmlTag::Samp => "samp",
                HtmlTag::Kbd => "kbd",
                HtmlTag::Sub => "sub",
                HtmlTag::Sup => "sup",
                HtmlTag::I => "i",
                HtmlTag::B => "b",
                HtmlTag::U => "u",
                HtmlTag::Mark => "mark",
                HtmlTag::Bdi => "bdi",
                HtmlTag::Bdo => "bdo",
                HtmlTag::Span => "span",
                HtmlTag::Br => "br",
                HtmlTag::Wbr => "wbr",
                HtmlTag::Ins => "ins",
                HtmlTag::Del => "del",
                HtmlTag::Picture => "picture",
                HtmlTag::Source => "source",
                HtmlTag::Img => "img",
                HtmlTag::Iframe => "iframe",
                HtmlTag::Embed => "embed",
                HtmlTag::Object => "object",
                HtmlTag::Video => "video",
                HtmlTag::Audio => "audio",
                HtmlTag::Track => "track",
                HtmlTag::Map => "map",
                HtmlTag::Area => "area",
                HtmlTag::Math => "math",
                HtmlTag::Annotation => "annotation",
                HtmlTag::AnnotationXML => "annotation-xml",
                HtmlTag::Mi => "mi",
                HtmlTag::Mo => "mo",
                HtmlTag::Mn => "mn",
                HtmlTag::Ms => "ms",
                HtmlTag::Mfrac => "mfrac",
                HtmlTag::Mmultiscripts => "mmultiscripts",
                HtmlTag::Mover => "mover",
                HtmlTag::Mpadded => "mpadded",
                HtmlTag::Mphantom => "mphantom",
                HtmlTag::Mprescripts => "mprescripts",
                HtmlTag::Mroot => "mroot",
                HtmlTag::Mrow => "mrow",
                HtmlTag::Mspace => "mspace",
                HtmlTag::Msqrt => "msqrt",
                HtmlTag::Mstyle => "mstyle",
                HtmlTag::Msub => "msub",
                HtmlTag::Msubsup => "msubsup",
                HtmlTag::Msup => "msup",
                HtmlTag::Mtable => "mtable",
                HtmlTag::Mtd => "mtd",
                HtmlTag::Mtr => "mtr",
                HtmlTag::Munder => "munder",
                HtmlTag::Munderover => "munderover",
                HtmlTag::Semantics => "semantics",
                HtmlTag::Mtext => "mtext",
                HtmlTag::Merror => "merror",
                HtmlTag::Svg => "svg",
                HtmlTag::Table => "table",
                HtmlTag::Caption => "caption",
                HtmlTag::Colgroup => "colgroup",
                HtmlTag::Col => "col",
                HtmlTag::Tbody => "tbody",
                HtmlTag::Thead => "thead",
                HtmlTag::Tfoot => "tfoot",
                HtmlTag::Tr => "tr",
                HtmlTag::Td => "td",
                HtmlTag::Th => "th",
                HtmlTag::Form => "form",
                HtmlTag::Label => "label",
                HtmlTag::Input => "input",
                HtmlTag::Button => "button",
                HtmlTag::Select => "select",
                HtmlTag::Datalist => "datalist",
                HtmlTag::Optgroup => "optgroup",
                HtmlTag::Option => "option",
                HtmlTag::Textarea => "textarea",
                HtmlTag::Output => "output",
                HtmlTag::Progress => "progress",
                HtmlTag::Meter => "meter",
                HtmlTag::Fieldset => "fieldset",
                HtmlTag::Legend => "legend",
                HtmlTag::Selectedcontent => "selectedcontent",
                HtmlTag::Details => "details",
                HtmlTag::Summary => "summary",
                HtmlTag::Dialog => "dialog",
                HtmlTag::Script => "script",
                HtmlTag::Noscript => "noscript",
                HtmlTag::Template => "template",
                HtmlTag::Slot => "slot",
                HtmlTag::Canvas => "canvas",
            }
        )
    }
}

/// Represents SVG tags as an enum.
///
/// This enum includes common SVG tags that are recognized by the parser.
///
/// <https://developer.mozilla.org/en-US/docs/Web/SVG/Reference/Element>
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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
    pub fn from_str_insensitive(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        match bytes.len() {
            1 => match [bytes[0].to_ascii_lowercase()] {
                [b'a'] => Some(SvgTag::A),
                [b'g'] => Some(SvgTag::G),
                _ => None,
            },
            3 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                ] {
                    [b's', b'e', b't'] => Some(SvgTag::Set),
                    [b's', b'v', b'g'] => Some(SvgTag::Svg),
                    [b'u', b's', b'e'] => Some(SvgTag::Use),
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
                    [b'd', b'e', b'f', b's'] => Some(SvgTag::Defs),
                    [b'd', b'e', b's', b'c'] => Some(SvgTag::Desc),
                    [b'l', b'i', b'n', b'e'] => Some(SvgTag::Line),
                    [b'm', b'a', b's', b'k'] => Some(SvgTag::Mask),
                    [b'p', b'a', b't', b'h'] => Some(SvgTag::Path),
                    [b'r', b'e', b'c', b't'] => Some(SvgTag::Rect),
                    [b's', b't', b'o', b'p'] => Some(SvgTag::Stop),
                    [b't', b'e', b'x', b't'] => Some(SvgTag::Text),
                    [b'v', b'i', b'e', b'w'] => Some(SvgTag::View),
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
                    [b'i', b'm', b'a', b'g', b'e'] => Some(SvgTag::Image),
                    [b'm', b'p', b'a', b't', b'h'] => Some(SvgTag::Mpath),
                    [b't', b'i', b't', b'l', b'e'] => Some(SvgTag::Title),
                    [b't', b's', b'p', b'a', b'n'] => Some(SvgTag::Tspan),
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
                    [b'c', b'i', b'r', b'c', b'l', b'e'] => Some(SvgTag::Circle),
                    [b'f', b'e', b't', b'i', b'l', b'e'] => Some(SvgTag::FeTile),
                    [b'f', b'i', b'l', b't', b'e', b'r'] => Some(SvgTag::Filter),
                    [b'm', b'a', b'r', b'k', b'e', b'r'] => Some(SvgTag::Marker),
                    [b's', b'c', b'r', b'i', b'p', b't'] => Some(SvgTag::Script),
                    [b's', b'w', b'i', b't', b'c', b'h'] => Some(SvgTag::Switch),
                    [b's', b'y', b'm', b'b', b'o', b'l'] => Some(SvgTag::Symbol),
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
                    [b'a', b'n', b'i', b'm', b'a', b't', b'e'] => Some(SvgTag::Animate),
                    [b'e', b'l', b'l', b'i', b'p', b's', b'e'] => Some(SvgTag::Ellipse),
                    [b'f', b'e', b'b', b'l', b'e', b'n', b'd'] => Some(SvgTag::FeBlend),
                    [b'f', b'e', b'f', b'l', b'o', b'o', b'd'] => Some(SvgTag::FeFlood),
                    [b'f', b'e', b'f', b'u', b'n', b'c', b'a'] => Some(SvgTag::FeFuncA),
                    [b'f', b'e', b'f', b'u', b'n', b'c', b'b'] => Some(SvgTag::FeFuncB),
                    [b'f', b'e', b'f', b'u', b'n', b'c', b'g'] => Some(SvgTag::FeFuncG),
                    [b'f', b'e', b'f', b'u', b'n', b'c', b'r'] => Some(SvgTag::FeFuncR),
                    [b'f', b'e', b'i', b'm', b'a', b'g', b'e'] => Some(SvgTag::FeImage),
                    [b'f', b'e', b'm', b'e', b'r', b'g', b'e'] => Some(SvgTag::FeMerge),
                    [b'p', b'a', b't', b't', b'e', b'r', b'n'] => Some(SvgTag::Pattern),
                    [b'p', b'o', b'l', b'y', b'g', b'o', b'n'] => Some(SvgTag::Polygon),
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
                    [b'c', b'l', b'i', b'p', b'p', b'a', b't', b'h'] => Some(SvgTag::ClipPath),
                    [b'f', b'e', b'o', b'f', b'f', b's', b'e', b't'] => Some(SvgTag::FeOffset),
                    [b'm', b'e', b't', b'a', b'd', b'a', b't', b'a'] => Some(SvgTag::Metadata),
                    [b'p', b'o', b'l', b'y', b'l', b'i', b'n', b'e'] => Some(SvgTag::Polyline),
                    [b't', b'e', b'x', b't', b'p', b'a', b't', b'h'] => Some(SvgTag::TextPath),
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
                    ] => Some(SvgTag::FeComposite),
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
                    ] => Some(SvgTag::FeMergeNode),
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
                    ] => Some(SvgTag::FeSpotLight),
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
                    ] => Some(SvgTag::FeDropShadow),
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
                    ] => Some(SvgTag::FeMorphology),
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
                    ] => Some(SvgTag::FePointLight),
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
                    ] => Some(SvgTag::FeTurbulence),
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
                    ] => Some(SvgTag::AnimateMotion),
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
                    ] => Some(SvgTag::FeColorMatrix),
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
                    ] => Some(SvgTag::ForeignObject),
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
                    ] => Some(SvgTag::FeDistantLight),
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
                    ] => Some(SvgTag::FeGaussianBlur),
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
                    ] => Some(SvgTag::LinearGradient),
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
                    ] => Some(SvgTag::RadialGradient),
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
                    ] => Some(SvgTag::AnimateTransform),
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
                    ] => Some(SvgTag::FeConvolveMatrix),
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
                    ] => Some(SvgTag::FeDiffuseLighting),
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
                    ] => Some(SvgTag::FeDisplacementMap),
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
                    ] => Some(SvgTag::FeSpecularLighting),
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
                    ] => Some(SvgTag::FeComponentTransfer),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn is_void_element(&self) -> bool {
        matches!(
            self,
            SvgTag::Circle
                | SvgTag::Ellipse
                | SvgTag::Line
                | SvgTag::Path
                | SvgTag::Polygon
                | SvgTag::Polyline
                | SvgTag::Rect
                | SvgTag::Stop
                | SvgTag::Use
                | SvgTag::Image
                | SvgTag::FeBlend
                | SvgTag::FeColorMatrix
                | SvgTag::FeComponentTransfer
                | SvgTag::FeComposite
                | SvgTag::FeConvolveMatrix
                | SvgTag::FeDiffuseLighting
                | SvgTag::FeDisplacementMap
                | SvgTag::FeDistantLight
                | SvgTag::FeDropShadow
                | SvgTag::FeFlood
                | SvgTag::FeFuncA
                | SvgTag::FeFuncB
                | SvgTag::FeFuncG
                | SvgTag::FeFuncR
                | SvgTag::FeGaussianBlur
                | SvgTag::FeImage
                | SvgTag::FeMergeNode
                | SvgTag::FeMorphology
                | SvgTag::FeOffset
                | SvgTag::FePointLight
                | SvgTag::FeSpecularLighting
                | SvgTag::FeSpotLight
                | SvgTag::FeTile
                | SvgTag::FeTurbulence
                | SvgTag::Animate
                | SvgTag::AnimateMotion
                | SvgTag::AnimateTransform
                | SvgTag::Mpath
                | SvgTag::Set
        )
    }
}

impl Display for SvgTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SvgTag::A => "a",
                SvgTag::Animate => "animate",
                SvgTag::AnimateMotion => "animateMotion",
                SvgTag::AnimateTransform => "animateTransform",
                SvgTag::Circle => "circle",
                SvgTag::ClipPath => "clipPath",
                SvgTag::Defs => "defs",
                SvgTag::Desc => "desc",
                SvgTag::Ellipse => "ellipse",
                SvgTag::FeBlend => "feBlend",
                SvgTag::FeColorMatrix => "feColorMatrix",
                SvgTag::FeComponentTransfer => "feComponentTransfer",
                SvgTag::FeComposite => "feComposite",
                SvgTag::FeConvolveMatrix => "feConvolveMatrix",
                SvgTag::FeDiffuseLighting => "feDiffuseLighting",
                SvgTag::FeDisplacementMap => "feDisplacementMap",
                SvgTag::FeDistantLight => "feDistantLight",
                SvgTag::FeDropShadow => "feDropShadow",
                SvgTag::FeFlood => "feFlood",
                SvgTag::FeFuncA => "feFuncA",
                SvgTag::FeFuncB => "feFuncB",
                SvgTag::FeFuncG => "feFuncG",
                SvgTag::FeFuncR => "feFuncR",
                SvgTag::FeGaussianBlur => "feGaussianBlur",
                SvgTag::FeImage => "feImage",
                SvgTag::FeMerge => "feMerge",
                SvgTag::FeMergeNode => "feMergeNode",
                SvgTag::FeMorphology => "feMorphology",
                SvgTag::FeOffset => "feOffset",
                SvgTag::FePointLight => "fePointLight",
                SvgTag::FeSpecularLighting => "feSpecularLighting",
                SvgTag::FeSpotLight => "feSpotLight",
                SvgTag::FeTile => "feTile",
                SvgTag::FeTurbulence => "feTurbulence",
                SvgTag::Filter => "filter",
                SvgTag::ForeignObject => "foreignObject",
                SvgTag::G => "g",
                SvgTag::Image => "image",
                SvgTag::Line => "line",
                SvgTag::LinearGradient => "linearGradient",
                SvgTag::Marker => "marker",
                SvgTag::Mask => "mask",
                SvgTag::Metadata => "metadata",
                SvgTag::Mpath => "mpath",
                SvgTag::Path => "path",
                SvgTag::Pattern => "pattern",
                SvgTag::Polygon => "polygon",
                SvgTag::Polyline => "polyline",
                SvgTag::RadialGradient => "radialGradient",
                SvgTag::Rect => "rect",
                SvgTag::Script => "script",
                SvgTag::Set => "set",
                SvgTag::Stop => "stop",
                SvgTag::Svg => "svg",
                SvgTag::Switch => "switch",
                SvgTag::Symbol => "symbol",
                SvgTag::Text => "text",
                SvgTag::TextPath => "textPath",
                SvgTag::Title => "title",
                SvgTag::Tspan => "tspan",
                SvgTag::Use => "use",
                SvgTag::View => "view",
            }
        )
    }
}
