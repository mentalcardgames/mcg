# QR Communication Protocol

This document describes the technical implementation of the QR-based communication channel introduced in the `qr_comm_test_incorporation` branch.

## Concept: Visual Network Coding

The goal is to transmit arbitrary binary data (files, game state, crypto keys) between devices using *only* the screen (as transmitter) and camera (as receiver).

**Challenge**: QR codes have limited capacity. A video stream of QR codes can transmit large data, but frames might be dropped (camera loses focus, tearing, lighting issues).
**Solution**: **Fountain Codes (Network Coding)**. Instead of chopping the file into pieces 1, 2, 3... and needing specifically piece #2 if it's lost, we transmit mathematical combinations of the pieces.
-   If the file is split into $N$ fragments.
-   The sender generates infinite random linear combinations (Equation frames).
-   The receiver just needs to collect *any* $N$ (plus a small overhead) linearly independent frames to solve the system and recover the file.

## Core Structs & Modules

The `qr_comm` crate implements the math and data framing for this protocol.

-   **`Package`** @ [crates/qr_comm/src/data_structures/application_package.rs](../crates/qr_comm/src/data_structures/application_package.rs):
    -   **Purpose**: Represents the application-layer binary data holding files or messages.
    -   **Usage**: Wraps arbitrary data with its size. It provides methods to split the data into smaller, uniform `Fragment`s (`into_fragments`) and to re-assemble those fragments back into a full package (`from_fragments`).

-   **`Fragment`** @ [crates/qr_comm/src/data_structures/fragment.rs](../crates/qr_comm/src/data_structures/fragment.rs):
    -   **Purpose**: A fixed-size slice of a `Package`.
    -   **Usage**: The atomic unit over which math operations (like XOR and Galois field multiplication) are performed when generating combinations.

-   **`FrameFactor`** & **`Factor`** @ [crates/qr_comm/src/data_structures/factors.rs](../crates/qr_comm/src/data_structures/factors.rs):
    -   **Purpose**: Represents the set of mathematical multipliers (coding factors) used to create a linear combination of fragments. 
    -   **Usage**: Encodes which fragments are included in a specific `Frame` and with what weight.

-   **`Frame`** @ [crates/qr_comm/src/data_structures/frame.rs](../crates/qr_comm/src/data_structures/frame.rs):
    -   **Purpose**: The wire format payload for a single QR Code.
    -   **Structure**: Consists of `FrameFactor`s (the coefficients), a `Fragment` (the mathematically combined payload), and a `FrameHeader` (metadata about the sender).
    -   **Usage**: This is the data structure that directly converts to and from a visual `qr_code::QrCode`.

-   **`Equation`** @ [crates/qr_comm/src/network_coding/equation.rs](../crates/qr_comm/src/network_coding/equation.rs):
    -   **Purpose**: Represents a row in the decoding matrix. Internally pairs a `Factor` (abstract representation of `FrameFactor`) with a `Fragment`.
    -   **Usage**: Supports core math operations (Add, Sub, Mul, Div) in the Galois field for Gaussian elimination.

-   **`GaloisField2p4`** @ [crates/qr_comm/src/network_coding/galois.rs](../crates/qr_comm/src/network_coding/galois.rs):
    -   **Purpose**: Arithmetic constraint module ($GF(2^4)$). Ensures that all additions and multiplications of bytes wrap logically and stay within fixed bounds, keeping combinations fitting neatly into bytes.

-   **`Epoch`** @ [crates/qr_comm/src/network_coding/epoch.rs](../crates/qr_comm/src/network_coding/epoch.rs):
    -   **Purpose**: The core engine managing the transmission and reception state for a single session.
    -   **Usage**: 
        -   **Transmitter**: Receives `Package`s via `Epoch::write()`. Calling `Epoch::pop_recent_frame()` randomly generates network-coded `Frame`s based on the written fragments.
        -   **Receiver**: Receives `Frame`s scanned by the camera via `Epoch::push_frame()`. It manages a `Matrix` of `Equation`s to perform Gaussian elimination. Once it achieves "full rank," it decodes `Fragment`s and stores them. Finished data is requested back via `Epoch::get_package()`.

## Sending Data (Transmitting)

The process of generating and displaying a QR code stream:

1.  **Preparation**: The application data (e.g., a file or a string message) is encapsulated into a `Package`.
2.  **Writing to Epoch**: The sender invokes `Epoch::write(package)`. Internally, the `Package` is split into chunks creating an array of `Fragment`s.
3.  **Frame Generation Cycle**: At a fixed interval (e.g., ~20Hz), the transmitter calls `Epoch::pop_recent_frame()`.
    -   The `Epoch` takes a subset of the active `Fragment`s.
    -   It assigns random scalar coefficients (`GaloisField2p4`) to them.
    -   It combines these fragments mathematically into a single output `Fragment` payload.
    -   It constructs a `Frame` containing the combined payload and the coefficients used.
4.  **Displaying**: The resulting `Frame` is serialized and passed into the `qrcode` crate to generate an image. This image is rendered to the UI texture, creating a rapid, continuous QR video stream. 

## Receiving Data (Assembling)

The process of capturing the QR code stream and decoding it:

1.  **Scanning**: The receiving device uses the `qr_scanner::QrScannerPopup` to capture frames from the camera using the browser's `mediaDevices` API.
2.  **QR Recognition**: Pixel data is passed to the `rqrr` library to locate and decode QR code matrices periodically.
3.  **Frame Conversion**: The decoded raw bytes are unpacked back into a `Frame` struct, containing the coding coefficients and the mixed data fragment.
4.  **Matrix Push**: The receiver calls `Epoch::push_frame(frame)`.
    -   The `Frame` is converted into an `Equation`.
    -   The `Epoch` inserts this `Equation` into its internal `Matrix` and attempts Gaussian elimination.
    -   If the equation provides linearly independent (novel) information, it is kept; otherwise, it resolves to zero and is discarded.
5.  **Reconstruction**: The UI continuously monitors the rank of the matrix (# of useful equations) compared to the required number of fragments.
6.  **Package Extraction**: Once full rank is achieved (Gaussian elimination produces plain, single-variable equations), individual `Fragment`s are reconstructed. The frontend retrieves the full files by repeatedly calling `Epoch::get_package(participant, index)`.

## Usage Implementation

The frontend has been modified to support this "Test Mode". You can find the respective integrations in these components.

### Scanning (`frontend/src/qr_scanner.rs`)
-   Uses `web-sys` to access `navigator.mediaDevices.getUserMedia`.
-   Draws video frames to a `canvas`.
-   Reads pixel data from canvas and passes it to the `rqrr` library (Rust QR reader).
-   **Performance**: Scanning happens periodically (e.g., every 5th frame) to avoid blocking the UI thread.

### Receiving (`frontend/src/game/screens/qr_test_receive.rs`)
-   Displays a "Scan QR" popup.
-   When a QR is detected:
    1.  Raw bytes are extracted and converted to a `Frame`.
    2.  Pushed to the application's global `Epoch` instance.
    3.  UI shows "Rank" (number of useful equations) vs "Needed".
    4.  When full rank is achieved, the message is requested and displayed or saved.

### Transmitting (`frontend/src/game/screens/qr_test_transmit.rs`)
-   Requests data (e.g., a file) from the backend via HTTP or an ad-hoc request mechanism, or accepts plain text input.
-   Pushes the data to the sender's `Epoch` via `write`.
-   Continuously renders the `Frame` produced by `pop_recent_frame()` into an `egui::Image` overlayed on screen.

