import App
import Foundation
import Testing

@testable import Pollux

// MARK: - Test double

/// A stub `URLProtocol` for the feed fetch, so tests never touch the network. It records
/// the request it receives (headers and cache policy) and replies with a canned response
/// or a transport failure. State is static because URLSession instantiates the protocol
/// itself, so the suite is `.serialized`.
private final class FeedStubProtocol: URLProtocol, @unchecked Sendable {
    struct Stub: Sendable {
        var statusCode = 200
        var headers: [String: String] = [:]
        var body = Data()
        var failure: URLError?
    }

    private static let lock = NSLock()
    nonisolated(unsafe) private static var stub = Stub()
    nonisolated(unsafe) private static var lastRequest: URLRequest?

    static func configure(_ stub: Stub) {
        lock.lock()
        defer { lock.unlock() }
        self.stub = stub
        lastRequest = nil
    }

    static var received: URLRequest? {
        lock.lock()
        defer { lock.unlock() }
        return lastRequest
    }

    override class func canInit(with _: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }

    override func startLoading() {
        Self.lock.lock()
        let stub = Self.stub
        Self.lastRequest = request
        Self.lock.unlock()

        guard let client else { return }
        if let failure = stub.failure {
            client.urlProtocol(self, didFailWithError: failure)
            return
        }
        guard let url = request.url, let response = HTTPURLResponse(
            url: url, statusCode: stub.statusCode, httpVersion: "HTTP/1.1", headerFields: stub.headers,
        ) else { return }
        client.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
        if !stub.body.isEmpty {
            client.urlProtocol(self, didLoad: stub.body)
        }
        client.urlProtocolDidFinishLoading(self)
    }

    override func stopLoading() {}
}

private func makeSession() -> URLSession {
    let configuration = URLSessionConfiguration.ephemeral
    configuration.protocolClasses = [FeedStubProtocol.self]
    return URLSession(configuration: configuration)
}

private let feedURL = "https://example.com/feed.rss"

private struct Fetched {
    var status: UInt16
    var body: [UInt8]
    var etag: String?
    var lastModified: String?
    var retryAfterSecs: UInt64?
}

/// Runs a fetch and unpacks a `.response`, recording an issue for any other result.
private func fetch(etag: String? = nil, lastModified: String? = nil) async -> Fetched? {
    let result = await FeedFetcher.fetch(
        url: feedURL, etag: etag, lastModified: lastModified, session: makeSession(),
    )
    guard case let .response(status, body, etag, lastModified, retryAfterSecs) = result else {
        Issue.record("Expected .response, got \(result)")
        return nil
    }
    return Fetched(
        status: status, body: body, etag: etag, lastModified: lastModified, retryAfterSecs: retryAfterSecs,
    )
}

// MARK: - Tests

@Suite("FeedFetcher", .serialized)
struct FeedFetcherTests {
    @Test func sendsStoredValidatorsAsConditionalHeaders() async throws {
        FeedStubProtocol.configure(.init(statusCode: 304))

        _ = await fetch(etag: "\"v1\"", lastModified: "Wed, 01 Oct 2026 00:00:00 GMT")

        let request = try #require(FeedStubProtocol.received)
        #expect(request.value(forHTTPHeaderField: "If-None-Match") == "\"v1\"")
        #expect(request.value(forHTTPHeaderField: "If-Modified-Since") == "Wed, 01 Oct 2026 00:00:00 GMT")
    }

    @Test func sendsNoConditionalHeadersWithoutValidators() async throws {
        FeedStubProtocol.configure(.init())

        _ = await fetch()

        let request = try #require(FeedStubProtocol.received)
        #expect(request.value(forHTTPHeaderField: "If-None-Match") == nil)
        #expect(request.value(forHTTPHeaderField: "If-Modified-Since") == nil)
    }

    @Test func sendsOnlyTheValidatorsItHas() async throws {
        FeedStubProtocol.configure(.init())

        _ = await fetch(etag: "\"only-etag\"")

        let request = try #require(FeedStubProtocol.received)
        #expect(request.value(forHTTPHeaderField: "If-None-Match") == "\"only-etag\"")
        #expect(request.value(forHTTPHeaderField: "If-Modified-Since") == nil)
    }

    @Test func bypassesTheLocalCacheSoA304IsNotMasked() async throws {
        FeedStubProtocol.configure(.init())

        _ = await fetch(etag: "\"v1\"")

        let request = try #require(FeedStubProtocol.received)
        #expect(request.cachePolicy == .reloadIgnoringLocalCacheData)
    }

    @Test func usesAShortIdleTimeoutSoAHungHostCannotStallTheQueue() async throws {
        FeedStubProtocol.configure(.init())

        _ = await fetch()

        let request = try #require(FeedStubProtocol.received)
        #expect(request.timeoutInterval == FeedFetcher.requestTimeout)
        #expect(
            request.timeoutInterval < 30,
            "must fit inside a background refresh's ~30s budget, not URLSession's 60s default",
        )
    }

    @Test func appliesTheTimeoutToConditionalRequestsToo() async throws {
        FeedStubProtocol.configure(.init(statusCode: 304))

        _ = await fetch(etag: "\"v1\"", lastModified: "Wed, 01 Oct 2026 00:00:00 GMT")

        let request = try #require(FeedStubProtocol.received)
        #expect(request.timeoutInterval == FeedFetcher.requestTimeout)
    }

    @Test func forwardsStatusBodyAndValidatorHeaders() async {
        FeedStubProtocol.configure(.init(
            statusCode: 200,
            headers: ["ETag": "\"v2\"", "Last-Modified": "Thu, 02 Oct 2026 00:00:00 GMT"],
            body: Data("<rss/>".utf8),
        ))

        let fetched = await fetch()

        #expect(fetched?.status == 200)
        #expect(fetched?.body == Array("<rss/>".utf8))
        #expect(fetched?.etag == "\"v2\"")
        #expect(fetched?.lastModified == "Thu, 02 Oct 2026 00:00:00 GMT")
        #expect(fetched?.retryAfterSecs == nil)
    }

    @Test func passesThroughA304WithNoBody() async {
        FeedStubProtocol.configure(.init(statusCode: 304, headers: ["ETag": "\"v1\""]))

        let fetched = await fetch(etag: "\"v1\"")

        #expect(fetched?.status == 304, "a 304 must reach the core, not be turned into a cached 200")
        #expect(fetched?.body.isEmpty == true)
        #expect(fetched?.etag == "\"v1\"")
    }

    @Test func normalizesRetryAfterOnA429() async {
        FeedStubProtocol.configure(.init(statusCode: 429, headers: ["Retry-After": "120"]))

        let fetched = await fetch()

        #expect(fetched?.status == 429)
        #expect(fetched?.retryAfterSecs == 120)
    }

    @Test func missingHeadersComeBackNil() async {
        FeedStubProtocol.configure(.init(statusCode: 200))

        let fetched = await fetch()

        #expect(fetched?.etag == nil)
        #expect(fetched?.lastModified == nil)
        #expect(fetched?.retryAfterSecs == nil)
    }

    @Test(arguments: [
        URLError.Code.notConnectedToInternet, .networkConnectionLost, .dataNotAllowed,
        .internationalRoamingOff, .callIsActive,
    ])
    func reportsADeviceConnectivityFailureAsUnreachable(code: URLError.Code) async {
        FeedStubProtocol.configure(.init(failure: URLError(code)))

        let result = await FeedFetcher.fetch(
            url: feedURL, etag: nil, lastModified: nil, session: makeSession(),
        )

        guard case .unreachable = result else {
            Issue.record("Expected .unreachable for \(code), got \(result)")
            return
        }
    }

    @Test(arguments: [
        URLError.Code.cannotConnectToHost, .cannotFindHost, .timedOut, .secureConnectionFailed,
        .badServerResponse,
    ])
    func reportsAHostSideTransportFailureAsAnError(code: URLError.Code) async {
        // These can't be told apart from a dead or refusing host, so they stay plain
        // errors and the core backs the feed off.
        FeedStubProtocol.configure(.init(failure: URLError(code)))

        let result = await FeedFetcher.fetch(
            url: feedURL, etag: nil, lastModified: nil, session: makeSession(),
        )

        guard case .error = result else {
            Issue.record("Expected .error for \(code), got \(result)")
            return
        }
    }

    @Test func rejectsAnInvalidURLWithoutTouchingTheNetwork() async {
        FeedStubProtocol.configure(.init())

        let result = await FeedFetcher.fetch(
            // `URL(string:)` is lenient (it percent-encodes "not a url"); only a string it
            // can't represent at all, like an empty one, is rejected.
            url: "", etag: nil, lastModified: nil, session: makeSession(),
        )

        guard case let .error(message) = result else {
            Issue.record("Expected .error, got \(result)")
            return
        }
        #expect(message.contains("invalid URL"))
        #expect(FeedStubProtocol.received == nil)
    }
}
