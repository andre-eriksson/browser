use std::fmt::Display;

/// Represents an HTML tag, which can be either a known tag or an unknown tag.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HtmlTag {
    /// Represents a known HTML tag defined in the `KnownTag` enum.
    Known(KnownTag),
    /// Represents an unknown HTML tag, where the string is the tag name, for instance custom tags like `<yt-thumbnail-view-model>`.
    Unknown(String),
}

/// Represents known HTML tags as an enum.
///
/// This enum includes common HTML tags that are recognized by the parser.
///
/// <https://html.spec.whatwg.org/multipage/#toc-semantics>
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum KnownTag {
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

/// A perfect hash map for HTML tags
static TAGS: phf::Map<&'static str, HtmlTag> = phf::phf_map! {
    "html" => HtmlTag::Known(KnownTag::Html),
    "head" => HtmlTag::Known(KnownTag::Head),
    "title" => HtmlTag::Known(KnownTag::Title),
    "base" => HtmlTag::Known(KnownTag::Base),
    "link" => HtmlTag::Known(KnownTag::Link),
    "meta" => HtmlTag::Known(KnownTag::Meta),
    "style" => HtmlTag::Known(KnownTag::Style),
    "body" => HtmlTag::Known(KnownTag::Body),
    "article" => HtmlTag::Known(KnownTag::Article),
    "section" => HtmlTag::Known(KnownTag::Section),
    "nav" => HtmlTag::Known(KnownTag::Nav),
    "aside" => HtmlTag::Known(KnownTag::Aside),
    "h1" => HtmlTag::Known(KnownTag::H1),
    "h2" => HtmlTag::Known(KnownTag::H2),
    "h3" => HtmlTag::Known(KnownTag::H3),
    "h4" => HtmlTag::Known(KnownTag::H4),
    "h5" => HtmlTag::Known(KnownTag::H5),
    "h6" => HtmlTag::Known(KnownTag::H6),
    "hgroup" => HtmlTag::Known(KnownTag::HGroup),
    "header" => HtmlTag::Known(KnownTag::Header),
    "footer" => HtmlTag::Known(KnownTag::Footer),
    "address" => HtmlTag::Known(KnownTag::Address),
    "p" => HtmlTag::Known(KnownTag::P),
    "hr" => HtmlTag::Known(KnownTag::Hr),
    "pre" => HtmlTag::Known(KnownTag::Pre),
    "blockquote" => HtmlTag::Known(KnownTag::Blockquote),
    "ol" => HtmlTag::Known(KnownTag::Ol),
    "ul" => HtmlTag::Known(KnownTag::Ul),
    "menu" => HtmlTag::Known(KnownTag::Menu),
    "li" => HtmlTag::Known(KnownTag::Li),
    "dl" => HtmlTag::Known(KnownTag::Dl),
    "dt" => HtmlTag::Known(KnownTag::Dt),
    "dd" => HtmlTag::Known(KnownTag::Dd),
    "figure" => HtmlTag::Known(KnownTag::Figure),
    "figcaption" => HtmlTag::Known(KnownTag::Figcaption),
    "main" => HtmlTag::Known(KnownTag::Main),
    "search" => HtmlTag::Known(KnownTag::Search),
    "div" => HtmlTag::Known(KnownTag::Div),
    "a" => HtmlTag::Known(KnownTag::A),
    "em" => HtmlTag::Known(KnownTag::Em),
    "strong" => HtmlTag::Known(KnownTag::Strong),
    "small" => HtmlTag::Known(KnownTag::Small),
    "s" => HtmlTag::Known(KnownTag::S),
    "cite" => HtmlTag::Known(KnownTag::Cite),
    "q" => HtmlTag::Known(KnownTag::Q),
    "dfn" => HtmlTag::Known(KnownTag::Dfn),
    "abbr" => HtmlTag::Known(KnownTag::Abbr),
    "ruby" => HtmlTag::Known(KnownTag::Ruby),
    "rt" => HtmlTag::Known(KnownTag::Rt),
    "rp" => HtmlTag::Known(KnownTag::Rp),
    "data" => HtmlTag::Known(KnownTag::Data),
    "time" => HtmlTag::Known(KnownTag::Time),
    "code" => HtmlTag::Known(KnownTag::Code),
    "var" => HtmlTag::Known(KnownTag::Var),
    "samp" => HtmlTag::Known(KnownTag::Samp),
    "kbd" => HtmlTag::Known(KnownTag::Kbd),
    "sub" => HtmlTag::Known(KnownTag::Sub),
    "sup" => HtmlTag::Known(KnownTag::Sup),
    "i" => HtmlTag::Known(KnownTag::I),
    "b" => HtmlTag::Known(KnownTag::B),
    "u" => HtmlTag::Known(KnownTag::U),
    "mark" => HtmlTag::Known(KnownTag::Mark),
    "bdi" => HtmlTag::Known(KnownTag::Bdi),
    "bdo" => HtmlTag::Known(KnownTag::Bdo),
    "span" => HtmlTag::Known(KnownTag::Span),
    "br" => HtmlTag::Known(KnownTag::Br),
    "wbr" => HtmlTag::Known(KnownTag::Wbr),
    "ins" => HtmlTag::Known(KnownTag::Ins),
    "del" => HtmlTag::Known(KnownTag::Del),
    "picture" => HtmlTag::Known(KnownTag::Picture),
    "source" => HtmlTag::Known(KnownTag::Source),
    "img" => HtmlTag::Known(KnownTag::Img),
    "iframe" => HtmlTag::Known(KnownTag::Iframe),
    "embed" => HtmlTag::Known(KnownTag::Embed),
    "object" => HtmlTag::Known(KnownTag::Object),
    "video" => HtmlTag::Known(KnownTag::Video),
    "audio" => HtmlTag::Known(KnownTag::Audio),
    "track" => HtmlTag::Known(KnownTag::Track),
    "map" => HtmlTag::Known(KnownTag::Map),
    "area" => HtmlTag::Known(KnownTag::Area),
    "math" => HtmlTag::Known(KnownTag::Math),
    "annotation" => HtmlTag::Known(KnownTag::Annotation),
    "annotation-xml" => HtmlTag::Known(KnownTag::AnnotationXML),
    "mi" => HtmlTag::Known(KnownTag::Mi),
    "mo" => HtmlTag::Known(KnownTag::Mo),
    "mn" => HtmlTag::Known(KnownTag::Mn),
    "ms" => HtmlTag::Known(KnownTag::Ms),
    "mfrac" => HtmlTag::Known(KnownTag::Mfrac),
    "mmultiscripts" => HtmlTag::Known(KnownTag::Mmultiscripts),
    "mover" => HtmlTag::Known(KnownTag::Mover),
    "mpadded" => HtmlTag::Known(KnownTag::Mpadded),
    "mphantom" => HtmlTag::Known(KnownTag::Mphantom),
    "mprescripts" => HtmlTag::Known(KnownTag::Mprescripts),
    "mroot" => HtmlTag::Known(KnownTag::Mroot),
    "mrow" => HtmlTag::Known(KnownTag::Mrow),
    "mspace" => HtmlTag::Known(KnownTag::Mspace),
    "msqrt" => HtmlTag::Known(KnownTag::Msqrt),
    "mstyle" => HtmlTag::Known(KnownTag::Mstyle),
    "msub" => HtmlTag::Known(KnownTag::Msub),
    "msubsup" => HtmlTag::Known(KnownTag::Msubsup),
    "msup" => HtmlTag::Known(KnownTag::Msup),
    "mtable" => HtmlTag::Known(KnownTag::Mtable),
    "mtd" => HtmlTag::Known(KnownTag::Mtd),
    "mtr" => HtmlTag::Known(KnownTag::Mtr),
    "munder" => HtmlTag::Known(KnownTag::Munder),
    "munderover" => HtmlTag::Known(KnownTag::Munderover),
    "semantics" => HtmlTag::Known(KnownTag::Semantics),
    "mtext" => HtmlTag::Known(KnownTag::Mtext),
    "merror" => HtmlTag::Known(KnownTag::Merror),
    "svg" => HtmlTag::Known(KnownTag::Svg),
    "table" => HtmlTag::Known(KnownTag::Table),
    "caption" => HtmlTag::Known(KnownTag::Caption),
    "colgroup" => HtmlTag::Known(KnownTag::Colgroup),
    "col" => HtmlTag::Known(KnownTag::Col),
    "tbody" => HtmlTag::Known(KnownTag::Tbody),
    "thead" => HtmlTag::Known(KnownTag::Thead),
    "tfoot" => HtmlTag::Known(KnownTag::Tfoot),
    "tr" => HtmlTag::Known(KnownTag::Tr),
    "td" => HtmlTag::Known(KnownTag::Td),
    "th" => HtmlTag::Known(KnownTag::Th),
    "form" => HtmlTag::Known(KnownTag::Form),
    "label" => HtmlTag::Known(KnownTag::Label),
    "input" => HtmlTag::Known(KnownTag::Input),
    "button" => HtmlTag::Known(KnownTag::Button),
    "select" => HtmlTag::Known(KnownTag::Select),
    "datalist" => HtmlTag::Known(KnownTag::Datalist),
    "optgroup" => HtmlTag::Known(KnownTag::Optgroup),
    "option" => HtmlTag::Known(KnownTag::Option),
    "textarea" => HtmlTag::Known(KnownTag::Textarea),
    "output" => HtmlTag::Known(KnownTag::Output),
    "progress" => HtmlTag::Known(KnownTag::Progress),
    "meter" => HtmlTag::Known(KnownTag::Meter),
    "fieldset" => HtmlTag::Known(KnownTag::Fieldset),
    "legend" => HtmlTag::Known(KnownTag::Legend),
    "selectedcontent" => HtmlTag::Known(KnownTag::Selectedcontent),
    "details" => HtmlTag::Known(KnownTag::Details),
    "summary" => HtmlTag::Known(KnownTag::Summary),
    "dialog" => HtmlTag::Known(KnownTag::Dialog),
    "script" => HtmlTag::Known(KnownTag::Script),
    "noscript" => HtmlTag::Known(KnownTag::Noscript),
    "template" => HtmlTag::Known(KnownTag::Template),
    "slot" => HtmlTag::Known(KnownTag::Slot),
    "canvas" => HtmlTag::Known(KnownTag::Canvas),
};

impl Display for HtmlTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HtmlTag::Known(tag) => write!(f, "{}", format!("{:?}", tag).to_lowercase()),
            HtmlTag::Unknown(name) => write!(f, "{}", name),
        }
    }
}

/// Converts a string representation of an HTML tag into an `HtmlTag` enum.
///
/// # Arguments
/// * `s` - The string representation of the HTML tag.
///
/// # Returns
/// An `HtmlTag` enum representing the HTML tag.
pub fn tag_from_str(s: &str) -> HtmlTag {
    TAGS.get(s)
        .cloned()
        .unwrap_or_else(|| HtmlTag::Unknown(s.to_string()))
}

/// Checks if the given tag is a void element.
/// Void elements are those that do not have any content and do not require a closing tag.
///
/// # Arguments
/// * `tag` - The HTML tag to check.
///
/// # Returns
/// `true` if the tag is a void element, `false` otherwise.
pub fn is_void_element(tag: &HtmlTag) -> bool {
    if let HtmlTag::Known(tag_name) = tag {
        return is_void_element_known(tag_name);
    }

    false
}

fn is_void_element_known(tag: &KnownTag) -> bool {
    matches!(
        tag,
        KnownTag::Area
            | KnownTag::Base
            | KnownTag::Br
            | KnownTag::Col
            | KnownTag::Embed
            | KnownTag::Hr
            | KnownTag::Img
            | KnownTag::Input
            | KnownTag::Link
            | KnownTag::Meta
            | KnownTag::Source
            | KnownTag::Track
            | KnownTag::Wbr
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
pub fn should_auto_close(current_tag: &HtmlTag, new_tag: &HtmlTag) -> bool {
    if let HtmlTag::Known(current_known) = current_tag
        && let HtmlTag::Known(new_known) = new_tag
    {
        return should_auto_close_known(current_known, new_known);
    }

    false
}

fn should_auto_close_known(current: &KnownTag, new: &KnownTag) -> bool {
    matches!(
        (current, new),
        (KnownTag::P, KnownTag::Div)
            | (KnownTag::P, KnownTag::P)
            | (KnownTag::P, KnownTag::H1)
            | (KnownTag::P, KnownTag::H2)
            | (KnownTag::P, KnownTag::H3)
            | (KnownTag::P, KnownTag::H4)
            | (KnownTag::P, KnownTag::H5)
            | (KnownTag::P, KnownTag::H6)
            | (KnownTag::P, KnownTag::Ul)
            | (KnownTag::P, KnownTag::Ol)
            | (KnownTag::P, KnownTag::Li)
            | (KnownTag::P, KnownTag::Dl)
            | (KnownTag::P, KnownTag::Dt)
            | (KnownTag::P, KnownTag::Dd)
            | (KnownTag::P, KnownTag::Blockquote)
            | (KnownTag::P, KnownTag::Pre)
            | (KnownTag::P, KnownTag::Form)
            | (KnownTag::P, KnownTag::Table)
            | (KnownTag::P, KnownTag::Section)
            | (KnownTag::P, KnownTag::Article)
            | (KnownTag::P, KnownTag::Aside)
            | (KnownTag::P, KnownTag::Header)
            | (KnownTag::P, KnownTag::Footer)
            | (KnownTag::P, KnownTag::Nav)
            | (KnownTag::P, KnownTag::Main)
            | (KnownTag::P, KnownTag::Figure)
            | (KnownTag::P, KnownTag::Hr)
            | (KnownTag::Li, KnownTag::Li)
            | (KnownTag::Dd, KnownTag::Dd)
            | (KnownTag::Dt, KnownTag::Dt)
            | (KnownTag::Option, KnownTag::Option)
            | (KnownTag::Option, KnownTag::Optgroup)
            | (KnownTag::Tr, KnownTag::Tr)
            | (KnownTag::Td, KnownTag::Td)
            | (KnownTag::Td, KnownTag::Th)
            | (KnownTag::Td, KnownTag::Tr)
            | (KnownTag::Th, KnownTag::Th)
            | (KnownTag::Th, KnownTag::Td)
            | (KnownTag::Th, KnownTag::Tr)
    )
}
