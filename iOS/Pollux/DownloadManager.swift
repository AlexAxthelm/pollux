import App
import Foundation

/// Errors from the download shell. Carried in `DownloadResult.error` back to the
/// core, which turns them into a `Failed` episode row.
enum DownloadManagerError: Error, LocalizedError {
    case storageUnavailable
    case invalidURL(String)
    case badStatus(Int)

    var errorDescription: String? {
        switch self {
        case .storageUnavailable:
            "device storage is unavailable"
        case let .invalidURL(url):
            "invalid download URL: \(url)"
        case let .badStatus(code):
            "download failed: HTTP \(code)"
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

    func perform(_ operation: DownloadOperation) async -> DownloadResult {
        switch operation {
        case let .download(episodeId, url):
            await download(episodeId: episodeId, urlString: url)
        case let .delete(localPath):
            delete(localPath: localPath)
        }
    }

    // MARK: - Download

    private func download(episodeId: String, urlString: String) async -> DownloadResult {
        guard let url = URL(string: urlString) else {
            return .error(DownloadManagerError.invalidURL(urlString).localizedDescription)
        }
        do {
            try FileManager.default.createDirectory(
                at: downloadsDir, withIntermediateDirectories: true,
            )

            // PROGRESS SEAM: this uses the one-shot async download (status-only
            // pass). Live 0–100% progress is added here by switching to a
            // URLSessionDownloadDelegate whose `didWriteData` reports percent — the
            // core already accepts `DownloadResult.progress`, so that change is
            // additive and touches only this method plus the resolve loop.
            let (tempURL, response) = try await URLSession.shared.download(from: url)

            if let http = response as? HTTPURLResponse, !(200 ..< 300).contains(http.statusCode) {
                return .error(DownloadManagerError.badStatus(http.statusCode).localizedDescription)
            }

            let fileName = Self.fileName(episodeId: episodeId, url: url)
            let destination = downloadsDir.appendingPathComponent(fileName)
            // A re-download replaces any previous file at the same path.
            try? FileManager.default.removeItem(at: destination)
            try FileManager.default.moveItem(at: tempURL, to: destination)

            let size = fileSize(at: destination)
            let relativePath = "\(Self.subdirectory)/\(fileName)"
            return .completed(localPath: relativePath, sizeBytes: size)
        } catch {
            return .error(error.localizedDescription)
        }
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
