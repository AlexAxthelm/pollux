import App
import Foundation
import Shared

@MainActor
class Core: ObservableObject {
    @Published var view: ViewModel

    private var core: CoreFfi
    private let db: DatabaseManager

    init() {
        core = CoreFfi()
        do {
            db = try DatabaseManager()
        } catch {
            fatalError("Failed to initialize DatabaseManager: \(error)")
        }
        guard let view = try? ViewModel.bincodeDeserialize(input: [UInt8](core.view())) else {
            fatalError("Failed to deserialize initial ViewModel from core")
        }
        self.view = view
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
            Task { @MainActor in
                let result: StorageResult
                do {
                    result = try await db.execute(operation)
                } catch {
                    result = .error(error.localizedDescription)
                }
                resolveAndDispatch(requestId: request.id, result: result)
            }

        case let .http(operation):
            let requestId = request.id
            Task.detached { [weak self] in
                let result = await Core.fetchHttp(operation)
                await MainActor.run { [weak self] in
                    self?.resolveAndDispatch(requestId: requestId, result: result)
                }
            }
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
}
