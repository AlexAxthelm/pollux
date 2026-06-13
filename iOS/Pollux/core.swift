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
                input: [UInt8](core.view())
            ) else {
                fatalError("Failed to deserialize ViewModel during render")
            }
            view = updatedView

        case .storage(let operation):
            Task {
                let result: StorageResult
                do {
                    result = try await db.execute(operation)
                } catch {
                    result = .error(error.localizedDescription)
                }
                resolveAndDispatch(requestId: request.id, result: result)
            }
        }
    }

    private func resolveAndDispatch(requestId: UInt32, result: StorageResult) {
        guard let resultBytes = try? result.bincodeSerialize() else { return }
        let newEffects = [UInt8](core.resolve(id: requestId, data: Data(resultBytes)))
        guard let newRequests: [Request] = try? .bincodeDeserialize(input: newEffects) else { return }
        for req in newRequests { processEffect(req) }
    }
}
