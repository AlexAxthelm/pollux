import App
import Foundation
import Testing

@testable import Pollux

// MARK: - Test doubles

/// A stub `URLProtocol` so download tests never touch the network. A test configures
/// the canned response (status, whether a Content-Length is advertised, body bytes,
/// and whether to stall mid-flight for cancellation), and the loader delivers it in
/// small chunks so byte-progress callbacks fire.
private final class MockURLProtocol: URLProtocol, @unchecked Sendable {
    struct Stub: Sendable {
        var statusCode = 200
        var advertiseContentLength = true
        var body = Data()
        /// Deliver the response + body but never finish, so the request stays in
        /// flight until the test cancels it.
        var stall = false
    }

    private static let lock = NSLock()
    nonisolated(unsafe) private static var stub = Stub()
    nonisolated(unsafe) private static var onStart: (@Sendable () -> Void)?

    static func configure(_ stub: Stub, onStart: (@Sendable () -> Void)? = nil) {
        lock.lock()
        defer { lock.unlock() }
        self.stub = stub
        self.onStart = onStart
    }

    override class func canInit(with _: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }

    override func startLoading() {
        Self.lock.lock()
        let stub = Self.stub
        let onStart = Self.onStart
        Self.lock.unlock()

        guard let url = request.url, let client else { return }
        var headers: [String: String] = [:]
        if stub.advertiseContentLength {
            headers["Content-Length"] = String(stub.body.count)
        }
        guard let response = HTTPURLResponse(
            url: url, statusCode: stub.statusCode, httpVersion: "HTTP/1.1", headerFields: headers,
        ) else { return }
        client.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)

        // Chunk the body so more than one write callback can fire.
        let chunkSize = 64
        var offset = 0
        while offset < stub.body.count {
            let end = min(offset + chunkSize, stub.body.count)
            client.urlProtocol(self, didLoad: stub.body.subdata(in: offset ..< end))
            offset = end
        }

        onStart?()
        if stub.stall { return }
        client.urlProtocolDidFinishLoading(self)
    }

    override func stopLoading() {}
}

/// Thread-safe collector for the `onProgress` callbacks, which arrive on the session's
/// delegate queue.
private final class ProgressRecorder: @unchecked Sendable {
    private let lock = NSLock()
    private var storage: [(received: UInt64, total: UInt64?)] = []

    func record(_ received: UInt64, _ total: UInt64?) {
        lock.lock()
        defer { lock.unlock() }
        storage.append((received, total))
    }

    var samples: [(received: UInt64, total: UInt64?)] {
        lock.lock()
        defer { lock.unlock() }
        return storage
    }
}

/// One-shot signal a test awaits to learn the stub has begun loading (so it can cancel
/// the request while it is genuinely in flight).
private final class StartSignal: @unchecked Sendable {
    private let lock = NSLock()
    private var continuation: CheckedContinuation<Void, Never>?
    private var fired = false

    func fire() {
        lock.lock()
        let waiter = continuation
        continuation = nil
        fired = true
        lock.unlock()
        waiter?.resume()
    }

    func wait() async {
        await withCheckedContinuation { continuation in
            lock.lock()
            if fired {
                lock.unlock()
                continuation.resume()
            } else {
                self.continuation = continuation
                lock.unlock()
            }
        }
    }
}

// MARK: - Helpers

private func makeTempRoot() -> URL {
    FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
}

private func makeManager(root: URL, reportInterval: TimeInterval = 0) throws -> DownloadManager {
    let configuration = URLSessionConfiguration.ephemeral
    configuration.protocolClasses = [MockURLProtocol.self]
    return try DownloadManager(
        storageRoot: root, sessionConfiguration: configuration, reportInterval: reportInterval,
    )
}

private let noProgress: @Sendable (UInt64, UInt64?) -> Void = { _, _ in }

// MARK: - Tests

@Suite("DownloadManager", .serialized)
struct DownloadManagerTests {

    @Test func successfulDownloadStoresTheFileAndReportsItsSize() async throws {
        let root = makeTempRoot()
        let manager = try makeManager(root: root)
        let body = Data((0 ..< 300).map { UInt8($0 % 251) })
        MockURLProtocol.configure(.init(body: body))

        let result = await manager.perform(
            .download(episodeId: "ep-1", url: "https://example.com/a.mp3"), onProgress: noProgress,
        )

        #expect(result == .completed(localPath: "Downloads/ep_1.mp3", sizeBytes: UInt64(body.count)))
        let stored = try Data(contentsOf: root.appendingPathComponent("Downloads/ep_1.mp3"))
        #expect(stored == body)
    }

    @Test func nonSuccessStatusFailsWithoutLeavingAFile() async throws {
        let root = makeTempRoot()
        let manager = try makeManager(root: root)
        MockURLProtocol.configure(.init(statusCode: 404, body: Data("not found".utf8)))

        let result = await manager.perform(
            .download(episodeId: "ep-2", url: "https://example.com/b.mp3"), onProgress: noProgress,
        )

        guard case let .error(message) = result else {
            Issue.record("expected .error, got \(result)")
            return
        }
        #expect(message.contains("404"))
        #expect(!FileManager.default.fileExists(
            atPath: root.appendingPathComponent("Downloads/ep_2.mp3").path,
        ))
    }

    @Test func cancellingAnInFlightDownloadReturnsCancelledAndLeavesNoFile() async throws {
        let root = makeTempRoot()
        let manager = try makeManager(root: root)
        let started = StartSignal()
        MockURLProtocol.configure(
            .init(advertiseContentLength: false, body: Data(count: 128), stall: true),
            onStart: { started.fire() },
        )

        let download = Task {
            await manager.perform(
                .download(episodeId: "ep-3", url: "https://example.com/c.mp3"),
                onProgress: noProgress,
            )
        }
        await started.wait()
        _ = await manager.perform(.cancel(episodeId: "ep-3"), onProgress: noProgress)

        let result = await download.value
        #expect(result == .cancelled)
        #expect(!FileManager.default.fileExists(
            atPath: root.appendingPathComponent("Downloads/ep_3.mp3").path,
        ))
    }

    @Test func deletingAStoredFileIsIdempotent() async throws {
        let root = makeTempRoot()
        let manager = try makeManager(root: root)
        let relativePath = "Downloads/keep.mp3"
        let fileURL = root.appendingPathComponent(relativePath)
        try FileManager.default.createDirectory(
            at: fileURL.deletingLastPathComponent(), withIntermediateDirectories: true,
        )
        try Data("audio".utf8).write(to: fileURL)

        let first = await manager.perform(.delete(localPath: relativePath), onProgress: noProgress)
        #expect(first == .deleted)
        #expect(!FileManager.default.fileExists(atPath: fileURL.path))

        // Deleting the now-absent file still succeeds.
        let second = await manager.perform(.delete(localPath: relativePath), onProgress: noProgress)
        #expect(second == .deleted)
    }

    @Test func progressIsReportedWithMonotonicBytesAndTheKnownTotal() async throws {
        let root = makeTempRoot()
        let manager = try makeManager(root: root)
        let body = Data(count: 5000)
        MockURLProtocol.configure(.init(body: body))
        let recorder = ProgressRecorder()

        let result = await manager.perform(
            .download(episodeId: "ep-p", url: "https://example.com/p.mp3"),
        ) { @Sendable received, total in
            recorder.record(received, total)
        }

        #expect(result == .completed(localPath: "Downloads/ep_p.mp3", sizeBytes: UInt64(body.count)))
        let samples = recorder.samples
        #expect(!samples.isEmpty)
        // Every report carries the advertised total, and received bytes never go backwards.
        #expect(samples.allSatisfy { $0.total == UInt64(body.count) })
        #expect(samples.map(\.received) == samples.map(\.received).sorted())
        #expect(samples.last?.received == UInt64(body.count))
    }

    @Test func unknownSizeProgressIsThrottled() async throws {
        let root = makeTempRoot()
        // A long interval means the time throttle should let at most the first report
        // through, no matter how many write callbacks a chunked body produces.
        let manager = try makeManager(root: root, reportInterval: 1000)
        // ~1600 chunks of 64 bytes: without throttling this would report constantly.
        MockURLProtocol.configure(.init(advertiseContentLength: false, body: Data(count: 100_000)))
        let recorder = ProgressRecorder()

        let result = await manager.perform(
            .download(episodeId: "ep-u", url: "https://example.com/u.mp3"),
        ) { @Sendable received, total in
            recorder.record(received, total)
        }

        #expect(result == .completed(localPath: "Downloads/ep_u.mp3", sizeBytes: 100_000))
        let samples = recorder.samples
        // The throttle caps reporting at the single initial callback.
        #expect(samples.count <= 1)
        // Any report that did fire carries no total (the size was unknown).
        #expect(samples.allSatisfy { $0.total == nil })
    }
}
