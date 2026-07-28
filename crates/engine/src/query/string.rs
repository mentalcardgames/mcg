use super::Evaluator;
use crate::game_data::{Card, GameData, MemoryValue};
use front_end::ast::{
    QueryString, StringCollection, StringExpr, Types, UseMemory, UseSingleMemory,
};

impl Evaluator {
    pub fn eval_string(expr: &StringExpr, game_data: &GameData) -> Result<String, String> {
        match expr {
            StringExpr::Literal { value } => Ok(value.clone()),
            StringExpr::Query { query } => match query {
                QueryString::KeyOf { key, card_position } => {
                    let card_id = Self::eval_card_position(card_position, game_data)?;
                    let card = game_data
                        .get_card(card_id)
                        .ok_or(format!("Card {} not found", card_id))?;
                    card.get(key)
                        .cloned()
                        .ok_or(format!("Key {} not found in card {}", key, card_id))
                }
                QueryString::StringCollectionAt {
                    string_collection,
                    int_expr,
                } => {
                    let strings = Self::eval_string_collection(string_collection, game_data)?;
                    let idx = Self::eval_int(int_expr, game_data)? as usize;
                    strings
                        .get(idx)
                        .cloned()
                        .ok_or(format!("No string at index {}", idx))
                }
            },
            StringExpr::Memory { memory } => {
                let key = match memory {
                    UseSingleMemory::Memory { memory: m } => m.clone(),
                    UseSingleMemory::WithOwner { memory: m, .. } => m.clone(),
                };
                match game_data.get_memory(&key) {
                    Some(MemoryValue::String(v)) => Ok(v.clone()),
                    Some(_) => Err("Memory value is not a String".to_string()),
                    None => Err(format!("Memory {} not found", key)),
                }
            }
        }
    }

    pub(super) fn eval_string_collection(
        col: &StringCollection,
        game_data: &GameData,
    ) -> Result<Vec<String>, String> {
        match col {
            StringCollection::Literal { strings } => {
                let mut result = vec![];
                for s in strings {
                    result.push(Self::eval_string(s, game_data)?);
                }
                Ok(result)
            }
            StringCollection::AggregateMemory { memory: _, multi } => {
                todo!(
                    "StringCollection::AggregateMemory not yet implemented: {:?}",
                    multi
                )
            }
            StringCollection::Memory { memory } => {
                let key = match memory {
                    UseMemory::Memory { memory: m } => m.clone(),
                    UseMemory::WithOwner { memory: m, .. } => m.clone(),
                };
                match game_data.get_memory(&key) {
                    Some(MemoryValue::StringCollection(v)) => Ok(v.clone()),
                    Some(_) => Err("Memory value is not a StringCollection".to_string()),
                    None => Err(format!("Memory {} not found", key)),
                }
            }
        }
    }

    pub fn expand_types(types: &Types) -> Vec<Card> {
        let mut result = vec![Card::new()];
        for (attr, values) in &types.types {
            let mut new_result = vec![];
            for card in result.clone() {
                for value in values {
                    let mut new_card = card.clone();
                    new_card.insert(attr.clone(), value.clone());
                    new_result.push(new_card);
                }
            }
            result = new_result;
        }
        result
    }
}

#[cfg(test)]
#[path = "string_tests.rs"]
mod tests;
