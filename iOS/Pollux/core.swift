import App
import Foundation
import Shared

@MainActor
class Core: ObservableObject {
    @Published var view: ViewModel

    private var core: CoreFfi
    private let db: DatabaseManager
    private let downloads: DownloadManager
    /// Storage operations are funnelled here and executed one at a time, in order.
    private let storage: AsyncStream<(StorageOperation, UInt32)>.Continuation

    init() {
        core = CoreFfi()
        do {
            db = try DatabaseManager()
        } catch {
            fatalError("Failed to initialize DatabaseManager: \(error)")
        }
        do {
            downloads = try DownloadManager()
        } catch {
            fatalError("Failed to initialize DownloadManager: \(error)")
        }
        // One serial consumer so writes for a given episode apply in submission order
        // (Queued → Downloading → terminal). A Task per op could reorder them — a fast
        // failure/cancel racing the Downloading write would leave a stale status in the
        // DB and re-download the episode after restart.
        let (storageStream, storageContinuation) =
            AsyncStream.makeStream(of: (StorageOperation, UInt32).self)
        storage = storageContinuation
        guard let view = try? ViewModel.bincodeDeserialize(input: [UInt8](core.view())) else {
            fatalError("Failed to deserialize initial ViewModel from core")
        }
        self.view = view
        Task { @MainActor [weak self] in
            for await (operation, requestId) in storageStream {
                guard let self else { return }
                let result: StorageResult
                do {
                    result = try await db.execute(operation)
                } catch {
                    result = .error(error.localizedDescription)
                }
                resolveAndDispatch(requestId: requestId, result: result)
            }
        }
        update(.started)
    }

    func update(_ event: Event) {
        guard let serialized = try? event.bincodeSerialize() else {
            fatalError("Failed to serialize Event: \(event)")
        }
        let effects = [UInt8](core.update(data: Data(serialized)))

        guard let requests: [Request] = try? .bincodeDeserialize(input: effects) else {
            fatalError("Failed to deserialize requests from core effects")
        }
        for request in requests {
            processEffect(request)
        }
    }

    func processEffect(_ request: Request) {
        switch request.effect {
        case .render:
            guard let updatedView = try? ViewModel.bincodeDeserialize(
                input: [UInt8](core.view()),
            ) else {
                fatalError("Failed to deserialize ViewModel during render")
            }
            view = updatedView

        case let .storage(operation):
            // Enqueue for the serial consumer set up in init (preserves write order).
            storage.yield((operation, request.id))

        case let .http(operation):
            let requestId = request.id
            Task.detached { [weak self] in
                let result = await Core.fetchHttp(operation)
                await MainActor.run { [weak self] in
                    self?.resolveAndDispatch(requestId: requestId, result: result)
                }
            }

        case let .download(operation):
            handleDownload(operation, requestId: request.id)
        }
    }

    /// Runs a download-capability operation on the DownloadManager actor (so the UI
    /// isn't blocked) and resolves the request with its terminal result. Byte progress
    /// is funnelled through a stream drained by a single consumer, so DownloadProgress
    /// events reach the core in order — a Task-per-callback could reorder them.
    private func handleDownload(_ operation: DownloadOperation, requestId: UInt32) {
        let progressEpisodeId: String? = {
            if case let .download(episodeId, _) = operation {
                return episodeId
            }
            return nil
        }()
        Task { @MainActor in
            let (progressStream, progressContinuation) =
                AsyncStream.makeStream(of: (UInt64, UInt64?).self)
            let consumer = Task { @MainActor [weak self] in
                for await (received, total) in progressStream {
                    guard let progressEpisodeId else { continue }
                    self?.update(.downloadProgress(
                        episodeId: progressEpisodeId,
                        receivedBytes: received,
                        totalBytes: total,
                    ))
                }
            }
            let result = await downloads.perform(operation) { received, total in
                progressContinuation.yield((received, total))
            }
            // Drain any buffered progress before the terminal result lands.
            progressContinuation.finish()
            await consumer.value
            resolveAndDispatch(requestId: requestId, result: result)
        }
    }

    // MARK: - HTTP

    private static func fetchHttp(_ operation: HttpOperation) async -> HttpResult {
        switch operation {
        case let .fetchFeed(url):
            await fetchFeed(url: url)
        }
    }

    private static func fetchFeed(url: String) async -> HttpResult {
        guard let parsedURL = URL(string: url) else {
            return .error("invalid URL: \(url)")
        }
        do {
            let (data, response) = try await URLSession.shared.data(from: parsedURL)
            let status = (response as? HTTPURLResponse)?.statusCode ?? 0
            return .response(status: UInt16(clamping: status), body: Array(data))
        } catch {
            return .error(error.localizedDescription)
        }
    }

    // MARK: - Resolve

    private func resolveBytes(requestId: UInt32, bytes: [UInt8]) {
        let newEffects = [UInt8](core.resolve(id: requestId, data: Data(bytes)))
        guard let newRequests: [Request] = try? .bincodeDeserialize(input: newEffects) else {
            fatalError("Failed to deserialize effects after resolving request \(requestId)")
        }
        for req in newRequests {
            processEffect(req)
        }
    }

    private func resolveAndDispatch(requestId: UInt32, result: StorageResult) {
        guard let bytes = try? result.bincodeSerialize() else {
            fatalError("Failed to serialize StorageResult for request \(requestId)")
        }
        resolveBytes(requestId: requestId, bytes: bytes)
    }

    private func resolveAndDispatch(requestId: UInt32, result: HttpResult) {
        guard let bytes = try? result.bincodeSerialize() else {
            fatalError("Failed to serialize HttpResult for request \(requestId)")
        }
        resolveBytes(requestId: requestId, bytes: bytes)
    }

    private func resolveAndDispatch(requestId: UInt32, result: DownloadResult) {
        guard let bytes = try? result.bincodeSerialize() else {
            fatalError("Failed to serialize DownloadResult for request \(requestId)")
        }
        resolveBytes(requestId: requestId, bytes: bytes)
    }
}
