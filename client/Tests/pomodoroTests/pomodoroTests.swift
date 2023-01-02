import XCTest
@testable import pomodoro

final class pomodoroTests: XCTestCase {
    func testExample() throws {
        // This is an example of a functional test case.
        // Use XCTAssert and related functions to verify your tests produce the correct
        // results.
        XCTAssertEqual(pomodoro().text, "Hello, World!")
    }
}
