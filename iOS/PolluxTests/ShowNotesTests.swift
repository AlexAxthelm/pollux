import Foundation
import Testing

@testable import Pollux

/// The HTML reader must run on the main thread, so the whole suite is main-actor.
@MainActor
@Suite("ShowNotes")
struct ShowNotesTests {

    @Test func stripsTagsAndDecodesEntities() throws {
        let result = try #require(
            ShowNotesRenderer.attributed(from: "<p>Hello &amp; welcome</p>"),
        )
        let plain = String(result.characters)
        #expect(plain.contains("Hello & welcome"))
        #expect(!plain.contains("<p>"))
    }

    @Test func preservesLinks() throws {
        let html = "<p>Visit <a href=\"https://example.com\">the site</a>.</p>"
        let result = try #require(ShowNotesRenderer.attributed(from: html))

        // The parser normalizes the URL (e.g. appends a trailing slash), so match
        // on the host rather than an exact string.
        let links = result.runs.compactMap { $0.link?.absoluteString }
        #expect(links.contains { $0.contains("example.com") })
        #expect(String(result.characters).contains("the site"))
    }

    @Test func plainTextPassesThrough() throws {
        let result = try #require(ShowNotesRenderer.attributed(from: "just text"))
        #expect(String(result.characters) == "just text")
    }

    @Test func collapsesTrailingBlockNewline() throws {
        // The HTML reader appends a newline after a final block; it shouldn't leak
        // into the rendered notes.
        let result = try #require(ShowNotesRenderer.attributed(from: "<p>done</p>"))
        #expect(!String(result.characters).hasSuffix("\n"))
    }
}
