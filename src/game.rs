use egui::Context;
use std::cell::RefCell;
use std::collections::{hash_map, HashMap};
use std::rc::Rc;
pub mod card;
pub mod field;
pub mod screens;
pub mod screen; // Keep for backward compatibility
use screens::{MainMenu, ScreenWidget, ScreenType};

pub struct App {
    // Collection of screens indexed by their type
    screens: HashMap<ScreenType, Rc<RefCell<dyn ScreenWidget>>>,
    default_screen: Rc<RefCell<dyn ScreenWidget>>,
    current_screen: Rc<RefCell<ScreenType>>,
}

impl Default for App {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_type_implementation() {
        let mut app = App::new(None);
        
        // Test initial state
        assert_eq!(app.current_screen(), ScreenType::Main);
        assert!(app.is_current_screen(ScreenType::Main));
        assert!(!app.is_current_screen(ScreenType::Game));
        
        // Test screen switching
        app.set_screen(ScreenType::Game);
        assert_eq!(app.current_screen(), ScreenType::Game);
        assert!(app.is_current_screen(ScreenType::Game));
        assert!(!app.is_current_screen(ScreenType::Main));
        
        // Test screen registration
        let test_screen = Rc::new(RefCell::new(MainMenu::new()));
        assert!(app.register_screen(ScreenType::Settings, test_screen).is_ok());
        assert!(app.has_screen(ScreenType::Settings));
        assert!(!app.has_screen(ScreenType::Pairing));
        
        // Test duplicate registration fails
        let another_screen = Rc::new(RefCell::new(MainMenu::new()));
        assert!(app.register_screen(ScreenType::Settings, another_screen).is_err());
    }

    #[test]
    fn test_screen_type_conversion() {
        // Test all screen type conversions
        let test_cases = vec![
            (ScreenType::Main, "main"),
            (ScreenType::GameSetup, "game_setup"),
            (ScreenType::Game, "game"),
            (ScreenType::Pairing, "pairing"),
            (ScreenType::Settings, "settings"),
            (ScreenType::DndTest, "dnd_test"),
            (ScreenType::GameDndSetup, "game_dnd_setup"),
            (ScreenType::GameDnd, "game_dnd"),
        ];

        for (screen_type, expected_string) in test_cases {
            assert_eq!(screen_type.to_string(), expected_string);
            assert_eq!(ScreenType::from_string(expected_string), Some(screen_type));
        }

        // Test invalid string conversion
        assert_eq!(ScreenType::from_string("invalid_screen"), None);
    }
}

impl App {
    #[allow(unused)]
    pub fn new(main_screen: Option<Rc<RefCell<dyn ScreenWidget>>>) -> Self {
        let default_screen = main_screen.unwrap_or_else(|| Rc::new(RefCell::new(MainMenu::new())));
        let current_screen = Rc::new(RefCell::new(ScreenType::Main));
        let mut screens = HashMap::new();
        screens.insert(ScreenType::Main, Rc::clone(&default_screen));
        Self {
            screens,
            default_screen,
            current_screen,
        }
    }
    #[allow(clippy::result_unit_err)]
    pub fn register_screen(
        &mut self,
        screen_type: ScreenType,
        screen: Rc<RefCell<dyn ScreenWidget>>,
    ) -> Result<(), ()> {
        if let hash_map::Entry::Vacant(e) = self.screens.entry(screen_type) {
            e.insert(screen);
            Ok(())
        } else {
            Err(())
        }
    }
    
    /// Set the current screen to display
    pub fn set_screen(&mut self, screen_type: ScreenType) {
        *self.current_screen.borrow_mut() = screen_type;
    }
    
    /// Get the current screen type
    pub fn current_screen(&self) -> ScreenType {
        self.current_screen.borrow().clone()
    }

    /// Create a screen switcher that can be used by screens to change the current screen
    pub fn screen_switcher(&self) -> impl Fn(ScreenType) {
        let current_screen = Rc::clone(&self.current_screen);
        move |screen_type: ScreenType| {
            *current_screen.borrow_mut() = screen_type;
        }
    }

    /// Check if a specific screen is currently active
    pub fn is_current_screen(&self, screen_type: ScreenType) -> bool {
        *self.current_screen.borrow() == screen_type
    }

    /// Get all registered screen types
    pub fn registered_screens(&self) -> Vec<ScreenType> {
        self.screens.keys().cloned().collect()
    }

    /// Check if a screen is registered
    pub fn has_screen(&self, screen_type: ScreenType) -> bool {
        self.screens.contains_key(&screen_type)
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // Set pixels_per_point to increase DPI scaling (1.5 = 150% scaling)
        ctx.set_pixels_per_point(1.5);
        
        let screen_type = self.current_screen.borrow().clone();
        let current_screen = match self.screens.get(&screen_type) {
            Some(screen) => Rc::clone(screen),
            None => Rc::clone(&self.default_screen),
        };
        
        current_screen.borrow_mut().update(Rc::clone(&self.current_screen), ctx, frame);
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