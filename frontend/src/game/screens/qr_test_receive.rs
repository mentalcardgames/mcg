use super::{AppInterface, ScreenDef, ScreenMetadata, ScreenWidget};
use crate::qr_scanner::QrScannerPopup;
use egui::{TextEdit, TextStyle};
use mcg_qr_comm::data_structures::Frame;
use mcg_qr_comm::network_coding::Epoch;
use mcg_qr_comm::FRAME_SIZE_BYTES;

pub struct QrTestReceive {
    frame_buffer: Vec<u8>,
    epoch: Epoch,
    scanner: QrScannerPopup,
    matrix: String,
}

impl QrTestReceive {
    pub fn new() -> Self {
        Self {
            epoch: Epoch::default(),
            frame_buffer: Vec::new(),
            scanner: QrScannerPopup::new(),
            matrix: String::new(),
        }
    }
}

impl Default for QrTestReceive {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for QrTestReceive {
    fn ui(
        &mut self,
        _app_interface: &mut AppInterface,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        let ctx = ui.ctx().clone();
        ui.heading("QR Scanner Demo");
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            self.scanner
                .button_and_popup(ui, &ctx, &mut String::new(), &mut self.frame_buffer);
            if ui.button("Sweep upwards").clicked() {
                self.epoch.matrix.sweep_upwards();
            }
        });
        ui.add_space(8.0);
        ui.label("Tip: Click 'Scan QR' to scan a QR code.");

        if self.frame_buffer.len() == FRAME_SIZE_BYTES {
            let frame_data: Result<Box<[u8; FRAME_SIZE_BYTES]>, _> =
                vec![0; FRAME_SIZE_BYTES].try_into();
            if let Ok(mut frame_data) = frame_data {
                frame_data.copy_from_slice(&self.frame_buffer);
                std::mem::take(&mut self.frame_buffer);
                let frame: Frame = (*frame_data).into();
                self.epoch.push_frame(frame);
                self.matrix = self.epoch.print_matrix();
            }
        } else if !self.frame_buffer.is_empty() {
            std::mem::take(&mut self.frame_buffer);
        }

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.label("Received Frames:");
            ui.label(self.epoch.equations.len().to_string());
            ui.label("Number equations:");
            ui.label(self.epoch.needed_eqs.to_string());
        });
        ui.label("Decoded fragments per party:");
        ui.horizontal(|ui| {
            for (idx, frags) in self.epoch.decoded_fragments.iter().enumerate() {
                if frags.is_empty() {
                    continue;
                }
                ui.label(format!("Party {}: {:?}", idx, frags));
            }
        });

        let text_edit = TextEdit::multiline(&mut self.matrix)
            .interactive(false)
            .font(TextStyle::Monospace);
        ui.add(text_edit);

        if let Some(ap) = self.epoch.get_package(0, 0) {
            if let Ok(s) = String::from_utf8(ap.data) {
                ui.label(format!("AP of party 0:\t{}", s));
            }
        }
    }
}

crate::impl_screen_def!(
    QrTestReceive,
    "/receive",
    "Scan QR-Codes",
    "🔍",
    "Receive QR-Codes from peers",
    true
);
