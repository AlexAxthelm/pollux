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

    /// The in-flight download's task and episode, so a Cancel operation can find and
    /// cancel it. At most one at a time (downloads are serial). Set before the await
    /// and cleared when it returns.
    private var activeTask: (episodeId: String, task: URLSessionDownloadTask)?

    /// Subdirectory (and stored-path prefix) for downloaded audio.
    private static let subdirectory = "Downloads"

    init() throws {
        guard let support = FileManager.default.urls(
            for: .applicationSupportDirectory, in: .userDomainMask,
        ).first else {
            throw DownloadManagerError.storageUnavailable
        }
        storageRoot = support
        downloadsDir = support.appendingPathComponent(Self.subdirectory, isDirectory: true)
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
            try FileManager.default.createDirectory(
                at: downloadsDir, withIntermediateDirectories: true,
            )
            let fileName = Self.fileName(episodeId: episodeId, url: url)
            let destination = downloadsDir.appendingPathComponent(fileName)

            // A session-level delegate is required for byte progress: the async
            // download(from:delegate:) convenience never invokes didWriteData. The
            // delegate reports progress, moves the finished temp file to `destination`
            // (the system temp file is valid only inside didFinishDownloadingTo), and
            // resumes the await; a non-2xx response fails the download.
            let delegate = DownloadSessionDelegate(destination: destination, onProgress: onProgress)
            let session = URLSession(configuration: .default, delegate: delegate, delegateQueue: nil)
            let task = session.downloadTask(with: url)
            // Registered before the await so a Cancel can reach it; cleared after.
            activeTask = (episodeId: episodeId, task: task)
            defer {
                activeTask = nil
                session.finishTasksAndInvalidate()
            }

            let finalURL: URL = try await withCheckedThrowingContinuation { continuation in
                delegate.attach(continuation)
                task.resume()
            }

            let size = fileSize(at: finalURL)
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

    private func fileSize(at url: URL) -> UInt64 {
        guard let attributes = try? FileManager.default.attributesOfItem(atPath: url.path),
              let size = attributes[.size] as? UInt64
        else {
            return 0
        }
        return size
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
    private var continuation: CheckedContinuation<URL, Error>?
    private var movedURL: URL?
    private var completionError: Error?
    private var lastPercent = -1

    init(destination: URL, onProgress: @escaping @Sendable (UInt64, UInt64?) -> Void) {
        self.destination = destination
        self.onProgress = onProgress
    }

    func attach(_ continuation: CheckedContinuation<URL, Error>) {
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
        // When the size is known, coalesce to whole-percent steps so the core gets at
        // most ~100 updates per download rather than one per network packet. When it's
        // unknown, forward as-is (already packet-paced). onProgress runs outside the lock.
        if let total, total > 0 {
            let percent = Int(received * 100 / total)
            lock.lock()
            let unchanged = percent == lastPercent
            if !unchanged {
                lastPercent = percent
            }
            lock.unlock()
            if unchanged {
                return
            }
        }
        onProgress(received, total)
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
        lock.unlock()

        guard let continuation else { return }
        if let error {
            continuation.resume(throwing: error)
        } else if let finishError {
            continuation.resume(throwing: finishError)
        } else if let moved {
            continuation.resume(returning: moved)
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
