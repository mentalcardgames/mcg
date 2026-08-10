use super::Evaluator;
use crate::error::EngineError;
use crate::game_data::{Card, GameData, MemoryValue};
use front_end::ast::{QueryString, StringCollection, StringExpr, Types};

impl Evaluator {
    pub fn eval_string(expr: &StringExpr, game_data: &GameData) -> Result<String, EngineError> {
        match expr {
            StringExpr::Literal { value } => Ok(value.clone()),
            StringExpr::Query { query } => match query {
                QueryString::KeyOf { key, card_position } => {
                    let card_id = Self::eval_card_position(card_position, game_data)?;
                    let card = game_data
                        .get_card(card_id)
                        .ok_or(EngineError::CardNotFound { card_id })?;
                    card.get(key).cloned().ok_or(EngineError::CardKeyNotFound {
                        key: key.clone(),
                        card_id,
                    })
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
                        .ok_or(EngineError::StringCollectionAtOutOfRange { idx })
                }
            },
            StringExpr::Memory { memory } => {
                let key = Self::resolve_memory_key(memory, game_data)?;
                match game_data.get_memory(&key) {
                    Some(MemoryValue::String(v)) => Ok(v.clone()),
                    Some(_) => Err(EngineError::MemoryNotString),
                    None => Err(EngineError::MemoryNotFound { key }),
                }
            }
        }
    }

    pub(super) fn eval_string_collection(
        col: &StringCollection,
        game_data: &GameData,
    ) -> Result<Vec<String>, EngineError> {
        match col {
            StringCollection::Literal { strings } => {
                let mut result = vec![];
                for s in strings {
                    result.push(Self::eval_string(s, game_data)?);
                }
                Ok(result)
            }
            // Aggregate a String memory across every owner in `multi`.
            StringCollection::AggregateMemory { memory, multi } => {
                let names = super::Evaluator::resolve_multi_owner_names(multi, game_data)?;
                let mut result = vec![];
                for name in names {
                    let key = format!("{}_{}", name, memory);
                    match game_data.get_memory(&key) {
                        Some(MemoryValue::String(v)) => result.push(v.clone()),
                        Some(_) => {
                            return Err(EngineError::MemoryNotStringFor { key });
                        }
                        None => {
                            return Err(EngineError::MemoryNotFound { key });
                        }
                    }
                }
                Ok(result)
            }
            StringCollection::Memory { memory } => {
                let key = Self::resolve_collection_memory_key(memory, game_data)?;
                match game_data.get_memory(&key) {
                    Some(MemoryValue::StringCollection(v)) => Ok(v.clone()),
                    Some(_) => Err(EngineError::MemoryNotStringCollection),
                    None => Err(EngineError::MemoryNotFound { key }),
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
