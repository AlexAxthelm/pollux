/// Reduces an HTML fragment to a single line of plain text, for compact snippets
/// (e.g. an episode row's description preview).
///
/// This is deliberately not a real HTML parser: it drops anything between `<` and
/// `>`, decodes a handful of common entities, and collapses whitespace. That is
/// enough for a truncated one-line summary; the full show notes are rendered from
/// the original HTML shell-side, where a real reader is available.
pub fn strip_html(input: &str) -> String {
    // Tags become a space so adjacent blocks (</p><p>, <br>) don't run their words
    // together; runs of whitespace are collapsed afterwards.
    let mut without_tags = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => {
                in_tag = true;
                without_tags.push(' ');
            }
            '>' => in_tag = false,
            _ if !in_tag => without_tags.push(ch),
            _ => {}
        }
    }

    let decoded = decode_entities(&without_tags);
    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Decodes the small set of named entities common in feed descriptions. `&amp;`
/// is decoded last so an encoded entity like `&amp;lt;` is not double-decoded.
fn decode_entities(input: &str) -> String {
    input
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_tags() {
        assert_eq!(strip_html("<p>Hello <b>world</b></p>"), "Hello world");
    }

    #[test]
    fn separates_adjacent_blocks() {
        assert_eq!(strip_html("<p>one</p><p>two</p>"), "one two");
        assert_eq!(strip_html("line<br>break"), "line break");
    }

    #[test]
    fn collapses_whitespace() {
        assert_eq!(strip_html("  a\n\n  b\t c  "), "a b c");
    }

    #[test]
    fn decodes_common_entities() {
        assert_eq!(strip_html("R&amp;B"), "R&B");
        assert_eq!(strip_html("a &lt;tag&gt; b"), "a <tag> b");
        assert_eq!(strip_html("she said &quot;hi&quot;"), "she said \"hi\"");
        assert_eq!(strip_html("it&#39;s fine"), "it's fine");
    }

    #[test]
    fn nbsp_becomes_collapsible_space() {
        assert_eq!(strip_html("tune&nbsp;in"), "tune in");
    }

    #[test]
    fn does_not_double_decode() {
        // "&amp;lt;" should decode once to "&lt;", not all the way to "<".
        assert_eq!(strip_html("&amp;lt;"), "&lt;");
    }

    #[test]
    fn plain_text_passes_through() {
        assert_eq!(strip_html("just text"), "just text");
    }

    #[test]
    fn empty_input_is_empty() {
        assert_eq!(strip_html(""), "");
        assert_eq!(strip_html("   "), "");
    }
}
