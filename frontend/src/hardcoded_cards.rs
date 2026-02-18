use crate::game::card::DirectoryCardType;
use crate::sprintln;
use mcg_shared::CARD_NATURAL_SIZE;

pub const AVAILABLE_THEMES: &[&str] = &["img_cards", "alt_cards"];
pub const DEFAULT_THEME: &str = "img_cards";

const STANDARD_CARDS: &[&str] = &[
    "1_club.png",
    "1_diamond.png",
    "1_heart.png",
    "1_spade.png",
    "2_club.png",
    "2_diamond.png",
    "2_heart.png",
    "2_spade.png",
    "3_club.png",
    "3_diamond.png",
    "3_heart.png",
    "3_spade.png",
    "4_club.png",
    "4_diamond.png",
    "4_heart.png",
    "4_spade.png",
    "5_club.png",
    "5_diamond.png",
    "5_heart.png",
    "5_spade.png",
    "6_club.png",
    "6_diamond.png",
    "6_heart.png",
    "6_spade.png",
    "7_club.png",
    "7_diamond.png",
    "7_heart.png",
    "7_spade.png",
    "8_club.png",
    "8_diamond.png",
    "8_heart.png",
    "8_spade.png",
    "9_club.png",
    "9_diamond.png",
    "9_heart.png",
    "9_spade.png",
    "10_club.png",
    "10_diamond.png",
    "10_heart.png",
    "10_spade.png",
    "11_club.png",
    "11_diamond.png",
    "11_heart.png",
    "11_spade.png",
    "12_club.png",
    "12_diamond.png",
    "12_heart.png",
    "12_spade.png",
    "13_club.png",
    "13_diamond.png",
    "13_heart.png",
    "13_spade.png",
    "card_back.png",
];

const ALT_CARDS: &[&str] = &[
    "card_clubs_1.png",
    "card_clubs_2.png",
    "card_clubs_3.png",
    "card_clubs_4.png",
    "card_clubs_5.png",
    "card_clubs_6.png",
    "card_clubs_7.png",
    "card_clubs_8.png",
    "card_clubs_9.png",
    "card_clubs_10.png",
    "card_clubs_11.png",
    "card_clubs_12.png",
    "card_clubs_13.png",
    "card_diamond_1.png",
    "card_diamond_2.png",
    "card_diamond_3.png",
    "card_diamond_4.png",
    "card_diamond_5.png",
    "card_diamond_6.png",
    "card_diamond_7.png",
    "card_diamond_8.png",
    "card_diamond_9.png",
    "card_diamond_10.png",
    "card_diamond_11.png",
    "card_diamond_12.png",
    "card_diamond_13.png",
    "card_heart_1.png",
    "card_heart_2.png",
    "card_heart_3.png",
    "card_heart_4.png",
    "card_heart_5.png",
    "card_heart_6.png",
    "card_heart_7.png",
    "card_heart_8.png",
    "card_heart_9.png",
    "card_heart_10.png",
    "card_heart_11.png",
    "card_heart_12.png",
    "card_heart_13.png",
    "card_spade_1.png",
    "card_spade_2.png",
    "card_spade_3.png",
    "card_spade_4.png",
    "card_spade_5.png",
    "card_spade_6.png",
    "card_spade_7.png",
    "card_spade_8.png",
    "card_spade_9.png",
    "card_spade_10.png",
    "card_spade_11.png",
    "card_spade_12.png",
    "card_spade_13.png",
    "card_joker.png",
    "card_joker_black.png",
    "card_joker_red.png",
];

pub fn create_deck(theme: &str) -> DirectoryCardType {
    match theme {
        "img_cards" => {
            let path = "img_cards".to_string();
            let img_names: Vec<String> = STANDARD_CARDS.iter().map(|&s| s.to_string()).collect();
            sprintln!("Created standard deck with {} cards", img_names.len());
            let natural_size = CARD_NATURAL_SIZE;
            DirectoryCardType::new(path, img_names, natural_size)
        }
        "alt_cards" => {
            let path = "alt_cards".to_string();
            let img_names: Vec<String> = ALT_CARDS.iter().map(|&s| s.to_string()).collect();
            sprintln!("Created alternative deck with {} cards", img_names.len());
            let natural_size = CARD_NATURAL_SIZE;
            DirectoryCardType::new(path, img_names, natural_size)
        }
        _ => {
            sprintln!("Unknown theme: {}, falling back to default theme", theme);
            create_deck(DEFAULT_THEME)
        }
    }
}

pub fn set_deck_by_theme(card_config: &mut Option<DirectoryCardType>, theme: &str) {
    let deck = create_deck(theme);
    *card_config = Some(deck);
}

pub fn set_hardcoded_deck(card_config: &mut Option<DirectoryCardType>, use_alt_deck: bool) {
    let theme = if use_alt_deck {
        "alt_cards"
    } else {
        "img_cards"
    };
    set_deck_by_theme(card_config, theme);
}
