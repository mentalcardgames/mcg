use super::{AppInterface, ScreenDef, ScreenMetadata, ScreenWidget};
use crate::qr_scanner::QrScannerPopup;

#[derive(Default)]
pub struct QrScreen {
    input: String,
    raw: Vec<u8>,
    scanner: QrScannerPopup,
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
            self.scanner
                .button_and_popup(ui, &ctx, &mut self.input, &mut self.raw);
        });
        ui.add_space(8.0);
        ui.label("Tip: Click 'Scan QR' to fill this field from a QR code.");
        if !self.raw.is_empty() {
            let lossy = String::from_utf8_lossy(&self.raw);
            ui.label(lossy);
        }
    }
}

crate::impl_screen_def!(
    QrScreen,
    "/qr",
    "QR Demo",
    "🔍",
    "Scan QR codes into an input",
    true
);
