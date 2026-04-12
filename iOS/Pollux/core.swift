import App
import Foundation
import Shared

@MainActor
class Core: ObservableObject {
    @Published var view: ViewModel

    private var core: CoreFfi

    init() {
        core = CoreFfi()
        guard let view = try? ViewModel.bincodeDeserialize(input: [UInt8](core.view())) else {
            fatalError("Failed to deserialize initial ViewModel from core")
        }
        self.view = view
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
            DispatchQueue.main.async {
                guard let updatedView = try? ViewModel.bincodeDeserialize(
                    input: [UInt8](self.core.view()),
                ) else {
                    fatalError("Failed to deserialize ViewModel during render")
                }
                self.view = updatedView
            }
        }
    }
}
