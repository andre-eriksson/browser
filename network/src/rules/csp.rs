use std::collections::HashMap;

use http::{HeaderMap, HeaderValue, header::CONTENT_SECURITY_POLICY};
use url::Origin;

use crate::util::source::{SourceType, get_source_from_tag};

#[derive(Default)]
struct CSPFallback {
    directives: HashMap<String, Vec<String>>,
}

fn test_csp(csp: &str, source_type: SourceType, request_origin: &Origin) -> Result<(), String> {
    // Parse the CSP header
    let mut csp_fallback = CSPFallback::default();
    let directives: Vec<&str> = csp.split(';').map(|s| s.trim()).collect();

    fn fallback_to_default_src(
        csp_fallback: &mut CSPFallback,
        request_origin: &Origin,
    ) -> Result<(), String> {
        if let Some(default_sources) = csp_fallback.directives.get("default-src") {
            for source in default_sources {
                if source == &request_origin.unicode_serialization() || source == "*" {
                    return Ok(()); // Allowed
                }
            }
        }
        Err("CSP blocks script execution".to_string())
    }

    // Check if the CSP allows the request origin
    for directive in directives {
        let parts: Vec<&str> = directive.splitn(2, ' ').collect();
        if parts.len() < 2 {
            continue; // Skip invalid directives
        }

        let directive_name = parts[0].to_lowercase();
        let allowed_sources = parts[1].split_whitespace();

        match directive_name.as_str() {
            "connect-src" => {
                if source_type != SourceType::Fetch {
                    continue; // Skip if not a connect request
                }

                if allowed_sources.clone().count() == 0 {
                    if allowed_sources.clone().count() == 0 {
                        return fallback_to_default_src(&mut csp_fallback, request_origin);
                    }
                }

                for source in allowed_sources {
                    if matches!(source, "'none'" | "none") {
                        return Err("CSP blocks script execution".to_string());
                    }

                    if source == request_origin.unicode_serialization() || source == "*" {
                        return Ok(()); // Allowed
                    }
                }
            }

            "default-src" => {
                // Fallback for all other directives
                for source in allowed_sources {
                    csp_fallback
                        .directives
                        .entry("default-src".to_string())
                        .or_default()
                        .push(source.to_string());
                }
            }

            "font-src" => {
                if source_type != SourceType::Font {
                    continue; // Skip if not a font request
                }

                if allowed_sources.clone().count() == 0 {
                    return fallback_to_default_src(&mut csp_fallback, request_origin);
                }

                for source in allowed_sources {
                    if matches!(source, "'none'" | "none") {
                        return Err("CSP blocks script execution".to_string());
                    }

                    if source == request_origin.unicode_serialization() || source == "*" {
                        return Ok(()); // Allowed
                    }
                }
            }

            "frame-src" => {
                if source_type != SourceType::Frame {
                    continue; // Skip if not a frame request
                }

                if allowed_sources.clone().count() == 0 {
                    return fallback_to_default_src(&mut csp_fallback, request_origin);
                }

                for source in allowed_sources {
                    if matches!(source, "'none'" | "none") {
                        return Err("CSP blocks frame execution".to_string());
                    }

                    if source == request_origin.unicode_serialization() || source == "*" {
                        return Ok(()); // Allowed
                    }
                }
            }

            "script-src" => {
                if source_type != SourceType::Script {
                    continue; // Skip if not a script request
                }

                if allowed_sources.clone().count() == 0 {
                    if allowed_sources.clone().count() == 0 {
                        return fallback_to_default_src(&mut csp_fallback, request_origin);
                    }
                }

                for source in allowed_sources {
                    // TODO: Handle script-src-attr and script-src-elem
                    csp_fallback
                        .directives
                        .entry("script-src".to_string())
                        .or_default()
                        .push(source.to_string());

                    if matches!(source, "'none'" | "none") {
                        return Err("CSP blocks script execution".to_string());
                    }

                    if source == request_origin.unicode_serialization() || source == "*" {
                        return Ok(()); // Allowed
                    }
                }
            }

            "img-src" => {
                if source_type != SourceType::Image {
                    continue; // Skip if not a image request
                }

                if allowed_sources.clone().count() == 0 {
                    if allowed_sources.clone().count() == 0 {
                        return fallback_to_default_src(&mut csp_fallback, request_origin);
                    }
                }

                for source in allowed_sources {
                    if matches!(source, "'none'" | "none") {
                        return Err("CSP blocks script execution".to_string());
                    }

                    if source == request_origin.unicode_serialization() || source == "*" {
                        return Ok(()); // Allowed
                    }
                }
            }

            "media-src" => {
                if source_type != SourceType::Media {
                    continue; // Skip if not a media request
                }

                if allowed_sources.clone().count() == 0 {
                    if allowed_sources.clone().count() == 0 {
                        return fallback_to_default_src(&mut csp_fallback, request_origin);
                    }
                }

                for source in allowed_sources {
                    if matches!(source, "'none'" | "none") {
                        return Err("CSP blocks script execution".to_string());
                    }

                    if source == request_origin.unicode_serialization() || source == "*" {
                        return Ok(()); // Allowed
                    }
                }
            }

            "style-src" => {
                if source_type != SourceType::Style {
                    continue; // Skip if not a style request
                }

                if allowed_sources.clone().count() == 0 {
                    if allowed_sources.clone().count() == 0 {
                        return fallback_to_default_src(&mut csp_fallback, request_origin);
                    }
                }

                for source in allowed_sources {
                    // TODO: Handle style-src-attr and style-src-elem
                    csp_fallback
                        .directives
                        .entry("style-src".to_string())
                        .or_default()
                        .push(source.to_string());

                    if matches!(source, "'none'" | "none") {
                        return Err("CSP blocks script execution".to_string());
                    }

                    if source == request_origin.unicode_serialization() || source == "*" {
                        return Ok(()); // Allowed
                    }
                }
            }

            _ => continue, // Ignore other directives
        }
    }

    Ok(())
}

pub fn handle_csp(
    headers: &HeaderMap<HeaderValue>,
    tag_name: &str,
    request_origin: &Origin,
) -> Result<(), String> {
    let source_type = get_source_from_tag(tag_name);

    let csp = headers.get(CONTENT_SECURITY_POLICY);
    let csp_test = test_csp(
        csp.unwrap_or(&HeaderValue::from_static(""))
            .to_str()
            .unwrap_or(""),
        source_type,
        request_origin,
    );

    if let Err(e) = csp_test {
        return Err(format!("CSP violation: {}", e));
    }

    Ok(())
}
