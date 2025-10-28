use mcg_qr_comm::data_structures::Frame;
use mcg_qr_comm::FRAME_SIZE_BYTES;
use mcg_qr_comm::network_coding::Epoch;
use super::{AppInterface, ScreenDef, ScreenMetadata, ScreenWidget};
use crate::qr_scanner::QrScannerPopup;

pub struct QrTestReceive {
    frame_buffer: Vec<u8>,
    epoch: Epoch,
    scanner: QrScannerPopup,
}

impl QrTestReceive {
    pub fn new() -> Self {
        Self {
            epoch: Epoch::default(),
            frame_buffer: Vec::new(),
            scanner: QrScannerPopup::new(),
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
        self.scanner.button_and_popup(ui, &ctx, &mut String::new(), &mut self.frame_buffer);
        ui.add_space(8.0);
        ui.label("Tip: Click 'Scan QR' to scan a QR code.");

        if self.frame_buffer.len() == FRAME_SIZE_BYTES {
            let frame_data: Result<Box<[u8; FRAME_SIZE_BYTES]>, _> = vec![0; FRAME_SIZE_BYTES].try_into();
            if let Ok(mut frame_data) = frame_data {
                frame_data.copy_from_slice(&self.frame_buffer);
                std::mem::take(&mut self.frame_buffer);
                let frame: Frame = (*frame_data).into();
                self.epoch.push_frame(frame);
            }
        } else if !self.frame_buffer.is_empty() {
            std::mem::take(&mut self.frame_buffer);
        }

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.label("Received Frames:");
            ui.label(&self.epoch.equations.len().to_string());
        });
        ui.label("Decoded fragments per party:");
        ui.horizontal(|ui| {
            for (idx, frags) in self.epoch.decoded_fragments.iter().enumerate() {
                ui.label(format!("Party {}: {:?}", idx, frags.len()));
            }
        });

        if let Some(ap) = self.epoch.get_package(0, 0) {
            if let Ok(s) = String::from_utf8(ap.data) {
                ui.label(format!("AP of party 0:\t{}", s));
            }
        }
    }
}

impl ScreenDef for QrTestReceive {
    fn metadata() -> ScreenMetadata
    where
        Self: Sized,
    {
        ScreenMetadata {
            path: "/receive",
            display_name: "Scan QR-Codes",
            icon: "🔍",
            description: "Receive QR-Codes from peers",
            show_in_menu: true,
        }
    }

    fn create() -> Box<dyn ScreenWidget>
    where
        Self: Sized,
    {
        Box::new(Self::new())
    }
}
