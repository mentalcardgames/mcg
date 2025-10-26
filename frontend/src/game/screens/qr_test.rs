use super::{AppInterface, ScreenDef, ScreenMetadata, ScreenWidget};
use crate::qr_scanner::QrScannerPopup;

pub struct QrScreen {
    input: String,
    raw: Vec<u8>,
    scanner: QrScannerPopup,
}

impl QrScreen {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            raw: Vec::new(),
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
            self.scanner.button_and_popup(ui, &ctx, &mut self.input, &mut self.raw);
        });
        ui.add_space(8.0);
        ui.label("Tip: Click 'Scan QR' to fill this field from a QR code.");
        if !self.raw.is_empty() {
            let lossy = String::from_utf8_lossy(&self.raw);
            ui.label(lossy);
        }
    }
}

impl ScreenDef for QrScreen {
    fn metadata() -> ScreenMetadata
    where
        Self: Sized,
    {
        ScreenMetadata {
            path: "/qr",
            display_name: "QR Demo",
            icon: "🔍",
            description: "Scan QR codes into an input",
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
