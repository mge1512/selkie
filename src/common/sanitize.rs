//! Text sanitization utilities

use regex::Regex;
use std::sync::LazyLock;

use crate::config::{Config, SecurityLevel};

// Script tag patterns
static SCRIPT_BLOCK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap()
});

static SCRIPT_SRC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<script[^>]*src\s*=\s*["'][^"']*["'][^>]*>\s*</script>"#).unwrap()
});

static JAVASCRIPT_URL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<a[^>]*href\s*=\s*["']javascript[^"']*["'][^>]*>(.*?)</a>"#).unwrap()
});

static JAVASCRIPT_COLON_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<a[^>]*href\s*=\s*["']javascript&colon;[^"']*["'][^>]*>(.*?)</a>"#).unwrap()
});

static IMG_ONERROR_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<img\s*[^>]*onerror\s*=\s*["'][^"']*["'][^>]*>"#).unwrap()
});

static IFRAME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)<iframe[^>]*>.*?</iframe>").unwrap()
});

static TARGET_BLANK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)<a([^>]*target\s*=\s*["']_blank["'][^>]*)>"#).unwrap()
});

static HAS_REL_NOOPENER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)rel\s*=\s*["'][^"']*noopener[^"']*["']"#).unwrap()
});

/// Remove script tags and dangerous patterns from HTML
pub fn remove_script(text: &str) -> String {
    let mut result = text.to_string();

    // Remove script blocks with content
    result = SCRIPT_BLOCK_RE.replace_all(&result, "").to_string();

    // Remove script tags with src
    result = SCRIPT_SRC_RE.replace_all(&result, "").to_string();

    // Remove javascript: URLs
    result = JAVASCRIPT_URL_RE.replace_all(&result, "<a>$1</a>").to_string();

    // Remove javascript&colon; URLs
    result = JAVASCRIPT_COLON_RE.replace_all(&result, "<a>$1</a>").to_string();

    // Remove onerror handlers from images
    result = IMG_ONERROR_RE.replace_all(&result, "<img>").to_string();

    // Remove iframes
    result = IFRAME_RE.replace_all(&result, "").to_string();

    // Add rel="noopener" to target="_blank" links that don't have it
    result = TARGET_BLANK_RE.replace_all(&result, |caps: &regex::Captures| {
        let attrs = &caps[1];
        if HAS_REL_NOOPENER_RE.is_match(attrs) {
            format!("<a{}>", attrs)
        } else {
            format!("<a{} rel=\"noopener\">", attrs)
        }
    }).to_string();

    result
}

/// Sanitize text based on security level
pub fn sanitize_text(text: &str, config: &Config) -> String {
    match config.security_level {
        SecurityLevel::Strict => {
            // Remove javascript: protocol attempts with various bypass attempts
            let mut result = text.to_string();

            // Handle "javajavascript:script:" pattern
            while result.contains("javascript:") {
                result = result.replace("javascript:", "");
            }

            result = remove_script(&result);
            result
        }
        SecurityLevel::Sandbox => {
            // In sandbox mode, allow HTML but remove scripts
            remove_script(text)
        }
        SecurityLevel::Loose | SecurityLevel::Antiscript => {
            remove_script(text)
        }
    }
}

/// Parse generic type notation using tildes (e.g., `test~T~` -> `test<T>`)
///
/// This converts TypeScript-style generic notation using tildes to angle brackets.
/// For example: `Array~Array~string~~` becomes `Array<Array<string>>`
pub fn parse_generic_types(input: &str) -> String {
    // Split on commas first
    let input_sets: Vec<&str> = input.split(',').collect();
    let mut output: Vec<String> = Vec::new();

    let mut i = 0;
    while i < input_sets.len() {
        let this_set = input_sets[i];

        // Check if we need to combine with previous and next sets
        if i > 0 && i + 1 < input_sets.len() {
            let previous_set = output.last().map(|s| s.as_str()).unwrap_or("");
            let next_set = input_sets[i + 1];

            if should_combine_sets(previous_set, next_set) {
                // Combine previous, comma, and next
                let combined = format!("{},{},{}", output.pop().unwrap_or_default(), this_set, next_set);
                output.push(process_set(&combined));
                i += 2; // Skip the next set since we combined it
                continue;
            }
        }

        output.push(process_set(this_set));
        i += 1;
    }

    output.join(",")
}

fn should_combine_sets(previous_set: &str, next_set: &str) -> bool {
    let prev_count = count_occurrence(previous_set, "~");
    let next_count = count_occurrence(next_set, "~");
    prev_count == 1 && next_count == 1
}

fn process_set(input: &str) -> String {
    let tilde_count = count_occurrence(input, "~");

    if tilde_count <= 1 {
        return input.to_string();
    }

    let mut input = input.to_string();
    let mut has_starting_tilde = false;

    // If there is an odd number of tildes, and the input starts with a tilde,
    // we need to remove it and add it back in later
    if tilde_count % 2 != 0 && input.starts_with('~') {
        input = input[1..].to_string();
        has_starting_tilde = true;
    }

    let mut chars: Vec<char> = input.chars().collect();

    loop {
        let first = chars.iter().position(|&c| c == '~');
        let last = chars.iter().rposition(|&c| c == '~');

        match (first, last) {
            (Some(f), Some(l)) if f != l => {
                chars[f] = '<';
                chars[l] = '>';
            }
            _ => break,
        }
    }

    // Add the starting tilde back in if we removed it
    if has_starting_tilde {
        chars.insert(0, '~');
    }

    chars.into_iter().collect()
}

/// Count occurrences of a substring in a string
pub fn count_occurrence(text: &str, substring: &str) -> usize {
    if substring.is_empty() {
        return 0;
    }
    text.matches(substring).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    mod remove_script_tests {
        use super::*;

        fn compare_remove_script(original: &str, result: &str) {
            assert_eq!(remove_script(original).trim(), result);
        }

        #[test]
        fn should_remove_all_script_block_script_inline() {
            let label_string = r#"1
		Act1: Hello 1<script src="http://abc.com/script1.js"></script>1
		<b>Act2</b>:
		1<script>
			alert('script run......');
		</script>1
	1"#;
            let exactly_string = r#"1
		Act1: Hello 11
		<b>Act2</b>:
		11
	1"#;
            compare_remove_script(label_string, exactly_string);
        }

        #[test]
        fn should_remove_all_javascript_urls() {
            compare_remove_script(
                r#"This is a <a href="javascript:runHijackingScript();">clean link</a> + <a href="javascript:runHijackingScript();">clean link</a>
  and <a href="javascript&colon;bypassedMining();">me too</a>"#,
                r#"This is a <a>clean link</a> + <a>clean link</a>
  and <a>me too</a>"#,
            );
        }

        #[test]
        fn should_detect_malicious_images() {
            compare_remove_script("<img onerror=\"alert('hello');\">", "<img>");
        }

        #[test]
        fn should_detect_unsecured_target_blank_and_add_noopener() {
            compare_remove_script(
                r#"<a href="https://mermaid.js.org/" target="_blank">note about mermaid</a>"#,
                r#"<a href="https://mermaid.js.org/" target="_blank" rel="noopener">note about mermaid</a>"#,
            );
        }

        #[test]
        fn should_not_modify_target_self() {
            compare_remove_script(
                r#"<a href="https://mermaid.js.org/" target="_self">note about mermaid</a>"#,
                r#"<a href="https://mermaid.js.org/" target="_self">note about mermaid</a>"#,
            );
        }

        #[test]
        fn should_detect_iframes() {
            compare_remove_script(
                r#"<iframe src="http://abc.com/script1.js"></iframe>
    <iframe src="http://example.com/iframeexample"></iframe>"#,
                "",
            );
        }
    }

    mod sanitize_text_tests {
        use super::*;

        #[test]
        fn should_remove_script_tag() {
            let malicious_str = "javajavascript:script:alert(1)";
            let config = Config {
                security_level: SecurityLevel::Strict,
                ..Default::default()
            };
            let result = sanitize_text(malicious_str, &config);
            assert!(!result.contains("javascript:alert(1)"));
        }

        #[test]
        fn should_allow_html_tags_in_sandbox_mode() {
            let html_str = "<p>This is a <strong>bold</strong> text</p>";
            let config = Config {
                security_level: SecurityLevel::Sandbox,
                ..Default::default()
            };
            let result = sanitize_text(html_str, &config);
            assert!(result.contains("<p>"));
            assert!(result.contains("<strong>"));
            assert!(result.contains("</strong>"));
            assert!(result.contains("</p>"));
        }

        #[test]
        fn should_remove_script_tags_in_sandbox_mode() {
            let malicious_str = "<p>Hello <script>alert(1)</script> world</p>";
            let config = Config {
                security_level: SecurityLevel::Sandbox,
                ..Default::default()
            };
            let result = sanitize_text(malicious_str, &config);
            assert!(!result.contains("<script>"));
            assert!(!result.contains("alert(1)"));
            assert!(result.contains("<p>"));
            assert!(result.contains("Hello"));
            assert!(result.contains("world"));
        }
    }

    mod generic_parser_tests {
        use super::*;

        #[test]
        fn should_parse_generic_types() {
            let test_cases = vec![
                ("test~T~", "test<T>"),
                ("test~Array~Array~string~~~", "test<Array<Array<string>>>"),
                ("test~Array~Array~string[]~~~", "test<Array<Array<string[]>>>"),
                ("test ~Array~Array~string[]~~~", "test <Array<Array<string[]>>>"),
                ("~test", "~test"),
                ("~test~T~", "~test<T>"),
            ];

            for (input, expected) in test_cases {
                assert_eq!(parse_generic_types(input), expected, "Failed for input: {}", input);
            }
        }
    }

    mod count_occurrence_tests {
        use super::*;

        #[test]
        fn should_count_occurrences() {
            let test_cases = vec![
                ("", "", 0),
                ("", "x", 0),
                ("test", "x", 0),
                ("test", "t", 2),
                ("test", "te", 1),
                ("test~T~", "~", 2),
                ("test~Array~Array~string~~~", "~", 6),
            ];

            for (text, substring, count) in test_cases {
                assert_eq!(
                    count_occurrence(text, substring),
                    count,
                    "Failed for text: '{}', substring: '{}'",
                    text,
                    substring
                );
            }
        }
    }
}
