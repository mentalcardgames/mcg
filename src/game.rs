use egui::Context;
use std::cell::RefCell;
use std::collections::{hash_map, HashMap};
use std::rc::Rc;
pub mod card;
pub mod field;
pub mod screens;
pub mod screen; // Keep for backward compatibility
use screens::{MainMenu, ScreenWidget};

pub struct App {
    // TODO Make custom struct of this HashMap so that screens can dynamically register other screens
    screens: HashMap<String, Rc<RefCell<dyn ScreenWidget>>>,
    default_screen: Rc<RefCell<dyn ScreenWidget>>,
    current_screen: Rc<RefCell<String>>, // Note: in future consider using ScreenType enum here instead
}

impl Default for App {
    fn default() -> Self {
        Self::new(None)
    }
}

impl App {
    #[allow(unused)]
    pub fn new(main_screen: Option<Rc<RefCell<dyn ScreenWidget>>>) -> Self {
        let default_screen = main_screen.unwrap_or_else(|| Rc::new(RefCell::new(MainMenu::new())));
        let current_screen = Rc::new(RefCell::new(String::from("main")));
        let mut screens = HashMap::new();
        screens.insert(String::from("main"), Rc::clone(&default_screen));
        Self {
            screens,
            default_screen,
            current_screen,
        }
    }
    #[allow(clippy::result_unit_err)]
    pub fn register_screen(
        &mut self,
        name: String,
        screen: Rc<RefCell<dyn ScreenWidget>>,
    ) -> Result<(), ()> {
        if let hash_map::Entry::Vacant(e) = self.screens.entry(name) {
            e.insert(screen);
            Ok(())
        } else {
            Err(())
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // Set pixels_per_point to increase DPI scaling (1.5 = 150% scaling)
        ctx.set_pixels_per_point(1.5);
        
        let current_screen = match self.screens.get_mut(&self.current_screen.borrow().clone()) {
            Some(screen) => Rc::clone(screen),
            None => Rc::clone(&self.default_screen),
        };
        let next_screen = Rc::clone(&self.current_screen);
        current_screen.borrow_mut().update(next_screen, ctx, frame);
        // TODO Create custom screen for text sizes
        // use egui::FontFamily::Proportional;
        // use egui::FontId;
        // use egui::TextStyle::*;
        // use std::collections::BTreeMap;
        // egui::CentralPanel::default().show(ctx, |ui| {
        //     let size = 30.0;
        //     let text_styles: BTreeMap<_, _> = [
        //         (Heading, FontId::new(size, Proportional)),
        //         (Name("Heading2".into()), FontId::new(size, Proportional)),
        //         (Name("Context".into()), FontId::new(size, Proportional)),
        //         (Body, FontId::new(size, Proportional)),
        //         (Monospace, FontId::new(size, Proportional)),
        //         (Button, FontId::new(size, Proportional)),
        //         (Small, FontId::new(size, Proportional)),
        //     ]
        //     .into();
        //     // Mutate global styles with new text styles
        //     ctx.all_styles_mut(move |style| style.text_styles = text_styles.clone());
        // }
    }
}