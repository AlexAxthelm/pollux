/// A one-line plain-text preview of an HTML description, at most `max_chars`
/// characters. Only a bounded prefix of the input is processed: an episode row
/// shows a single line, so stripping an entire (possibly multi-KB) show note would
/// be wasted work. This keeps the per-episode cost fixed regardless of how long
/// the description is, so it is cheap to run in `view()` for a whole feed.
pub fn strip_html_preview(input: &str, max_chars: usize) -> String {
    // Tags and entities shrink when stripped, so read a generous multiple of
    // max_chars to be sure enough visible text remains — but still a small,
    // bounded amount however long the description is.
    let budget = max_chars.saturating_mul(4).max(max_chars + 16);
    let mut prefix: String = input.chars().take(budget).collect();
    // A tag cut in half at the budget boundary would otherwise leak its '<';
    // drop the dangling opener so it can't appear in the preview.
    if let Some(open) = prefix.rfind('<') {
        if !prefix[open..].contains('>') {
            prefix.truncate(open);
        }
    }
    strip_html(&prefix).chars().take(max_chars).collect()
}

/// Reduces an HTML fragment to a single line of plain text. Deliberately not a
/// real HTML parser: it drops anything between `<` and `>`, decodes a handful of
/// common entities, and collapses whitespace. Callers go through
/// `strip_html_preview`; the full show notes are rendered from the original HTML
/// shell-side, where a real reader is available.
fn strip_html(input: &str) -> String {
    // Block/break tags become a space so adjacent blocks (</p><p>, <br>) don't run
    // their words together; inline tags (<b>, <i>, <a>) are dropped without a space
    // so they don't split a word they were only emphasizing. Whitespace is then
    // collapsed.
    let mut without_tags = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(lt) = rest.find('<') {
        without_tags.push_str(&rest[..lt]);
        let after = &rest[lt + 1..];

        // Comments are skipped to their closing "-->", which can itself contain a
        // '>' that must not be mistaken for the end of a tag. An unterminated
        // comment runs to the end of the input, as browsers treat it.
        if let Some(body) = after.strip_prefix("!--") {
            rest = match body.find("-->") {
                Some(end) => &body[end + 3..],
                None => "",
            };
            continue;
        }

        match after.find('>') {
            Some(gt) => {
                if is_separating_tag(&after[..gt]) {
                    without_tags.push(' ');
                }
                rest = &after[gt + 1..];
            }
            // Unterminated '<': treat it (and the remainder) as literal text.
            None => {
                without_tags.push('<');
                rest = after;
            }
        }
    }
    without_tags.push_str(rest);

    let decoded = decode_entities(&without_tags);
    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whether a tag's removal should leave a separating space. True for block-level
/// and line-break tags (which delimit text), false for inline tags. `tag` is the
/// text between `<` and `>`, e.g. `"p"`, `"/p"`, `"br/"`, or `"a href=…"`.
fn is_separating_tag(tag: &str) -> bool {
    let name: String = tag
        .trim_start_matches('/')
        .chars()
        .take_while(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect();
    matches!(
        name.as_str(),
        "p" | "br"
            | "div"
            | "li"
            | "ul"
            | "ol"
            | "tr"
            | "hr"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "blockquote"
            | "section"
            | "article"
            | "header"
            | "footer"
            | "table"
            | "pre"
            | "dd"
            | "dt"
    )
}

/// Decodes HTML entities in a single left-to-right pass, so an already-decoded
/// character is never re-scanned as part of another entity (e.g. `&amp;lt;`
/// decodes once to `&lt;`, not all the way to `<`). Both named entities common in
/// feed descriptions and numeric references (`&#8217;`, `&#x2019;`) are handled;
/// an unrecognized `&…;` is emitted verbatim.
fn decode_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let after = &rest[amp + 1..];
        if let Some(semi) = after.find(';') {
            if let Some(decoded) = decode_entity(&after[..semi]) {
                out.push(decoded);
                rest = &after[semi + 1..];
                continue;
            }
        }
        // Not a recognizable entity: keep the '&' literally and move past it.
        out.push('&');
        rest = after;
    }
    out.push_str(rest);
    out
}

/// Decodes a single entity body (the text between `&` and `;`), or `None` if it
/// isn't one we recognize.
fn decode_entity(body: &str) -> Option<char> {
    match body {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some(' '),
        "mdash" => Some('\u{2014}'),
        "ndash" => Some('\u{2013}'),
        "hellip" => Some('\u{2026}'),
        "lsquo" => Some('\u{2018}'),
        "rsquo" => Some('\u{2019}'),
        "ldquo" => Some('\u{201C}'),
        "rdquo" => Some('\u{201D}'),
        _ => decode_numeric(body),
    }
}

/// Decodes a numeric character reference: decimal `#8217` or hex `#x2019`.
fn decode_numeric(body: &str) -> Option<char> {
    let digits = body.strip_prefix('#')?;
    let code = match digits.strip_prefix(['x', 'X']) {
        Some(hex) => u32::from_str_radix(hex, 16).ok()?,
        None => digits.parse::<u32>().ok()?,
    };
    char::from_u32(code)
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
    fn drops_comments_including_ones_containing_angle_brackets() {
        // A '>' inside a comment must not end the comment early and leak text.
        assert_eq!(strip_html("before<!-- a > b -->after"), "beforeafter");
        assert_eq!(strip_html("x<!-- just a note -->y"), "xy");
        // An unterminated comment runs to the end of the input.
        assert_eq!(strip_html("keep<!-- oops"), "keep");
    }

    #[test]
    fn inline_tags_do_not_split_words() {
        // Inline emphasis inside a word must not introduce a space...
        assert_eq!(strip_html("un<i>believ</i>able"), "unbelievable");
        assert_eq!(strip_html("a<b>b</b>c"), "abc");
        // ...while block/break tags still separate their neighbours.
        assert_eq!(strip_html("a<br/>b"), "a b");
        assert_eq!(strip_html("<li>one</li><li>two</li>"), "one two");
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
    fn decodes_numeric_entities() {
        // Decimal and hex references for the right single quote (U+2019).
        assert_eq!(strip_html("it&#8217;s"), "it\u{2019}s");
        assert_eq!(strip_html("it&#x2019;s"), "it\u{2019}s");
        // Uppercase hex prefix is also valid.
        assert_eq!(strip_html("&#X2019;"), "\u{2019}");
    }

    #[test]
    fn decodes_common_punctuation_entities() {
        assert_eq!(
            strip_html("dash &mdash; and &hellip; more"),
            "dash \u{2014} and \u{2026} more"
        );
    }

    #[test]
    fn unrecognized_entity_is_left_verbatim() {
        // Not a known name and not numeric: emit it unchanged rather than dropping.
        assert_eq!(strip_html("a &bogus; b"), "a &bogus; b");
        // A bare ampersand (no terminating ';') is also preserved.
        assert_eq!(strip_html("Q &amp A"), "Q &amp A");
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

    #[test]
    fn preview_truncates_to_max_chars() {
        assert_eq!(strip_html_preview("<p>Hello world</p>", 5), "Hello");
        // Whole content when it already fits.
        assert_eq!(strip_html_preview("<p>hi</p>", 100), "hi");
        // Tags and entities are resolved before the length is measured.
        assert_eq!(strip_html_preview("A &amp; B and more", 5), "A & B");
    }

    #[test]
    fn preview_output_never_contains_markup() {
        // Whatever the input (including a tag cut off at the end), the preview must
        // not leak angle brackets.
        for input in [
            "plain text",
            "<p>hello <b>there</b></p>",
            "<a href=\"http://x\">link</a> and a good deal more text after it",
            "trailing <partial",
        ] {
            let preview = strip_html_preview(input, 40);
            assert!(
                !preview.contains('<') && !preview.contains('>'),
                "leaked markup for {input:?}: {preview:?}"
            );
            assert!(preview.chars().count() <= 40);
        }
    }
}
