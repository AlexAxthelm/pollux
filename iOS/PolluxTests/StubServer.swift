import Foundation

/// An in-process fake HTTP server for tests that drive a real `Core`: it answers requests
/// from canned replies, records what it was asked, and can hold a reply until the test
/// releases it (to freeze a fetch mid-flight).
///
/// Each server owns a unique host name (`t-<uuid>.test`), and requests are routed to the
/// server that owns their host. Tests run in parallel, so any shared static stub would let
/// them trample each other; a per-test host cannot collide.
final class StubServer: @unchecked Sendable {
    struct Reply: Sendable {
        var status = 200
        var headers: [String: String] = [:]
        var body = Data()
        /// Don't answer until `release(_:)` is called for this path.
        var hold = false
    }

    struct Request: Sendable {
        let path: String
        let headers: [String: String]
    }

    let host = "t-\(UUID().uuidString.lowercased()).test"

    private let lock = NSLock()
    private var replies: [String: Reply] = [:]
    private var seen: [Request] = []
    private var held: [String: StubProtocol] = [:]

    init() {
        StubRegistry.shared.add(self)
    }

    deinit {
        StubRegistry.shared.remove(host: host)
    }

    func url(_ path: String) -> String {
        "https://\(host)\(path)"
    }

    func reply(_ path: String, _ reply: Reply) {
        lock.lock()
        defer { lock.unlock() }
        replies[path] = reply
    }

    var requests: [Request] {
        lock.lock()
        defer { lock.unlock() }
        return seen
    }

    var requestedPaths: [String] {
        requests.map(\.path)
    }

    func requestCount(for path: String) -> Int {
        requests.count(where: { $0.path == path })
    }

    /// Delivers the reply for a request that is being held at `path`. A no-op if none is.
    func release(_ path: String) {
        lock.lock()
        let proto = held.removeValue(forKey: path)
        let reply = replies[path]
        lock.unlock()
        guard let proto, let reply else { return }
        proto.deliver(reply)
    }

    /// A session whose requests are answered by this server.
    func session() -> URLSession {
        URLSession(configuration: configuration())
    }

    /// A configuration whose requests are answered by this server (for `DownloadManager`).
    func configuration() -> URLSessionConfiguration {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [StubProtocol.self]
        return configuration
    }

    // Called by `StubProtocol` on a URLSession thread.
    fileprivate func handle(_ proto: StubProtocol, path: String, headers: [String: String]) {
        lock.lock()
        seen.append(Request(path: path, headers: headers))
        let reply = replies[path] ?? Reply(status: 404)
        if reply.hold {
            held[path] = proto
            lock.unlock()
            return
        }
        lock.unlock()
        proto.deliver(reply)
    }
}

/// Maps a host to the `StubServer` that owns it. Servers register on creation and drop out
/// when they deallocate, so a finished test leaves nothing behind.
private final class StubRegistry: @unchecked Sendable {
    static let shared = StubRegistry()

    private let lock = NSLock()
    private var servers: [String: Weak] = [:]

    private struct Weak {
        weak var server: StubServer?
    }

    func add(_ server: StubServer) {
        lock.lock()
        defer { lock.unlock() }
        servers[server.host] = Weak(server: server)
    }

    func remove(host: String) {
        lock.lock()
        defer { lock.unlock() }
        servers.removeValue(forKey: host)
    }

    func server(for host: String?) -> StubServer? {
        guard let host else { return nil }
        lock.lock()
        defer { lock.unlock() }
        return servers[host]?.server
    }
}

final class StubProtocol: URLProtocol, @unchecked Sendable {
    override class func canInit(with _: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }

    override func startLoading() {
        guard let url = request.url else { return }
        guard let server = StubRegistry.shared.server(for: url.host) else {
            client?.urlProtocol(self, didFailWithError: URLError(.cannotFindHost))
            return
        }
        server.handle(self, path: url.path, headers: request.allHTTPHeaderFields ?? [:])
    }

    override func stopLoading() {}

    /// Sends `reply` to the client. May be called later, from another thread, for a held
    /// request.
    func deliver(_ reply: StubServer.Reply) {
        guard let url = request.url, let client else { return }
        var headers = reply.headers
        headers["Content-Length"] = String(reply.body.count)
        guard let response = HTTPURLResponse(
            url: url, statusCode: reply.status, httpVersion: "HTTP/1.1", headerFields: headers,
        ) else { return }
        client.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
        if !reply.body.isEmpty {
            client.urlProtocol(self, didLoad: reply.body)
        }
        client.urlProtocolDidFinishLoading(self)
    }
}
