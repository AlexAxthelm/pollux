import SwiftUI
import UIKit

/// Renders an episode's HTML show notes as rich text. Feeds deliver descriptions
/// as HTML (`<p>`, `<a>`, `<br>`, entities like `&nbsp;`), so rendering — not just
/// stripping — is a shell/platform concern (UIKit's HTML reader). Parsing is done
/// once in a `.task`, never in `body`: the HTML reader spins up WebKit and is far
/// too costly to run every render (and must never run per-row in a long list).
struct ShowNotesText: View {
    let html: String
    /// Plain-text fallback (the core's stripped description) shown for the frame
    /// before the first parse, and permanently if parsing fails. Never raw HTML.
    var fallback: String?

    @Environment(\.themeColors) private var themeColors
    @State private var rendered: AttributedString?

    var body: some View {
        Group {
            if let rendered {
                Text(rendered)
                    .tint(themeColors.accent)
            } else {
                Text(fallback ?? "")
                    .foregroundStyle(themeColors.text)
            }
        }
        .task(id: html) {
            rendered = ShowNotesRenderer.attributed(from: html)
        }
    }
}

enum ShowNotesRenderer {
    /// Parses HTML into an `AttributedString`, re-basing every run onto the app's
    /// Dynamic Type body font (preserving bold/italic traits) and dropping the
    /// reader's hard-coded colors so text follows the label color in light/dark.
    /// Returns nil if the string isn't decodable as HTML.
    static func attributed(from html: String) -> AttributedString? {
        guard let data = html.data(using: .utf8) else { return nil }
        let options: [NSAttributedString.DocumentReadingOptionKey: Any] = [
            .documentType: NSAttributedString.DocumentType.html,
            .characterEncoding: String.Encoding.utf8.rawValue,
        ]
        guard
            let parsed = try? NSMutableAttributedString(
                data: data,
                options: options,
                documentAttributes: nil,
            )
        else {
            return nil
        }

        let whole = NSRange(location: 0, length: parsed.length)
        let body = UIFont.preferredFont(forTextStyle: .body)
        parsed.enumerateAttribute(.font, in: whole, options: []) { value, range, _ in
            let traits = (value as? UIFont)?.fontDescriptor.symbolicTraits ?? []
            let descriptor = body.fontDescriptor.withSymbolicTraits(traits) ?? body.fontDescriptor
            parsed.addAttribute(.font, value: UIFont(descriptor: descriptor, size: body.pointSize), range: range)
        }
        parsed.removeAttribute(.foregroundColor, range: whole)

        // Trim the trailing newline the HTML reader appends after a final block.
        while parsed.string.hasSuffix("\n"), parsed.length > 0 {
            parsed.deleteCharacters(in: NSRange(location: parsed.length - 1, length: 1))
        }

        return AttributedString(parsed)
    }
}
