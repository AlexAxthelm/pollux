import App
import Foundation
import Shared

@MainActor
class Core: ObservableObject {
    @Published var view: ViewModel

    private var core: CoreFfi
    private let db: DatabaseManager
    private let downloads: DownloadManager
    private var playback: PlaybackManager?
    private var nowPlaying: NowPlayingController?
    /// The session feed fetches go through. Injectable so tests can stub the network.
    private let feedSession: URLSession
    /// Asks the system for a future background-refresh wake-up. Injectable so tests can
    /// observe it without touching `BGTaskScheduler`.
    private let scheduleBackgroundRefresh: @MainActor () -> Void
    /// Storage operations are funnelled here and executed one at a time, in order.
    private let storage: AsyncStream<(StorageOperation, UInt32)>.Continuation

    /// The app's real collaborators: the on-disk database and downloads folder, the shared
    /// URL session, and the system scheduler.
    convenience init() {
        let db: DatabaseManager
        do {
            db = try DatabaseManager()
        } catch {
            fatalError("Failed to initialize DatabaseManager: \(error)")
        }
        let downloads: DownloadManager
        do {
            downloads = try DownloadManager()
        } catch {
            fatalError("Failed to initialize DownloadManager: \(error)")
        }
        self.init(db: db, downloads: downloads)
    }

    /// Everything the shell does on the core's behalf is injectable, so tests can drive a
    /// real `Core` (and the real Rust core behind it) against a temporary database and a
    /// stubbed network.
    init(
        db: DatabaseManager,
        downloads: DownloadManager,
        feedSession: URLSession = .shared,
        scheduleBackgroundRefresh: @escaping @MainActor () -> Void = { BackgroundRefresh.schedule() },
    ) {
        core = CoreFfi()
        self.db = db
        self.downloads = downloads
        self.feedSession = feedSession
        self.scheduleBackgroundRefresh = scheduleBackgroundRefresh
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
        if let root = DownloadManager.defaultStorageRoot() {
            playback = PlaybackManager(storageRoot: root) { [weak self] event in
                self?.update(event)
            }
        }
        nowPlaying = NowPlayingController { [weak self] event in
            self?.update(event)
        }
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

    // MARK: - Lifecycle

    /// The app came to the foreground (including at cold launch). The core decides which
    /// feeds are due, and holds the request if the library hasn't loaded yet, so sending
    /// this on every activation is safe. Interrupted downloads resume only here, never from
    /// `Started`: a background launch (the refresh task) never becomes active, so it can't
    /// start a download inside its short window. The core ignores every activation after the
    /// first for that.
    func appBecameActive() {
        update(.refreshStale)
        update(.resumePendingDownloads)
    }

    /// The app went to the background: flush the playback position (the app may be
    /// suspended or killed next) and queue the next best-effort wake-up.
    func appEnteredBackground() {
        update(.appBackgrounded)
        scheduleBackgroundRefresh()
    }

    /// The body of the system's background-refresh task. Reschedules first so the chain
    /// survives a run cut short by the system, then refreshes whatever is due until the
    /// queue drains or the task is cancelled at expiry.
    func runBackgroundRefresh() async {
        scheduleBackgroundRefresh()
        await refreshStaleAndWait()
    }

    /// Requests a refresh of due feeds and returns once the library has loaded and the
    /// refresh queue has drained (or the calling task is cancelled). Used by the
    /// background task, which has no UI to observe. `loading` is true from launch until
    /// the library loads, so waiting on it also covers a cold background launch, where the
    /// core holds the request until there are subscriptions to judge.
    ///
    /// When the system expires the task, the task is cancelled; that stops this wait, but
    /// the core's queue is in-memory Rust state that task cancellation can't reach. So
    /// cancellation also sends `.cancelRefresh`, which drops the feeds still waiting. The
    /// fetch already in flight is left to finish: the core records its outcome, and
    /// nothing further starts.
    func refreshStaleAndWait() async {
        update(.refreshStale)
        await withTaskCancellationHandler {
            await Polling.waitWhileBusy { view.library.loading || view.library.refreshing }
        } onCancel: {
            Task { @MainActor in self.update(.cancelRefresh) }
        }
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
            nowPlaying?.update(updatedView.player)

        case let .storage(operation):
            // Enqueue for the serial consumer set up in init (preserves write order).
            storage.yield((operation, request.id))

        case let .http(operation):
            let requestId = request.id
            let session = feedSession
            Task.detached { [weak self] in
                let result = await Core.fetchHttp(operation, session: session)
                await MainActor.run { [weak self] in
                    self?.resolveAndDispatch(requestId: requestId, result: result)
                }
            }

        case let .download(operation):
            handleDownload(operation, requestId: request.id)

        case let .player(operation):
            // The engine acts synchronously; resolve on the next turn so the result
            // doesn't re-enter the core while it is still handing us this batch.
            let result = playback?.perform(operation) ?? .error("Audio engine unavailable")
            let requestId = request.id
            Task { @MainActor [weak self] in
                self?.resolveAndDispatch(requestId: requestId, result: result)
            }
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

    private static func fetchHttp(_ operation: HttpOperation, session: URLSession) async -> HttpResult {
        switch operation {
        case let .fetchFeed(url, etag, lastModified):
            await FeedFetcher.fetch(url: url, etag: etag, lastModified: lastModified, session: session)
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

    private func resolveAndDispatch(requestId: UInt32, result: PlayerResult) {
        guard let bytes = try? result.bincodeSerialize() else {
            fatalError("Failed to serialize PlayerResult for request \(requestId)")
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
