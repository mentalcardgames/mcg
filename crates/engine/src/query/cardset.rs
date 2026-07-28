use super::Evaluator;
use crate::game_data::{GameData, MemoryValue};
use front_end::ast::{
    AggregateFilter, CardPosition, CardSet, Extrema, FilterExpr, Group, Groupable,
    QueryCardPosition,
};

impl Evaluator {
    pub fn eval_cardset(
        expr: &CardSet,
        game_data: &GameData,
    ) -> Result<(usize, Vec<usize>), String> {
        match expr {
            CardSet::Group { group } => Self::eval_group(group, game_data),
            CardSet::GroupOwner { group, owner } => {
                let owner_name = Self::resolve_owner_to_name(owner, game_data)?;
                // For plain locations with an explicit owner, resolve the
                // owner's specific location directly. The old path of "eval
                // first name match, then filter by owner" fails when
                // multiple players own locations with the same name (e.g.
                // P1:Hand, P2:Hand, P3:Hand).
                if let Some(name) = Self::group_location_name(group) {
                    let owned_loc = Self::find_owned_location(&owner_name, name, game_data)
                        .ok_or_else(|| {
                            format!(
                                "Location {} not found for owner {}",
                                name, owner_name
                            )
                        })?;
                    return Ok((owned_loc, game_data.locations[owned_loc].cards.clone()));
                }
                // For filtered groups / combos / card positions without an
                // owner, keep the existing filter-by-owner logic.
                let (loc_idx, card_ids) = Self::eval_group(group, game_data)?;
                let owner_idx = game_data
                    .players
                    .iter()
                    .position(|p| p.name == owner_name)
                    .unwrap_or(usize::MAX);
                let filtered: Vec<usize> = card_ids
                    .into_iter()
                    .filter(|&card_id| {
                        for (loc_i, loc) in game_data.locations.iter().enumerate() {
                            if loc.cards.contains(&card_id) {
                                if game_data.table.locations.contains(&loc_i) {
                                    return owner_name == "Table";
                                }
                                if game_data.players[owner_idx]
                                    .owner
                                    .locations
                                    .contains(&loc_i)
                                {
                                    return true;
                                }
                            }
                        }
                        false
                    })
                    .collect();
                let dest_loc_idx = match Self::group_location_name(group) {
                    Some(name) => {
                        Self::find_owned_location(&owner_name, name, game_data).unwrap_or(loc_idx)
                    }
                    None => loc_idx,
                };
                Ok((dest_loc_idx, filtered))
            }
            CardSet::Memory { memory } => {
                let key = Self::resolve_collection_memory_key(memory, game_data)?;
                match game_data.get_memory(&key) {
                    Some(MemoryValue::CardSet(card_ids)) => {
                        if let Some(&first_card) = card_ids.first() {
                            if let Some(loc_idx) = game_data.find_location_of_card(first_card) {
                                return Ok((loc_idx, card_ids.clone()));
                            }
                        }
                        Ok((0, card_ids.clone()))
                    }
                    Some(_) => Err("Memory value is not a CardSet".to_string()),
                    None => Err(format!("Memory {} not found", key)),
                }
            }
        }
    }

    /// Best-effort extraction of the location name from a plain
    /// `Group::Groupable { Groupable::Location { name } }`. Returns `None` for
    /// any more elaborate group (filters, combos, collections), in which case
    /// `eval_cardset` falls back to the first name-match location.
    fn group_location_name(group: &Group) -> Option<&str> {
        match group {
            Group::Groupable {
                groupable: Groupable::Location { name },
            } => Some(name),
            _ => None,
        }
    }

    /// Find the index of the location named `loc_name` that is owned by
    /// `owner_name` (a player name or `"Table"`). Used to resolve a
    /// dest-qualified `GroupOwner` to the owner's own location rather than the
    /// first name match.
    fn find_owned_location(
        owner_name: &str,
        loc_name: &str,
        game_data: &GameData,
    ) -> Option<usize> {
        game_data
            .locations
            .iter()
            .enumerate()
            .find(|(idx, loc)| {
                if loc.name != loc_name {
                    return false;
                }
                if owner_name == "Table" {
                    return game_data.table.locations.contains(idx);
                }
                game_data
                    .players
                    .iter()
                    .find(|p| p.name == owner_name)
                    .map(|p| p.owner.locations.contains(idx))
                    .unwrap_or(false)
            })
            .map(|(idx, _)| idx)
    }

    fn resolve_location_by_name(name: &str, game_data: &GameData) -> Option<usize> {
        if let Some(current) = game_data.get_current_player() {
            if let Some(idx) = Self::find_owned_location(&current.name, name, game_data) {
                return Some(idx);
            }
        }
        if let Some(idx) = Self::find_owned_location("Table", name, game_data) {
            return Some(idx);
        }
        game_data.locations.iter().position(|l| l.name == name)
    }

    fn eval_group(group: &Group, game_data: &GameData) -> Result<(usize, Vec<usize>), String> {
        match group {
            Group::Groupable { groupable } => Self::eval_groupable(groupable, game_data),
            Group::Where { groupable, filter } => {
                let (_, card_ids) = Self::eval_groupable(groupable, game_data)?;
                let filtered = Self::apply_filter(filter, &card_ids, game_data)?;
                let location_idx = Self::infer_location_from_cards(&filtered, game_data)?;
                Ok((location_idx, filtered))
            }
            Group::NotCombo { combo, groupable } => {
                let (loc_idx, card_ids) = Self::eval_groupable(groupable, game_data)?;
                let combo_filter = game_data
                    .combos
                    .iter()
                    .find(|c| c.name == *combo)
                    .map(|c| c.filter.clone())
                    .ok_or(format!("Combo {} not found", combo))?;
                let filtered: Vec<usize> = card_ids
                    .into_iter()
                    .filter(|&card_id| {
                        !Self::card_matches_filter(card_id, &combo_filter, game_data)
                    })
                    .collect();
                Ok((loc_idx, filtered))
            }
            Group::Combo { combo, groupable } => {
                let (loc_idx, card_ids) = Self::eval_groupable(groupable, game_data)?;
                let combo_filter = game_data
                    .combos
                    .iter()
                    .find(|c| c.name == *combo)
                    .map(|c| c.filter.clone())
                    .ok_or(format!("Combo {} not found", combo))?;
                let filtered: Vec<usize> = card_ids
                    .into_iter()
                    .filter(|&card_id| Self::card_matches_filter(card_id, &combo_filter, game_data))
                    .collect();
                Ok((loc_idx, filtered))
            }
            Group::CardPosition { card_position } => {
                let loc_name = match card_position {
                    CardPosition::Query { query } => match query {
                        QueryCardPosition::Top { location }
                        | QueryCardPosition::Bottom { location }
                        | QueryCardPosition::At { location, .. } => Some(location.as_str()),
                    },
                    _ => None,
                };

                if let Some(name) = loc_name {
                    if let Some(loc_idx) = Self::resolve_location_by_name(name, game_data) {
                        match Self::eval_card_position(card_position, game_data) {
                            Ok(card_id) => Ok((loc_idx, vec![card_id])),
                            Err(_) => Ok((loc_idx, vec![])),
                        }
                    } else {
                        Err(format!("Location '{}' not found for card position", name))
                    }
                } else {
                    let card_id = Self::eval_card_position(card_position, game_data)?;
                    match game_data.find_location_of_card(card_id) {
                        Some(loc_idx) => Ok((loc_idx, vec![card_id])),
                        None => Err("Card position not found in any location".to_string()),
                    }
                }
            }
        }
    }

    fn eval_groupable(
        groupable: &Groupable,
        game_data: &GameData,
    ) -> Result<(usize, Vec<usize>), String> {
        match groupable {
            Groupable::Location { name } => {
                let loc_idx = Self::resolve_location_by_name(name, game_data)
                    .ok_or_else(|| format!("Location {} not found", name))?;
                let card_ids = game_data
                    .locations
                    .get(loc_idx)
                    .map(|l| l.cards.clone())
                    .unwrap_or_default();
                Ok((loc_idx, card_ids))
            }
            Groupable::LocationCollection {
                location_collection,
            } => {
                let loc_names = Self::eval_location_collection(location_collection, game_data)?;
                let mut all_cards = vec![];
                let mut location_idx = 0;
                for name in &loc_names {
                    if let Some(idx) = Self::resolve_location_by_name(name, game_data) {
                        if location_idx == 0 {
                            location_idx = idx;
                        }
                        if let Some(loc) = game_data.locations.get(idx) {
                            all_cards.extend_from_slice(&loc.cards);
                        }
                    }
                }
                if location_idx == 0 && !loc_names.is_empty() {
                    location_idx =
                        Self::resolve_location_by_name(&loc_names[0], game_data).unwrap_or(0);
                }
                Ok((location_idx, all_cards))
            }
        }
    }

    fn apply_filter(
        filter: &FilterExpr,
        card_ids: &[usize],
        game_data: &GameData,
    ) -> Result<Vec<usize>, String> {
        match filter {
            FilterExpr::Aggregate { aggregate } => match aggregate {
                AggregateFilter::Size { cmp, int_expr } => {
                    let target = Self::eval_int(int_expr, game_data)?;
                    let size = card_ids.len() as i32;
                    if Self::eval_int_compare(size, cmp, target) {
                        Ok(card_ids.to_vec())
                    } else {
                        Ok(vec![])
                    }
                }
                AggregateFilter::Same { key } => {
                    let mut groups: std::collections::HashMap<String, Vec<usize>> =
                        std::collections::HashMap::new();
                    for &card_id in card_ids {
                        if let Some(card) = game_data.get_card(card_id) {
                            if let Some(value) = card.get(key) {
                                groups.entry(value.clone()).or_default().push(card_id);
                            }
                        }
                    }
                    let mut result = vec![];
                    for (_, mut ids) in groups {
                        if ids.len() > 1 {
                            result.append(&mut ids);
                        }
                    }
                    Ok(result)
                }
                AggregateFilter::Distinct { key } => {
                    let mut groups: std::collections::HashMap<String, Vec<usize>> =
                        std::collections::HashMap::new();
                    for &card_id in card_ids {
                        if let Some(card) = game_data.get_card(card_id) {
                            if let Some(value) = card.get(key) {
                                groups.entry(value.clone()).or_default().push(card_id);
                            }
                        }
                    }
                    let mut result = vec![];
                    for (_, mut ids) in groups {
                        if ids.len() == 1 {
                            result.append(&mut ids);
                        }
                    }
                    Ok(result)
                }
                AggregateFilter::Adjacent { key, precedence } => {
                    let prec = game_data
                        .precedences
                        .iter()
                        .find(|p| p.name == *precedence)
                        .ok_or(format!("Precedence {} not found", precedence))?;
                    let mut cards_with_values: Vec<(usize, i32, String)> = vec![];
                    for &card_id in card_ids {
                        if let Some(card) = game_data.get_card(card_id) {
                            if let Some(value) = card.get(key) {
                                if let Some(idx) = prec.values.iter().position(|v| v == value) {
                                    cards_with_values.push((card_id, idx as i32, value.clone()));
                                }
                            }
                        }
                    }
                    cards_with_values.sort_by_key(|&(_, idx, _)| idx);
                    let mut result = vec![];
                    for window in cards_with_values.windows(2) {
                        let (id1, idx1, _) = window[0];
                        let (id2, idx2, _) = window[1];
                        if idx2 - idx1 == 1 {
                            if !result.contains(&id1) {
                                result.push(id1);
                            }
                            if !result.contains(&id2) {
                                result.push(id2);
                            }
                        }
                    }
                    Ok(result)
                }
                AggregateFilter::Higher {
                    key,
                    value,
                    precedence,
                } => {
                    let prec = game_data
                        .precedences
                        .iter()
                        .find(|p| p.name == *precedence)
                        .ok_or(format!("Precedence {} not found", precedence))?;
                    let target_value = Self::eval_string(value, game_data)?;
                    let target_idx =
                        prec.values
                            .iter()
                            .position(|v| v == &target_value)
                            .ok_or(format!(
                                "Value {} not found in precedence {}",
                                target_value, precedence
                            ))?;
                    let mut result = vec![];
                    for &card_id in card_ids {
                        if let Some(card) = game_data.get_card(card_id) {
                            if let Some(card_value) = card.get(key) {
                                if let Some(idx) = prec.values.iter().position(|v| v == card_value)
                                {
                                    if idx as i32 > target_idx as i32 {
                                        result.push(card_id);
                                    }
                                }
                            }
                        }
                    }
                    Ok(result)
                }
                AggregateFilter::Lower {
                    key,
                    value,
                    precedence,
                } => {
                    let prec = game_data
                        .precedences
                        .iter()
                        .find(|p| p.name == *precedence)
                        .ok_or(format!("Precedence {} not found", precedence))?;
                    let target_value = Self::eval_string(value, game_data)?;
                    let target_idx =
                        prec.values
                            .iter()
                            .position(|v| v == &target_value)
                            .ok_or(format!(
                                "Value {} not found in precedence {}",
                                target_value, precedence
                            ))?;
                    let mut result = vec![];
                    for &card_id in card_ids {
                        if let Some(card) = game_data.get_card(card_id) {
                            if let Some(card_value) = card.get(key) {
                                if let Some(idx) = prec.values.iter().position(|v| v == card_value)
                                {
                                    if (idx as i32) < (target_idx as i32) {
                                        result.push(card_id);
                                    }
                                }
                            }
                        }
                    }
                    Ok(result)
                }
                AggregateFilter::KeyIsString { key, string } => {
                    let target = Self::eval_string(string, game_data)?;
                    let result: Vec<usize> = card_ids
                        .iter()
                        .filter(|&&card_id| {
                            game_data
                                .get_card(card_id)
                                .and_then(|c| c.get(key))
                                .map(|v| v == &target)
                                .unwrap_or(false)
                        })
                        .copied()
                        .collect();
                    Ok(result)
                }
                AggregateFilter::KeyIsNotString { key, string } => {
                    let target = Self::eval_string(string, game_data)?;
                    let result: Vec<usize> = card_ids
                        .iter()
                        .filter(|&&card_id| {
                            game_data
                                .get_card(card_id)
                                .and_then(|c| c.get(key))
                                .map(|v| v != &target)
                                .unwrap_or(true)
                        })
                        .copied()
                        .collect();
                    Ok(result)
                }
                AggregateFilter::Combo { combo } => {
                    let combo_filter = game_data
                        .combos
                        .iter()
                        .find(|c| c.name == *combo)
                        .map(|c| c.filter.clone())
                        .ok_or(format!("Combo {} not found", combo))?;
                    let result: Vec<usize> = card_ids
                        .iter()
                        .filter(|&&card_id| {
                            Self::card_matches_filter(card_id, &combo_filter, game_data)
                        })
                        .copied()
                        .collect();
                    Ok(result)
                }
                AggregateFilter::NotCombo { combo } => {
                    let combo_filter = game_data
                        .combos
                        .iter()
                        .find(|c| c.name == *combo)
                        .map(|c| c.filter.clone())
                        .ok_or(format!("Combo {} not found", combo))?;
                    let result: Vec<usize> = card_ids
                        .iter()
                        .filter(|&&card_id| {
                            !Self::card_matches_filter(card_id, &combo_filter, game_data)
                        })
                        .copied()
                        .collect();
                    Ok(result)
                }
            },
            FilterExpr::Binary {
                filter: f1,
                op,
                filter1: f2,
            } => {
                let result1 = Self::apply_filter(f1, card_ids, game_data)?;
                let result2 = Self::apply_filter(f2, card_ids, game_data)?;
                match op {
                    front_end::ast::FilterOp::And => {
                        let combined: Vec<usize> = result1
                            .into_iter()
                            .filter(|id| result2.contains(id))
                            .collect();
                        Ok(combined)
                    }
                    front_end::ast::FilterOp::Or => {
                        let mut combined = result1;
                        for id in result2 {
                            if !combined.contains(&id) {
                                combined.push(id);
                            }
                        }
                        Ok(combined)
                    }
                }
            }
        }
    }

    fn card_matches_filter(card_id: usize, filter: &FilterExpr, game_data: &GameData) -> bool {
        match filter {
            FilterExpr::Aggregate { aggregate } => match aggregate {
                AggregateFilter::Size { cmp, int_expr } => {
                    let cards = vec![card_id];
                    if let Ok(target) = Self::eval_int(int_expr, game_data) {
                        let size = cards.len() as i32;
                        return Self::eval_int_compare(size, cmp, target);
                    }
                    false
                }
                AggregateFilter::Same { key } => {
                    if let Some(card) = game_data.get_card(card_id) {
                        if let Some(value) = card.get(key) {
                            return game_data
                                .cards
                                .iter()
                                .filter(|c| c.get(key) == Some(value))
                                .any(|c| std::ptr::eq(c, card));
                        }
                    }
                    false
                }
                AggregateFilter::Distinct { key } => {
                    if let Some(card) = game_data.get_card(card_id) {
                        if let Some(value) = card.get(key) {
                            return !game_data
                                .cards
                                .iter()
                                .filter(|c| c.get(key) == Some(value))
                                .any(|c| !std::ptr::eq(c, card));
                        }
                    }
                    false
                }
                _ => false,
            },
            FilterExpr::Binary {
                filter: f1,
                op,
                filter1: f2,
            } => {
                let m1 = Self::card_matches_filter(card_id, f1, game_data);
                let m2 = Self::card_matches_filter(card_id, f2, game_data);
                match op {
                    front_end::ast::FilterOp::And => m1 && m2,
                    front_end::ast::FilterOp::Or => m1 || m2,
                }
            }
        }
    }

    fn infer_location_from_cards(
        card_ids: &[usize],
        game_data: &GameData,
    ) -> Result<usize, String> {
        for (loc_idx, loc) in game_data.locations.iter().enumerate() {
            if card_ids.iter().all(|&id| loc.cards.contains(&id)) {
                return Ok(loc_idx);
            }
        }
        if let Some(&first_card) = card_ids.first() {
            for (loc_idx, loc) in game_data.locations.iter().enumerate() {
                if loc.cards.contains(&first_card) {
                    return Ok(loc_idx);
                }
            }
        }
        Ok(0)
    }

    pub fn eval_card_position(expr: &CardPosition, game_data: &GameData) -> Result<usize, String> {
        match expr {
            CardPosition::Query { query } => match query {
                QueryCardPosition::At { location, int_expr } => {
                    let loc_idx = Self::resolve_location_by_name(location, game_data)
                        .ok_or_else(|| format!("Location {} not found", location))?;
                    let idx = Self::eval_int(int_expr, game_data)? as usize;
                    let card_id = *game_data
                        .locations
                        .get(loc_idx)
                        .and_then(|l| l.cards.get(idx))
                        .ok_or(format!("No card at index {} in location {}", idx, location))?;
                    Ok(card_id)
                }
                QueryCardPosition::Top { location } => {
                    let loc_idx = Self::resolve_location_by_name(location, game_data)
                        .ok_or_else(|| format!("Location {} not found", location))?;
                    let card_id = *game_data
                        .locations
                        .get(loc_idx)
                        .and_then(|l| l.cards.first())
                        .ok_or(format!("No card at top of location {}", location))?;
                    Ok(card_id)
                }
                QueryCardPosition::Bottom { location } => {
                    let loc_idx = Self::resolve_location_by_name(location, game_data)
                        .ok_or_else(|| format!("Location {} not found", location))?;
                    let card_id = *game_data
                        .locations
                        .get(loc_idx)
                        .and_then(|l| l.cards.last())
                        .ok_or(format!("No card at bottom of location {}", location))?;
                    Ok(card_id)
                }
            },
            CardPosition::Aggregate { aggregate } => match aggregate {
                front_end::ast::AggregateCardPosition::ExtremaPointMap {
                    extrema,
                    card_set,
                    pointmap,
                } => {
                    let (_, card_ids) = Self::eval_cardset(card_set, game_data)?;
                    let point_map = game_data
                        .point_maps
                        .iter()
                        .find(|pm| pm.name == *pointmap)
                        .ok_or(format!("PointMap {} not found", pointmap))?;
                    let mut best_card_id = None;
                    let mut best_value = None;
                    for &card_id in &card_ids {
                        if let Some(card) = game_data.get_card(card_id) {
                            let mut card_value = 0;
                            for (key, value) in card.iter() {
                                let map_key = format!("{}:{}", key, value);
                                if let Some(&points) = point_map.map.get(&map_key) {
                                    card_value = points;
                                    break;
                                }
                            }
                            match extrema {
                                Extrema::Min => {
                                    if best_value.is_none()
                                        || card_value < *best_value.as_ref().unwrap()
                                    {
                                        best_value = Some(card_value);
                                        best_card_id = Some(card_id);
                                    }
                                }
                                Extrema::Max => {
                                    if best_value.is_none()
                                        || card_value > *best_value.as_ref().unwrap()
                                    {
                                        best_value = Some(card_value);
                                        best_card_id = Some(card_id);
                                    }
                                }
                            }
                        }
                    }
                    best_card_id.ok_or("No card found for ExtremaPointMap".to_string())
                }
                front_end::ast::AggregateCardPosition::ExtremaPrecedence {
                    extrema,
                    card_set,
                    precedence,
                } => {
                    let (_, card_ids) = Self::eval_cardset(card_set, game_data)?;
                    let prec = game_data
                        .precedences
                        .iter()
                        .find(|p| p.name == *precedence)
                        .ok_or(format!("Precedence {} not found", precedence))?;
                    let mut best_card_id = None;
                    let mut best_idx = None;
                    for &card_id in &card_ids {
                        if let Some(card) = game_data.get_card(card_id) {
                            if let Some(value) = card.get(&prec.key) {
                                if let Some(idx) = prec.values.iter().position(|v| v == value) {
                                    match extrema {
                                        Extrema::Min => {
                                            if best_idx.is_none()
                                                || idx < *best_idx.as_ref().unwrap()
                                            {
                                                best_idx = Some(idx);
                                                best_card_id = Some(card_id);
                                            }
                                        }
                                        Extrema::Max => {
                                            if best_idx.is_none()
                                                || idx > *best_idx.as_ref().unwrap()
                                            {
                                                best_idx = Some(idx);
                                                best_card_id = Some(card_id);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    best_card_id.ok_or("No card found for ExtremaPrecedence".to_string())
                }
            },
        }
    }

    pub fn check_attr_value_in_cardset(
        attr_value: &String,
        card_set: &Vec<usize>,
        game_data: &GameData,
    ) -> bool {
        // This function checks if any card in the card set has an attribute value equal to the string.
        // For example, if the string is "Hearts" and the card set contains the indices of some cards, this function checks if any of those cards has the suit "Hearts".
        // This is used for the StringInCardSet and StringNotInCardSet aggregate bools.

        for card_id in card_set {
            if let Some(card) = game_data.get_card(*card_id) {
                if card.values().any(|v| v == attr_value) {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
#[path = "cardset_tests.rs"]
mod tests;
