---
outline: deep
---

# QR Communication Protocol

This document describes the fountain-code communication channel implemented by
the `qr_comm` crate and its current browser integration in `frontend`.

## Concept: Visual Network Coding

The channel transmits arbitrary binary data between devices using only a screen
as transmitter and a camera as receiver.

QR codes have limited capacity, and camera frames may be dropped because of
focus, tearing, motion, or lighting.
The implementation therefore sends fountain-coded linear combinations instead
of a numbered sequence of chunks.
If a package is split into $N$ fragments, the receiver can reconstruct it from
any sufficiently large set of linearly independent frames rather than requiring
a particular missing frame.

## Core Structs & Modules

The `qr_comm` crate implements framing and network-coding operations:

- **`Package`** @
  [crates/qr_comm/src/data_structures/application_package.rs](https://github.com/mentalcardgames/mcg/blob/main/crates/qr_comm/src/data_structures/application_package.rs)
  wraps arbitrary application data, splits it into uniform `Fragment`s, and can
  reconstruct the original package from decoded fragments.
- **`Fragment`** @
  [crates/qr_comm/src/data_structures/fragment.rs](https://github.com/mentalcardgames/mcg/blob/main/crates/qr_comm/src/data_structures/fragment.rs)
  is the fixed-size unit used by the coding operations.
- **`FrameFactor`** and **`Factor`** @
  [crates/qr_comm/src/data_structures/factors.rs](https://github.com/mentalcardgames/mcg/blob/main/crates/qr_comm/src/data_structures/factors.rs)
  represent the coefficients used to combine fragments.
- **`Frame`** @
  [crates/qr_comm/src/data_structures/frame.rs](https://github.com/mentalcardgames/mcg/blob/main/crates/qr_comm/src/data_structures/frame.rs)
  is the payload encoded by one QR code.
  It contains coding factors, the mixed fragment, and a `FrameHeader`
  identifying the sender/session context.
- **`Equation`** @
  [crates/qr_comm/src/network_coding/equation.rs](https://github.com/mentalcardgames/mcg/blob/main/crates/qr_comm/src/network_coding/equation.rs)
  represents one row of the decoding matrix.
- **`GaloisField2p4`** @
  [crates/qr_comm/src/network_coding/galois.rs](https://github.com/mentalcardgames/mcg/blob/main/crates/qr_comm/src/network_coding/galois.rs)
  implements arithmetic in $GF(2^4)$.
- **`Epoch`** @
  [crates/qr_comm/src/network_coding/epoch.rs](https://github.com/mentalcardgames/mcg/blob/main/crates/qr_comm/src/network_coding/epoch.rs)
  owns transmission or reception state.
  Writers add `Package`s and generate recent frames; receivers push frames,
  maintain the equation matrix, and retrieve reconstructed packages.

## Sending Data

1. Wrap the application data in a `Package`.
2. Call `Epoch::write(package)` to split and store its fragments.
3. Call `Epoch::pop_recent_frame()` repeatedly.
   Each frame combines a subset of fragments with random coefficients.
4. Convert each `Frame` to a `qrcode::QrCode` and render it as an image.
5. Keep generating frames at a controlled interval while transmission is active.

## Receiving Data

1. Capture browser camera frames with `Camera`.
2. Let `QrScanner` locate and decode a QR matrix into binary bytes.
3. Accept the bytes only when their length equals `FRAME_SIZE_BYTES` and
   convert them into a `Frame`.
4. Call `Epoch::push_frame(frame)`.
   The epoch inserts a corresponding `Equation` and performs matrix reduction.
5. Continue collecting linearly independent frames until enough fragments have
   been decoded.
6. Retrieve a reconstructed `Package` with `Epoch::get_package(participant,
   index)`.

## Frontend Integration

The camera, QR decoder, and transmission/reception screens own separate
responsibilities.
Their state is screen-local rather than stored in `FrontendState`.

### Camera (`frontend/src/widgets/camera.rs`)

`Camera` owns the browser `MediaStream`, hidden video/canvas elements, preview
texture, and selected facing mode. It:

- starts `getUserMedia` asynchronously;
- ignores obsolete asynchronous start results after a restart or close;
- captures a frame into an `egui::ColorImage`;
- can switch between user-facing and environment-facing cameras; and
- stops every media track when the camera is closed.

### Scanner (`frontend/src/widgets/qr_scanner.rs`)

`QrScanner` provides the "Scan QR" button and popup.
It displays the current camera texture and attempts decoding every fifth
captured frame to reduce work on the WASM UI thread.

The output is selected explicitly with `QrDecodeTarget`:

```rust
pub enum QrDecodeTarget<'a> {
  String(&'a mut String),
  Binary(&'a mut Vec<u8>),
}

impl QrScanner {
  pub fn button_and_popup(
    &mut self,
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    target: QrDecodeTarget<'_>,
  ) { ... }
}
```

String mode closes the popup after the first successful decode.
Binary mode keeps it open, which allows the receiver to process a stream of
fountain-code frames.
While the popup is open, the scanner requests repaints for live camera capture.

### Receiving (`frontend/src/screens/qr_test_receive.rs`)

`QrTestReceive` owns its `QrScanner`, binary frame buffer, `Epoch`, and matrix
display.
A decoded buffer of exactly `FRAME_SIZE_BYTES` is converted to a `Frame` and
pushed into the local epoch.
Invalid-length buffers are discarded.
The screen displays received equation counts, decoded fragments, the current
matrix, and reconstructed package text.

### Transmitting (`frontend/src/screens/qr_test_transmit.rs`)

`QrTestTransmit` owns its sender `Epoch`, QR image queue, texture,
timing state, and input.
It can write text directly or request package bytes from the backend
using typed frontend/backend messages.
`Backend2FrontendMsg::QrRes` is handled through `ScreenWidget::on_message` and
written into the sender epoch.

While transmission is running, the screen generates frames, queues rendered QR
images, advances them at a controlled interval, and schedules the next repaint
with `request_repaint_after`.
