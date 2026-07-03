import Foundation

struct Client {
    let title: String

    init(title: String) {
        self.title = title
    }

    func fetch(id: String) -> String {
        id
    }
}

protocol Screen {
    var name: String { get }
    func render()
}

extension Client {
    func reset() {}
}

enum Mode {
    case light
    case dark
}
