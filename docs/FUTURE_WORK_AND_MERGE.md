# Future Work & Merge Strategy

This document outlines the impact of the `qr_comm` integration, the strategy for merging it into `main`, and potential future applications.

## Impact Analysis

The changes in `qr_comm_test_incorporation` affect the codebase as follows:

| Area | Impact | Description |
|:---|:---|:---|
| **Dependencies** | **High** | Adds `rqrr`, `image`, `qrcode` to Frontend. These are heavy dependencies that increase WASM binary size. |
| **Architectural** | **Medium** | Introduces a Camera/Media layer (`qr_scanner.rs`). Currently isolates logic well, but needs to be managed for cross-platform compatibility (native vs web). |
| **Security** | **Critical** | The testing mechanism `QrReq` allows reading files from the server's `media/` directory. This **MUST** be removed or strictly sandboxed before production use. |
| **UI/UX** | **Low** | Currently adds isolated "Test" screens. No impact on main game flow yet. |

## Follow-up Projects (from Report)

The following areas have been identified as key next steps to transition from the current prototype to a fully functional P2P Mental Card Game.

### 1. QR Scanning Performance
**Problem**: The current scanner is I/O bound and uses hardcoded post-processing values.
**Solution**: 
-   **Profiling**: Identify if bottlenecks are in frame fetching (WASM->JS) or decoding.
-   **Dynamic Tuning**: Adjust image processing parameters at runtime based on lighting/camera.
-   **Native Interop**: Investigate using platform-native scanners (Android SDK) where possible.

### 2. Node Pairing & Discovery
**Problem**: No lobby system exists; nodes cannot easily find each other.
**Proposed Architecture**:
-   **DHT (Distributed Hash Table)**: Use Iroh's discovery mechanisms to publish node presence.
-   **Metadata Exchange**: Nodes should share supported transports and player names.
-   **QR Bootstrapping**: Use the QR channel to exchange initial keys/addresses to "enter" the DHT network.

### 3. "Make it Mental" (Cryptography)
**Problem**: The current poker engine is "naive" and trusts the backend. P2P poker requires Zero-Knowledge Proofs (ZKPs).
**Tasks**:
-   Implement cryptographic primitives for **Shuffling** and **Drawing** without a trusted dealer.
-   Integrate these proofs into the `native_mcg` backend logic.

### 4. Security & Trust
**Problem**: Nodes accept any `ClientMsg` from any source. 
**Solution**:
-   **Message Signing**: All messages must be signed by the player's private key.
-   **Emoji Hash**: Wire up the existing UI to allow users to visually verify public keys (Man-in-the-Middle protection).

### 5. Egui Ergonomics
**Problem**: Mobile experience (Android) brings challenges like on-screen keyboards and touch interactions.
**Task**: Research upstreaming improvements to `egui` or creating better wrappers for mobile text input.

## Workload Estimation

| Task | Effort | Complexity | Description |
|:---|:---|:---|:---|
| **Merge QR to Main** | Low | Low | Resolve conflicts, clean up `QrReq`. |
| **Node Pairing (DHT)** | High | High | core P2P networking work. |
| **ZK Proofs** | Very High | Very High | Requires deep crypto knowledge and performance optimization. |
| **Egui Mobile Polish** | Medium | Medium | annoying but necessary for usability. |
