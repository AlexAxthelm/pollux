import App
import Foundation

/// Errors from the download shell. Carried in `DownloadResult.error` back to the
/// core, which turns them into a `Failed` episode row.
enum DownloadManagerError: Error, LocalizedError {
    case storageUnavailable
    case invalidURL(String)
    case badStatus(Int)
    case incomplete

    var errorDescription: String? {
        switch self {
        case .storageUnavailable:
            "device storage is unavailable"
        case let .invalidURL(url):
            "invalid download URL: \(url)"
        case let .badStatus(code):
            "download failed: HTTP \(code)"
        case .incomplete:
            "download ended without a file"
        }
    }
}

/// Performs the actual episode-audio I/O for the `Download` capability: fetching a
/// file to local storage and removing it again. The core owns *when* to download
/// (and the serial queue); this type only carries out one operation at a time and
/// reports the outcome.
///
/// Files live under `<Application Support>/Downloads/`. The path returned to the
/// core (and stored in the DB) is **relative** to Application Support, so it keeps
/// resolving after an OS container-path change; `absoluteURL(for:)` turns it back
/// into a usable URL.
actor DownloadManager {
    private let storageRoot: URL
    private let downloadsDir: URL
    /// Configuration for the download session. Injectable so tests can register a
    /// stub `URLProtocol` instead of hitting the network.
    private let sessionConfiguration: URLSessionConfiguration
    /// Progress-report throttle passed to the session delegate (see there). Injectable
    /// so tests don't depend on a real one-second wall-clock gap.
    private let reportInterval: TimeInterval

    /// The in-flight download's task and episode, so a Cancel operation can find and
    /// cancel it. At most one at a time (downloads are serial). Set before the await
    /// and cleared when it returns.
    private var activeTask: (episodeId: String, task: URLSessionDownloadTask)?

    /// Subdirectory (and stored-path prefix) for downloaded audio.
    private static let subdirectory = "Downloads"

    /// - Parameters:
    ///   - storageRoot: where downloads live; defaults to Application Support. Tests
    ///     pass a temporary directory.
    ///   - sessionConfiguration: URLSession configuration; tests register a stub
    ///     `URLProtocol` here.
    ///   - reportInterval: throttle for progress reports on unknown-size downloads.
    init(
        storageRoot: URL? = nil,
        sessionConfiguration: URLSessionConfiguration = .default,
        reportInterval: TimeInterval = 1,
    ) throws {
        let root: URL
        if let storageRoot {
            root = storageRoot
        } else {
            guard let support = FileManager.default.urls(
                for: .applicationSupportDirectory, in: .userDomainMask,
            ).first else {
                throw DownloadManagerError.storageUnavailable
            }
            root = support
        }
        self.storageRoot = root
        downloadsDir = root.appendingPathComponent(Self.subdirectory, isDirectory: true)
        self.sessionConfiguration = sessionConfiguration
        self.reportInterval = reportInterval
    }

    /// `onProgress(received, total)` is called with byte counts while a download runs
    /// (total is nil when the server didn't report a size). It never fires for a delete.
    func perform(
        _ operation: DownloadOperation,
        onProgress: @escaping @Sendable (UInt64, UInt64?) -> Void,
    ) async -> DownloadResult {
        switch operation {
        case let .download(episodeId, url):
            await download(episodeId: episodeId, urlString: url, onProgress: onProgress)
        case let .delete(localPath):
            delete(localPath: localPath)
        case let .cancel(episodeId):
            cancel(episodeId: episodeId)
        }
    }

    // MARK: - Download

    private func download(
        episodeId: String, urlString: String,
        onProgress: @escaping @Sendable (UInt64, UInt64?) -> Void,
    ) async -> DownloadResult {
        guard let url = URL(string: urlString) else {
            return .error(DownloadManagerError.invalidURL(urlString).localizedDescription)
        }
        do {
            try ensureDownloadsDirectory()
            let fileName = Self.fileName(episodeId: episodeId, url: url)
            let destination = downloadsDir.appendingPathComponent(fileName)

            // A session-level delegate is required for byte progress: the async
            // download(from:delegate:) convenience never invokes didWriteData. The
            // delegate reports progress, moves the finished temp file to `destination`
            // (the system temp file is valid only inside didFinishDownloadingTo), and
            // resumes the await; a non-2xx response fails the download.
            let delegate = DownloadSessionDelegate(
                destination: destination, reportInterval: reportInterval, onProgress: onProgress,
            )
            let session = URLSession(
                configuration: sessionConfiguration, delegate: delegate, delegateQueue: nil,
            )
            let task = session.downloadTask(with: url)
            // Registered before the await so a Cancel can reach it; cleared after.
            activeTask = (episodeId: episodeId, task: task)
            defer {
                activeTask = nil
                session.finishTasksAndInvalidate()
            }

            // The delegate reports the completed file's byte count, so there's no
            // filesystem re-read that could fail and record a size of 0.
            let (_, size): (URL, UInt64) = try await withCheckedThrowingContinuation { continuation in
                delegate.attach(continuation)
                task.resume()
            }

            let relativePath = "\(Self.subdirectory)/\(fileName)"
            return .completed(localPath: relativePath, sizeBytes: size)
        } catch {
            // A cancelled task throws URLError.cancelled; URLSession discards its own
            // temp file, and we clear any file that reached `destination` to be sure.
            if (error as? URLError)?.code == .cancelled {
                let fileName = Self.fileName(episodeId: episodeId, url: url)
                try? FileManager.default.removeItem(
                    at: downloadsDir.appendingPathComponent(fileName),
                )
                return .cancelled
            }
            return .error(error.localizedDescription)
        }
    }

    // MARK: - Cancel

    private func cancel(episodeId: String) -> DownloadResult {
        // Cancelling the task makes the in-flight download throw URLError.cancelled,
        // which resolves the Download request as `.cancelled`; that is where the real
        // state change happens, so this just acks.
        if let activeTask, activeTask.episodeId == episodeId {
            activeTask.task.cancel()
        }
        return .cancelled
    }

    // MARK: - Delete

    private func delete(localPath: String) -> DownloadResult {
        let target = absoluteURL(for: localPath)
        // Idempotent: a file that is already gone is a successful delete.
        guard FileManager.default.fileExists(atPath: target.path) else {
            return .deleted
        }
        do {
            try FileManager.default.removeItem(at: target)
            return .deleted
        } catch {
            return .error(error.localizedDescription)
        }
    }

    // MARK: - Paths

    /// Creates the downloads directory if needed and marks it excluded from device
    /// backups. Downloaded audio is reproducible, so it shouldn't consume the user's
    /// iCloud backup quota or bloat their backups. The exclusion is best-effort — a
    /// failure to set the flag doesn't fail the download.
    private func ensureDownloadsDirectory() throws {
        try FileManager.default.createDirectory(
            at: downloadsDir, withIntermediateDirectories: true,
        )
        var dir = downloadsDir
        var values = URLResourceValues()
        values.isExcludedFromBackup = true
        try? dir.setResourceValues(values)
    }

    private func absoluteURL(for relativePath: String) -> URL {
        storageRoot.appendingPathComponent(relativePath)
    }

    /// A filesystem-safe, stable file name for an episode. The episode id can be a
    /// GUID, URL, or hash, so it is reduced to alphanumerics; the enclosure's file
    /// extension is preserved when present so players see a familiar type.
    private static func fileName(episodeId: String, url: URL) -> String {
        let safeId = episodeId.map { $0.isLetter || $0.isNumber ? $0 : "_" }
        let base = String(safeId)
        let ext = url.pathExtension
        return ext.isEmpty ? base : "\(base).\(ext)"
    }
}

/// Session delegate driving a single download. Reports byte progress via
/// didWriteData, moves the finished file to `destination` synchronously (the
/// system-supplied temp file is valid only inside didFinishDownloadingTo), and
/// resumes the awaiting continuation on completion. A non-2xx response fails it.
///
/// The delegate callbacks run on URLSession's delegate queue while `attach` is
/// called from the caller's thread, so all mutable state is guarded by `lock`
/// (which is what makes the `@unchecked Sendable` sound).
private final class DownloadSessionDelegate: NSObject, URLSessionDownloadDelegate, @unchecked Sendable {
    private let destination: URL
    private let onProgress: @Sendable (UInt64, UInt64?) -> Void
    private let lock = NSLock()
    private var continuation: CheckedContinuation<(URL, UInt64), Error>?
    private var movedURL: URL?
    private var completionError: Error?
    private var lastPercent = -1
    /// Last time an indeterminate-size download reported progress, for time throttling.
    private var lastReportAt = Date.distantPast
    /// Bytes written so far; the final value is the completed file's size, so it can be
    /// reported without a filesystem re-read (which could fail and record a size of 0).
    private var receivedBytes: UInt64 = 0

    /// Minimum spacing between progress reports when the total size is unknown and
    /// there is no percentage to coalesce on (~1 update per second in production).
    private let reportInterval: TimeInterval

    init(
        destination: URL, reportInterval: TimeInterval,
        onProgress: @escaping @Sendable (UInt64, UInt64?) -> Void,
    ) {
        self.destination = destination
        self.reportInterval = reportInterval
        self.onProgress = onProgress
    }

    func attach(_ continuation: CheckedContinuation<(URL, UInt64), Error>) {
        lock.lock()
        defer { lock.unlock() }
        self.continuation = continuation
    }

    func urlSession(
        _: URLSession, downloadTask _: URLSessionDownloadTask,
        didWriteData _: Int64, totalBytesWritten: Int64, totalBytesExpectedToWrite: Int64,
    ) {
        let received = UInt64(max(0, totalBytesWritten))
        let total: UInt64? = totalBytesExpectedToWrite > 0
            ? UInt64(totalBytesExpectedToWrite) : nil
        // Record every callback (so the final value is the completed size), but coalesce
        // onProgress so the core isn't flooded with an update per write: to whole-percent
        // steps when the size is known (~100 updates), or to ~1/sec when it isn't (no
        // percentage to step on). onProgress runs outside the lock.
        lock.lock()
        receivedBytes = received
        let report: Bool
        if let total, total > 0 {
            let percent = Int(received * 100 / total)
            report = percent != lastPercent
            if report {
                lastPercent = percent
            }
        } else {
            let now = Date()
            report = now.timeIntervalSince(lastReportAt) >= reportInterval
            if report {
                lastReportAt = now
            }
        }
        lock.unlock()
        if report {
            onProgress(received, total)
        }
    }

    func urlSession(
        _: URLSession, downloadTask: URLSessionDownloadTask, didFinishDownloadingTo location: URL,
    ) {
        let status = (downloadTask.response as? HTTPURLResponse)?.statusCode
        if let status, !(200 ..< 300).contains(status) {
            setCompletionError(DownloadManagerError.badStatus(status))
            return
        }
        do {
            // A re-download replaces any previous file at the same path.
            try? FileManager.default.removeItem(at: destination)
            try FileManager.default.moveItem(at: location, to: destination)
            lock.lock()
            movedURL = destination
            lock.unlock()
        } catch {
            setCompletionError(error)
        }
    }

    func urlSession(
        _: URLSession, task _: URLSessionTask, didCompleteWithError error: Error?,
    ) {
        // Snapshot under the lock, then resume outside it (resume can run the awaiting
        // task, which must not happen while we hold the lock).
        lock.lock()
        let continuation = continuation
        self.continuation = nil
        let finishError = completionError
        let moved = movedURL
        let bytes = receivedBytes
        lock.unlock()

        guard let continuation else { return }
        if let error {
            continuation.resume(throwing: error)
        } else if let finishError {
            continuation.resume(throwing: finishError)
        } else if let moved {
            continuation.resume(returning: (moved, bytes))
        } else {
            continuation.resume(throwing: DownloadManagerError.incomplete)
        }
    }

    private func setCompletionError(_ error: Error) {
        lock.lock()
        defer { lock.unlock() }
        completionError = error
    }
}
