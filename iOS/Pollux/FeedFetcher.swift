import App
import Foundation

/// Fetches a feed over HTTP for the core's `HttpOperation.fetchFeed`. Split out of `Core`
/// so the session is injectable: tests drive it with a stub `URLProtocol` and check the
/// conditional-request headers it sends and the validator headers it hands back.
enum FeedFetcher {
    /// Fetches a feed, replaying the stored validators as a conditional GET so an
    /// unchanged feed answers 304 with no body. The cache policy ignores URLSession's
    /// own cache: with our validators attached, a locally cached 200 would otherwise be
    /// substituted for the 304 we need to see.
    static func fetch(
        url: String, etag: String?, lastModified: String?, session: URLSession = .shared,
    ) async -> HttpResult {
        guard let parsedURL = URL(string: url) else {
            return .error("invalid URL: \(url)")
        }
        var request = URLRequest(url: parsedURL, cachePolicy: .reloadIgnoringLocalCacheData)
        if let etag {
            request.setValue(etag, forHTTPHeaderField: "If-None-Match")
        }
        if let lastModified {
            request.setValue(lastModified, forHTTPHeaderField: "If-Modified-Since")
        }
        do {
            let (data, response) = try await session.data(for: request)
            let http = response as? HTTPURLResponse
            return .response(
                status: UInt16(clamping: http?.statusCode ?? 0),
                body: Array(data),
                etag: http?.value(forHTTPHeaderField: "ETag"),
                lastModified: http?.value(forHTTPHeaderField: "Last-Modified"),
                retryAfterSecs: retryAfterSeconds(http?.value(forHTTPHeaderField: "Retry-After")),
            )
        } catch {
            return .error(error.localizedDescription)
        }
    }

    /// Normalizes a `Retry-After` header (delay-seconds or HTTP-date) to whole seconds
    /// from now, so the core needs no date parsing. Nil when absent or unparseable.
    static func retryAfterSeconds(_ value: String?, now: Date = Date()) -> UInt64? {
        guard let value = value?.trimmingCharacters(in: .whitespaces), !value.isEmpty else {
            return nil
        }
        if let seconds = UInt64(value) {
            return seconds
        }
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.timeZone = TimeZone(identifier: "GMT")
        formatter.dateFormat = "EEE, dd MMM yyyy HH:mm:ss zzz"
        guard let date = formatter.date(from: value) else { return nil }
        return UInt64(max(0, date.timeIntervalSince(now).rounded(.up)))
    }
}
