import Foundation

@available(macOS 13.0, *)
class WebSocketClient: NSObject {
    private let session = URLSession(configuration: .default)
    private var webSocketTask: URLSessionWebSocketTask?

    func connect(url: URL) {
        webSocketTask = session.webSocketTask(with: url)
        webSocketTask?.resume()
        print("connected")
    }

    func disconnect() {
        webSocketTask?.cancel()
        print("disconnected")
    }

    func sendMessage(_ message: Data) async -> Result<URLSessionWebSocketTask.Message, Error> {
        do {
            try await webSocketTask!.send(.data(message))

            let response = try await webSocketTask!.receive()

            return .success(response)
        } catch {
            return .failure(error)
        }
    }

    func sendMessage(_ message: String) async -> Result<URLSessionWebSocketTask.Message, Error> {
        do {
            try await webSocketTask!.send(.string(message))

            let response = try await webSocketTask!.receive()

            return .success(response)
        } catch {
            return .failure(error)
        }
    }
}

@available(macOS 13.0, *)
extension WebSocketClient: URLSessionWebSocketDelegate {
    func urlSession(_: URLSession, webSocketTask _: URLSessionWebSocketTask, didOpenWithProtocol _: String?) {
        print("opened/connected")
    }

    func urlSession(_: URLSession, webSocketTask _: URLSessionWebSocketTask, didCloseWith _: URLSessionWebSocketTask.CloseCode, reason _: Data?) {
        print("closed/disconnected")
    }
}

@available(macOS 13.0, *)
enum Command: String, Codable {
    case Pause
    case Resume
    case Show
    case Stop
    case Start
    case SkipBreak
    case Reset
}

@available(macOS 13.0, *)
enum CommandResult: Codable {
    case Success(Pomodoro)
    case Failure
}

@available(macOS 13.0, *)
enum State: Codable {
    case Paused(duration: Duration)
    case Stopped
    case Working(duration: Duration)
    case TakingShortBreak(duration: Duration)
    case TakingLongBreak(duration: Duration)

    enum CodingKeys: String, CodingKey {
        case Paused
        case Stopped
        case Working
        case TakingShortBreak
        case TakingLongBreak
    }

    enum AdditionalCodingKeys: String, CodingKey {
        case duration
        case secs
        case nanos
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if let state = try? container.decode(String.self) {
            // For when it's "state":"Stopped"
            switch state {
            case "Stopped":
                self = .Stopped
            default:
                fatalError("Unexpected value \(state)")
            }
        } else {
            // For when "state":{"Working":{"duration":{"secs":1,"nanos":0}}}
            let values = try decoder.container(keyedBy: CodingKeys.self)

            // Dynamically get the CodingKey for the State from its enum
            let stateKey = values.allKeys.first!
            let stateContainer = try values.nestedContainer(
                keyedBy: AdditionalCodingKeys.self, forKey: stateKey
            )

            let durationKey = stateContainer.allKeys.first!
            let durationContainer = try stateContainer.nestedContainer(
                keyedBy: AdditionalCodingKeys.self, forKey: durationKey
            )

            let nanos = try durationContainer.decode(Int.self, forKey: .nanos)
            let secs = try durationContainer.decode(Int.self, forKey: .secs)

            let duration = Duration.nanoseconds(nanos) + Duration.seconds(secs)

            let state: State = {
                switch stateKey.stringValue {
                case "Paused":
                    return State.Paused(duration: duration)
                case "Working":
                    return State.Working(duration: duration)
                case "TakingShortBreak":
                    return State.TakingShortBreak(duration: duration)
                case "TakingLongBreak":
                    return State.TakingLongBreak(duration: duration)
                default:
                    fatalError("Unexpected value \(stateKey.stringValue)")
                }
            }()

            self = state
        }
    }
}

@available(macOS 13.0, *)
struct Pomodoro: Codable {
    var state: State
    var completed_count: UInt64
    var break_count: UInt64
}

@available(macOS 13.0, *)
enum Output: String, Codable {
    case Success
    case Failure
}

@available(macOS 13.0, *)
struct Response: Codable {
    var command: Command
    var result: Output
    var pomodoro: Pomodoro?
}

@main
@available(macOS 13.0, *)
public enum pomodoro {
    public static func main() async {
        let client = WebSocketClient()
        let url = URL(string: "ws://127.0.0.1:3012")!
        client.connect(url: url)

        let command = CommandLine.arguments[1]

        let result = await client.sendMessage(command)

        switch result {
        case let .success(message):
            switch message {
            case let .data(data):
                let decoder = JSONDecoder()
                do {
                    let command_result = try decoder.decode(Response.self, from: data)
                    print(command_result)
                } catch {
                    print(error)
                    print(error.localizedDescription)
                }

            case let .string(string):
                let decoder = JSONDecoder()
                do {
                    let command_result = try decoder.decode(Response.self, from: string.data(using: .ascii)!)
                    print(command_result)
                } catch {
                    print(error)
                    print(error.localizedDescription)
                }

            default:
                print(message)
            }
        case let .failure(error):
            print(error)
        }

        client.disconnect()
    }
}
