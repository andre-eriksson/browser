#[derive(Debug, Clone, Default)]
pub struct ContentSecurityPolicy {
    // Fetch directives
    pub child_src: Option<Vec<String>>,
    pub connect_src: Option<Vec<String>>,
    pub default_src: Option<Vec<String>>,
    pub font_src: Option<Vec<String>>,
    pub frame_src: Option<Vec<String>>,
    pub img_src: Option<Vec<String>>,
    pub manifest_src: Option<Vec<String>>,
    pub media_src: Option<Vec<String>>,
    pub object_src: Option<Vec<String>>,
    pub script_src: Option<Vec<String>>,
    pub script_src_elem: Option<Vec<String>>,
    pub script_src_attr: Option<Vec<String>>,
    pub style_src: Option<Vec<String>>,
    pub style_src_elem: Option<Vec<String>>,
    pub style_src_attr: Option<Vec<String>>,
    pub worker_src: Option<Vec<String>>,

    // Document directives
    pub base_uri: Option<Vec<String>>,
    pub sandbox: Option<Vec<String>>,

    // Navigation directives
    pub form_action: Option<Vec<String>>,
    pub frame_ancestors: Option<Vec<String>>,

    // Reporting directives
    pub report_uri: Option<Vec<String>>,

    // Other directives
    pub require_trusted_types_for: Option<Vec<String>>,
    pub trusted_types: Option<Vec<String>>,
    pub upgrade_insecure_requests: bool,
}

impl ContentSecurityPolicy {
    pub fn new() -> Self {
        ContentSecurityPolicy {
            child_src: None,
            connect_src: None,
            default_src: None,
            font_src: None,
            frame_src: None,
            img_src: None,
            manifest_src: None,
            media_src: None,
            object_src: None,
            script_src: None,
            script_src_elem: None,
            script_src_attr: None,
            style_src: None,
            style_src_elem: None,
            style_src_attr: None,
            worker_src: None,

            base_uri: None,
            sandbox: None,

            form_action: None,
            frame_ancestors: None,

            report_uri: None,

            require_trusted_types_for: None,
            trusted_types: None,
            upgrade_insecure_requests: false,
        }
    }

    pub fn from_header_string(header: &str) -> Self {
        let mut csp = ContentSecurityPolicy::new();

        for directive in header.split(';') {
            let parts: Vec<&str> = directive.trim().split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let directive_name = parts[0].to_lowercase();
            let values = parts[1..]
                .iter()
                .map(|&s| s.to_string())
                .collect::<Vec<String>>();

            match directive_name.as_str() {
                "child-src" => csp.child_src = Some(values),
                "connect-src" => csp.connect_src = Some(values),
                "default-src" => csp.default_src = Some(values),
                "font-src" => csp.font_src = Some(values),
                "frame-src" => csp.frame_src = Some(values),
                "img-src" => csp.img_src = Some(values),
                "manifest-src" => csp.manifest_src = Some(values),
                "media-src" => csp.media_src = Some(values),
                "object-src" => csp.object_src = Some(values),
                "script-src" => csp.script_src = Some(values),
                "script-src-elem" => csp.script_src_elem = Some(values),
                "script-src-attr" => csp.script_src_attr = Some(values),
                "style-src" => csp.style_src = Some(values),
                "style-src-elem" => csp.style_src_elem = Some(values),
                "style-src-attr" => csp.style_src_attr = Some(values),
                "worker-src" => csp.worker_src = Some(values),

                "base-uri" => csp.base_uri = Some(values),
                "sandbox" => csp.sandbox = Some(values),

                "form-action" => csp.form_action = Some(values),
                "frame-ancestors" => csp.frame_ancestors = Some(values),

                "report-uri" | "report-to" => csp.report_uri = Some(values),

                "require-trusted-types-for" => csp.require_trusted_types_for = Some(values),
                "trusted-types" => csp.trusted_types = Some(values),

                "upgrade-insecure-requests" => {
                    if !values.is_empty() && values[0] == "" {
                        csp.upgrade_insecure_requests = true;
                    }
                }
                _ => {
                    eprintln!("Unknown CSP directive: {}", directive_name);
                    continue;
                }
            }
        }

        csp
    }

    pub fn is_blocked(&self, source_url: &str, element: &str, request_url: &str) -> bool {
        let sources = match element {
            "child" => self.child_src.as_ref(),
            "connect" => self.connect_src.as_ref(),
            "font" => self.font_src.as_ref(),
            "frame" => self.frame_src.as_ref().or(self.child_src.as_ref()),
            "img" => self.img_src.as_ref(),
            "manifest" => self.manifest_src.as_ref(),
            "media" => self.media_src.as_ref(),
            "object" => self.object_src.as_ref(),
            "script" => self.script_src.as_ref(),
            "script-elem" => self.script_src_elem.as_ref().or(self.script_src.as_ref()),
            "script-attr" => self.script_src_attr.as_ref().or(self.script_src.as_ref()),
            "style" => self.style_src.as_ref(),
            "style-elem" => self.style_src_elem.as_ref().or(self.style_src.as_ref()),
            "style-attr" => self.style_src_attr.as_ref().or(self.style_src.as_ref()),
            "worker" => self.worker_src.as_ref().or(self.child_src.as_ref()),
            _ => return false, // Unknown element type
        };

        // Fall back to default-src if no specific directive is set
        let sources = sources.or(self.default_src.as_ref());

        if let Some(sources) = sources {
            !sources.iter().any(|source| match source.as_str() {
                "*" => true,
                "'self'" => self.is_same_origin(source_url, request_url),
                _ => request_url.starts_with(source) || source == request_url,
            })
        } else {
            true
        }
    }

    fn is_same_origin(&self, source_url: &str, request_url: &str) -> bool {
        if let (Ok(source_parsed), Ok(request_parsed)) =
            (url::Url::parse(source_url), url::Url::parse(request_url))
        {
            source_parsed.scheme() == request_parsed.scheme()
                && source_parsed.host() == request_parsed.host()
                && source_parsed.port() == request_parsed.port()
        } else {
            false
        }
    }
}
