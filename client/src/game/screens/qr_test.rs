use super::{AppInterface, ScreenWidget};
use crate::qr_scanner::QrScannerPopup;

pub struct QrScreen {
    input: String,
    scanner: QrScannerPopup,
}

impl QrScreen {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            scanner: QrScannerPopup::new(),
        }
    }
}

impl Default for QrScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for QrScreen {
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
            ui.label("Text:");
            ui.text_edit_singleline(&mut self.input);
            self.scanner.button_and_popup(ui, &ctx, &mut self.input);
        });
        ui.add_space(8.0);
        ui.label("Tip: Click 'Scan QR' to fill this field from a QR code.");
    }
}
