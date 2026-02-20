# QR Communication Protocol

This document describes the technical implementation of the QR-based communication channel introduced in the `qr_comm_test_incorporation` branch.

## Concept: Visual Network Coding

The goal is to transmit arbitrary binary data (files, game state, crypto keys) between devices using *only* the screen (as transmitter) and camera (as receiver).

**Challenge**: QR codes have limited capacity. A video stream of QR codes can transmit large data, but frames might be dropped (camera loses focus, tearing, lighting issues).
**Solution**: **Fountain Codes (Network Coding)**. Instead of chopping the file into pieces 1, 2, 3... and needing specifically piece #2 if it's lost, we transmit mathematical combinations of the pieces.
-   If the file is split into $N$ fragments.
-   The sender generates infinite random linear combinations (Equation frames).
-   The receiver just needs to collect *any* $N$ (plus a small overhead) linearly independent frames to solve the system and recover the file.

## Implementation Details

### 1. `mcg_qr_comm` Crate
This crate implements the math and data framing.
-   **Galois Field ($GF(2^8)$)**: Arithmetic is done in a finite field to keep numbers fitting in bytes.
-   **Matrix**: Used to store the equations (received frames).
-   **Decoding**: Gaussian elimination is used to solve the matrix.

### 2. Frontend Integration

The frontend has been modified to support this "Test Mode".

#### Scanning (`frontend/src/qr_scanner.rs`)
-   Uses `web-sys` to access `navigator.mediaDevices.getUserMedia`.
-   Draws video frames to a `canvas`.
-   Reads pixel data from canvas and passes it to the `rqrr` library (Rust QR reader).
-   **Performance**: Scanning happens periodically (e.g., every 5th frame) to avoid blocking the UI thread.

#### Receiving (`qr_test_receive.rs`)
-   Displays a "Scan QR" popup.
-   When a QR is detected:
    1.  Raw bytes are extracted.
    2.  Converted to a `Frame`.
    3.  Pushed to `Epoch`.
    4.  `Epoch` updates the matrix state.
    5.  UI shows "Rank" (number of useful equations) vs "Needed".
    6.  When full rank is achieved, the message is decoded.

#### Transmitting (`qr_test_transmit.rs`)
-   Requests data (e.g., a file) from the backend via `QrReq` / `QrRes` (or can accept text input).
-   Chunks data into fragments.
-   Generates `Frame`s on the fly using the random coding factors.
-   Renders `Frame` -> `QrCode` (using `qrcode` crate) -> `Image` -> Texture.
-   Updates the displayed QR code at ~20Hz (configurable).

## Key Components

| Component | Location | Role |
|:---|:---|:---|
| `Epoch` | `crates/qr_comm/src/network_coding/` | Manages the decoding matrix and state for one transmission session. |
| `Frame` | `crates/qr_comm/src/data_structures/` | Wire format of a single QR payload. |
| `QrScannerPopup` | `frontend/src/qr_scanner.rs` | UI Widget that manages the camera stream and decoding loop. |
| `QrTestReceive` | `frontend/src/game/screens/` | Screen for the receiver role. |
| `QrTestTransmit` | `frontend/src/game/screens/` | Screen for the generator role. |
