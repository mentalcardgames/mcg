//! Player representation and management.

use mcg_shared::PlayerPublic;

#[derive(Clone, Debug)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub stack: u32,
    pub cards: [u8; 2],
    pub has_folded: bool,
    pub all_in: bool,
}

impl Player {
    pub fn public(&self) -> PlayerPublic {
        PlayerPublic {
            id: self.id,
            name: self.name.clone(),
            stack: self.stack,
            cards: None, // Private by default
            has_folded: self.has_folded,
        }
    }
}