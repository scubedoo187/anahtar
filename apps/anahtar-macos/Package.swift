// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "AnahtarMac",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .executable(name: "Anahtar", targets: ["AnahtarMac"])
    ],
    targets: [
        .executableTarget(
            name: "AnahtarMac",
            path: "Sources/AnahtarMac",
            linkerSettings: [
                .unsafeFlags(["-L", "../../target/aarch64-apple-darwin/release", "-lanahtar_ffi"])
            ]
        )
    ]
)
