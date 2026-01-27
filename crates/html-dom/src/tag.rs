use std::fmt::Display;

// === Master Tag ===

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

impl From<&str> for Tag {
    fn from(value: &str) -> Self {
        if let Ok(html_tag) = HtmlTag::try_from(value) {
            Tag::Html(html_tag)
        } else if let Ok(svg_tag) = SvgTag::try_from(value) {
            Tag::Svg(svg_tag)
        } else {
            Tag::Unknown(value.to_string())
        }
    }
}

/// Represents known HTML tags as an enum.
///
/// This enum includes common HTML tags that are recognized by the parser.
///
/// <https://html.spec.whatwg.org/multipage/#toc-semantics>
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
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

impl TryFrom<&str> for HtmlTag {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "html" => Ok(HtmlTag::Html),
            "head" => Ok(HtmlTag::Head),
            "title" => Ok(HtmlTag::Title),
            "base" => Ok(HtmlTag::Base),
            "link" => Ok(HtmlTag::Link),
            "meta" => Ok(HtmlTag::Meta),
            "style" => Ok(HtmlTag::Style),
            "body" => Ok(HtmlTag::Body),
            "article" => Ok(HtmlTag::Article),
            "section" => Ok(HtmlTag::Section),
            "nav" => Ok(HtmlTag::Nav),
            "aside" => Ok(HtmlTag::Aside),
            "h1" => Ok(HtmlTag::H1),
            "h2" => Ok(HtmlTag::H2),
            "h3" => Ok(HtmlTag::H3),
            "h4" => Ok(HtmlTag::H4),
            "h5" => Ok(HtmlTag::H5),
            "h6" => Ok(HtmlTag::H6),
            "hgroup" => Ok(HtmlTag::HGroup),
            "header" => Ok(HtmlTag::Header),
            "footer" => Ok(HtmlTag::Footer),
            "address" => Ok(HtmlTag::Address),
            "p" => Ok(HtmlTag::P),
            "hr" => Ok(HtmlTag::Hr),
            "pre" => Ok(HtmlTag::Pre),
            "blockquote" => Ok(HtmlTag::Blockquote),
            "ol" => Ok(HtmlTag::Ol),
            "ul" => Ok(HtmlTag::Ul),
            "menu" => Ok(HtmlTag::Menu),
            "li" => Ok(HtmlTag::Li),
            "dl" => Ok(HtmlTag::Dl),
            "dt" => Ok(HtmlTag::Dt),
            "dd" => Ok(HtmlTag::Dd),
            "figure" => Ok(HtmlTag::Figure),
            "figcaption" => Ok(HtmlTag::Figcaption),
            "main" => Ok(HtmlTag::Main),
            "search" => Ok(HtmlTag::Search),
            "div" => Ok(HtmlTag::Div),
            "a" => Ok(HtmlTag::A),
            "em" => Ok(HtmlTag::Em),
            "strong" => Ok(HtmlTag::Strong),
            "small" => Ok(HtmlTag::Small),
            "s" => Ok(HtmlTag::S),
            "cite" => Ok(HtmlTag::Cite),
            "q" => Ok(HtmlTag::Q),
            "dfn" => Ok(HtmlTag::Dfn),
            "abbr" => Ok(HtmlTag::Abbr),
            "ruby" => Ok(HtmlTag::Ruby),
            "rt" => Ok(HtmlTag::Rt),
            "rp" => Ok(HtmlTag::Rp),
            "data" => Ok(HtmlTag::Data),
            "time" => Ok(HtmlTag::Time),
            "code" => Ok(HtmlTag::Code),
            "var" => Ok(HtmlTag::Var),
            "samp" => Ok(HtmlTag::Samp),
            "kbd" => Ok(HtmlTag::Kbd),
            "sub" => Ok(HtmlTag::Sub),
            "sup" => Ok(HtmlTag::Sup),
            "i" => Ok(HtmlTag::I),
            "b" => Ok(HtmlTag::B),
            "u" => Ok(HtmlTag::U),
            "mark" => Ok(HtmlTag::Mark),
            "bdi" => Ok(HtmlTag::Bdi),
            "bdo" => Ok(HtmlTag::Bdo),
            "span" => Ok(HtmlTag::Span),
            "br" => Ok(HtmlTag::Br),
            "wbr" => Ok(HtmlTag::Wbr),
            "ins" => Ok(HtmlTag::Ins),
            "del" => Ok(HtmlTag::Del),
            "picture" => Ok(HtmlTag::Picture),
            "source" => Ok(HtmlTag::Source),
            "img" => Ok(HtmlTag::Img),
            "iframe" => Ok(HtmlTag::Iframe),
            "embed" => Ok(HtmlTag::Embed),
            "object" => Ok(HtmlTag::Object),
            "video" => Ok(HtmlTag::Video),
            "audio" => Ok(HtmlTag::Audio),
            "track" => Ok(HtmlTag::Track),
            "map" => Ok(HtmlTag::Map),
            "area" => Ok(HtmlTag::Area),
            "math" => Ok(HtmlTag::Math),
            "annotation" => Ok(HtmlTag::Annotation),
            "annotation-xml" => Ok(HtmlTag::AnnotationXML),
            "mi" => Ok(HtmlTag::Mi),
            "mo" => Ok(HtmlTag::Mo),
            "mn" => Ok(HtmlTag::Mn),
            "ms" => Ok(HtmlTag::Ms),
            "mfrac" => Ok(HtmlTag::Mfrac),
            "mmultiscripts" => Ok(HtmlTag::Mmultiscripts),
            "mover" => Ok(HtmlTag::Mover),
            "mpadded" => Ok(HtmlTag::Mpadded),
            "mphantom" => Ok(HtmlTag::Mphantom),
            "mprescripts" => Ok(HtmlTag::Mprescripts),
            "mroot" => Ok(HtmlTag::Mroot),
            "mrow" => Ok(HtmlTag::Mrow),
            "mspace" => Ok(HtmlTag::Mspace),
            "msqrt" => Ok(HtmlTag::Msqrt),
            "mstyle" => Ok(HtmlTag::Mstyle),
            "msub" => Ok(HtmlTag::Msub),
            "msubsup" => Ok(HtmlTag::Msubsup),
            "msup" => Ok(HtmlTag::Msup),
            "mtable" => Ok(HtmlTag::Mtable),
            "mtd" => Ok(HtmlTag::Mtd),
            "mtr" => Ok(HtmlTag::Mtr),
            "munder" => Ok(HtmlTag::Munder),
            "munderover" => Ok(HtmlTag::Munderover),
            "semantics" => Ok(HtmlTag::Semantics),
            "mtext" => Ok(HtmlTag::Mtext),
            "merror" => Ok(HtmlTag::Merror),
            "svg" => Ok(HtmlTag::Svg),
            "table" => Ok(HtmlTag::Table),
            "caption" => Ok(HtmlTag::Caption),
            "colgroup" => Ok(HtmlTag::Colgroup),
            "col" => Ok(HtmlTag::Col),
            "tbody" => Ok(HtmlTag::Tbody),
            "thead" => Ok(HtmlTag::Thead),
            "tfoot" => Ok(HtmlTag::Tfoot),
            "tr" => Ok(HtmlTag::Tr),
            "td" => Ok(HtmlTag::Td),
            "th" => Ok(HtmlTag::Th),
            "form" => Ok(HtmlTag::Form),
            "label" => Ok(HtmlTag::Label),
            "input" => Ok(HtmlTag::Input),
            "button" => Ok(HtmlTag::Button),
            "select" => Ok(HtmlTag::Select),
            "datalist" => Ok(HtmlTag::Datalist),
            "optgroup" => Ok(HtmlTag::Optgroup),
            "option" => Ok(HtmlTag::Option),
            "textarea" => Ok(HtmlTag::Textarea),
            "output" => Ok(HtmlTag::Output),
            "progress" => Ok(HtmlTag::Progress),
            "meter" => Ok(HtmlTag::Meter),
            "fieldset" => Ok(HtmlTag::Fieldset),
            "legend" => Ok(HtmlTag::Legend),
            "selectedcontent" => Ok(HtmlTag::Selectedcontent),
            "details" => Ok(HtmlTag::Details),
            "summary" => Ok(HtmlTag::Summary),
            "dialog" => Ok(HtmlTag::Dialog),
            "script" => Ok(HtmlTag::Script),
            "noscript" => Ok(HtmlTag::Noscript),
            "template" => Ok(HtmlTag::Template),
            "slot" => Ok(HtmlTag::Slot),
            "canvas" => Ok(HtmlTag::Canvas),
            s => Err(format!("'{}' is not a recognized HTML tag", s)),
        }
    }
}

impl TryFrom<String> for HtmlTag {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        HtmlTag::try_from(value.as_str())
    }
}

/// Represents SVG tags as an enum.
///
/// This enum includes common SVG tags that are recognized by the parser.
///
/// <https://developer.mozilla.org/en-US/docs/Web/SVG/Reference/Element>
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
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

impl TryFrom<&str> for SvgTag {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "a" => Ok(SvgTag::A),
            "animate" => Ok(SvgTag::Animate),
            "animateMotion" => Ok(SvgTag::AnimateMotion),
            "animateTransform" => Ok(SvgTag::AnimateTransform),
            "circle" => Ok(SvgTag::Circle),
            "clipPath" => Ok(SvgTag::ClipPath),
            "defs" => Ok(SvgTag::Defs),
            "desc" => Ok(SvgTag::Desc),
            "ellipse" => Ok(SvgTag::Ellipse),
            "feBlend" => Ok(SvgTag::FeBlend),
            "feColorMatrix" => Ok(SvgTag::FeColorMatrix),
            "feComponentTransfer" => Ok(SvgTag::FeComponentTransfer),
            "feComposite" => Ok(SvgTag::FeComposite),
            "feConvolveMatrix" => Ok(SvgTag::FeConvolveMatrix),
            "feDiffuseLighting" => Ok(SvgTag::FeDiffuseLighting),
            "feDisplacementMap" => Ok(SvgTag::FeDisplacementMap),
            "feDistantLight" => Ok(SvgTag::FeDistantLight),
            "feDropShadow" => Ok(SvgTag::FeDropShadow),
            "feFlood" => Ok(SvgTag::FeFlood),
            "feFuncA" => Ok(SvgTag::FeFuncA),
            "feFuncB" => Ok(SvgTag::FeFuncB),
            "feFuncG" => Ok(SvgTag::FeFuncG),
            "feFuncR" => Ok(SvgTag::FeFuncR),
            "feGaussianBlur" => Ok(SvgTag::FeGaussianBlur),
            "feImage" => Ok(SvgTag::FeImage),
            "feMerge" => Ok(SvgTag::FeMerge),
            "feMergeNode" => Ok(SvgTag::FeMergeNode),
            "feMorphology" => Ok(SvgTag::FeMorphology),
            "feOffset" => Ok(SvgTag::FeOffset),
            "fePointLight" => Ok(SvgTag::FePointLight),
            "feSpecularLighting" => Ok(SvgTag::FeSpecularLighting),
            "feSpotLight" => Ok(SvgTag::FeSpotLight),
            "feTile" => Ok(SvgTag::FeTile),
            "feTurbulence" => Ok(SvgTag::FeTurbulence),
            "filter" => Ok(SvgTag::Filter),
            "foreignObject" => Ok(SvgTag::ForeignObject),
            "g" => Ok(SvgTag::G),
            "image" => Ok(SvgTag::Image),
            "line" => Ok(SvgTag::Line),
            "linearGradient" => Ok(SvgTag::LinearGradient),
            "marker" => Ok(SvgTag::Marker),
            "mask" => Ok(SvgTag::Mask),
            "metadata" => Ok(SvgTag::Metadata),
            "mpath" => Ok(SvgTag::Mpath),
            "path" => Ok(SvgTag::Path),
            "pattern" => Ok(SvgTag::Pattern),
            "polygon" => Ok(SvgTag::Polygon),
            "polyline" => Ok(SvgTag::Polyline),
            "radialGradient" => Ok(SvgTag::RadialGradient),
            "rect" => Ok(SvgTag::Rect),
            "script" => Ok(SvgTag::Script),
            "set" => Ok(SvgTag::Set),
            "stop" => Ok(SvgTag::Stop),
            "svg" => Ok(SvgTag::Svg),
            "switch" => Ok(SvgTag::Switch),
            "symbol" => Ok(SvgTag::Symbol),
            "text" => Ok(SvgTag::Text),
            "textPath" => Ok(SvgTag::TextPath),
            "title" => Ok(SvgTag::Title),
            "tspan" => Ok(SvgTag::Tspan),
            "use" => Ok(SvgTag::Use),
            "view" => Ok(SvgTag::View),
            s => Err(format!("'{}' is not a recognized SVG tag", s)),
        }
    }
}

impl TryFrom<String> for SvgTag {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        SvgTag::try_from(value.as_str())
    }
}
