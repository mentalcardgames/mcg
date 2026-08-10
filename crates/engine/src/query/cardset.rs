use super::Evaluator;
use crate::error::EngineError;
use crate::game_data::{GameData, MemoryValue};
use front_end::ast::{
    AggregateFilter, CardPosition, CardSet, Extrema, FilterExpr, Group, Groupable,
    QueryCardPosition,
};

impl Evaluator {
    pub fn eval_cardset(
        expr: &CardSet,
        game_data: &GameData,
    ) -> Result<(usize, Vec<usize>), EngineError> {
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
                    let owned_loc = Self::find_owned_location(&owner_name, name, game_data).ok_or(
                        EngineError::LocationNotFoundForOwner {
                            name: name.to_string(),
                            owner: owner_name.clone(),
                        },
                    )?;
                    return Ok((owned_loc, game_data.locations[owned_loc].cards.clone()));
                }
                // For filtered groups (where/combo/not-combo) whose base
                // groupable is a plain location, resolve the base location
                // against the *owner* first, then apply the filter. Without
                // this, `Hand of P:P2 where Rank is "Ace"` would evaluate the
                // where-clause against the *current* player's hand (the bare
                // name `Hand` resolves to the current player first) and only
                // then keep cards owned by P2 — usually an empty result.
                if let Some((base_idx, base_cards)) =
                    Self::owner_base_location(group, &owner_name, game_data)
                {
                    let (loc_idx, card_ids) = match group {
                        Group::Where { filter, .. } => (
                            base_idx,
                            Self::apply_filter(filter, &base_cards, game_data)?,
                        ),
                        Group::Combo { combo, .. } => {
                            let combo_filter = game_data
                                .combos
                                .iter()
                                .find(|c| c.name == *combo)
                                .map(|c| c.filter.clone())
                                .ok_or(EngineError::ComboNotFound {
                                    name: combo.clone(),
                                })?;
                            (
                                base_idx,
                                Self::apply_filter(&combo_filter, &base_cards, game_data)?,
                            )
                        }
                        Group::NotCombo { combo, .. } => {
                            let combo_filter = game_data
                                .combos
                                .iter()
                                .find(|c| c.name == *combo)
                                .map(|c| c.filter.clone())
                                .ok_or(EngineError::ComboNotFound {
                                    name: combo.clone(),
                                })?;
                            let matched =
                                Self::apply_filter(&combo_filter, &base_cards, game_data)?;
                            let filtered: Vec<usize> = base_cards
                                .into_iter()
                                .filter(|id| !matched.contains(id))
                                .collect();
                            (base_idx, filtered)
                        }
                        _ => (base_idx, base_cards),
                    };
                    return Ok((loc_idx, card_ids));
                }
                // For filtered groups / combos / card positions without an
                // owner, keep the existing filter-by-owner logic.
                let (loc_idx, card_ids) = Self::eval_group(group, game_data)?;
                let owner_idx = game_data
                    .players
                    .iter()
                    .position(|p| p.name == owner_name)
                    .unwrap_or(usize::MAX);
                let filtered: Vec<usize> = if owner_idx == usize::MAX {
                    // The owner is a team (or unknown) — ownership cannot be
                    // verified per player; keep the evaluated cards.
                    card_ids.clone()
                } else {
                    card_ids
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
                        .collect()
                };
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
                    Some(_) => Err(EngineError::MemoryNotCardSet),
                    None => Err(EngineError::MemoryNotFound { key }),
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
    /// `owner_name` (a player name, a team name, or `"Table"`). Used to
    /// resolve a dest-qualified `GroupOwner` to the owner's own location
    /// rather than the first name match.
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
                if let Some(team) = game_data.teams.iter().find(|t| t.name == owner_name) {
                    return team.owner.locations.contains(idx);
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

    /// Resolve the base location of a structured group (`Where`/`Combo`/
    /// `NotCombo`) whose groupable is a plain location, against a specific
    /// owner. Returns `None` for any other group shape, in which case the
    /// caller falls back to the legacy current-player-relative evaluation.
    fn owner_base_location(
        group: &Group,
        owner_name: &str,
        game_data: &GameData,
    ) -> Option<(usize, Vec<usize>)> {
        let groupable = match group {
            Group::Where { groupable, .. }
            | Group::Combo { groupable, .. }
            | Group::NotCombo { groupable, .. } => groupable,
            _ => return None,
        };
        let name = match groupable {
            Groupable::Location { name } => name,
            _ => return None,
        };
        let loc_idx = Self::find_owned_location(owner_name, name, game_data)?;
        Some((loc_idx, game_data.locations[loc_idx].cards.clone()))
    }

    /// Bare-name resolution: the current player's own location, then the
    /// current player's **team** location, then the Table's, then the first
    /// location with that name anywhere.
    fn resolve_location_by_name(name: &str, game_data: &GameData) -> Option<usize> {
        if let Some(current) = game_data.get_current_player() {
            if let Some(idx) = Self::find_owned_location(&current.name, name, game_data) {
                return Some(idx);
            }
            if let Some(player_idx) = game_data
                .current_player
                .and_then(|pos| game_data.turn_order.get(pos).copied())
            {
                for team in &game_data.teams {
                    if team.players.contains(&player_idx) {
                        if let Some(idx) = Self::find_owned_location(&team.name, name, game_data) {
                            return Some(idx);
                        }
                    }
                }
            }
        }
        if let Some(idx) = Self::find_owned_location("Table", name, game_data) {
            return Some(idx);
        }
        game_data.locations.iter().position(|l| l.name == name)
    }

    fn eval_group(group: &Group, game_data: &GameData) -> Result<(usize, Vec<usize>), EngineError> {
        match group {
            Group::Groupable { groupable } => Self::eval_groupable(groupable, game_data),
            Group::Where { groupable, filter } => {
                let (base_idx, card_ids) = Self::eval_groupable(groupable, game_data)?;
                let filtered = Self::apply_filter(filter, &card_ids, game_data)?;
                // An empty filter result must not degrade to the location-0
                // sentinel (I-14): report the base location of the groupable
                // instead, so a `where`-set used as a move destination still
                // resolves to the pile it filters (engine-vs-design.md D-11).
                let location_idx = if filtered.is_empty() {
                    base_idx
                } else {
                    Self::infer_location_from_cards(&filtered, game_data)?
                };
                Ok((location_idx, filtered))
            }
            Group::NotCombo { combo, groupable } => {
                let (loc_idx, card_ids) = Self::eval_groupable(groupable, game_data)?;
                let combo_filter = game_data
                    .combos
                    .iter()
                    .find(|c| c.name == *combo)
                    .map(|c| c.filter.clone())
                    .ok_or(EngineError::ComboNotFound {
                        name: combo.clone(),
                    })?;
                let matched = Self::apply_filter(&combo_filter, &card_ids, game_data)?;
                let filtered: Vec<usize> = card_ids
                    .into_iter()
                    .filter(|id| !matched.contains(id))
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
                    .ok_or(EngineError::ComboNotFound {
                        name: combo.clone(),
                    })?;
                // Combo filters are evaluated group-wise (like `where`), not
                // per-card: `same Rank`/`distinct Suit`/`size` all need the
                // group context (engine-vs-design.md D-5).
                let filtered = Self::apply_filter(&combo_filter, &card_ids, game_data)?;
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
                        Err(EngineError::LocationNotFoundForCardPosition {
                            name: name.to_string(),
                        })
                    }
                } else {
                    let card_id = Self::eval_card_position(card_position, game_data)?;
                    match game_data.find_location_of_card(card_id) {
                        Some(loc_idx) => Ok((loc_idx, vec![card_id])),
                        None => Err(EngineError::CardPositionNotFound),
                    }
                }
            }
        }
    }

    fn eval_groupable(
        groupable: &Groupable,
        game_data: &GameData,
    ) -> Result<(usize, Vec<usize>), EngineError> {
        match groupable {
            Groupable::Location { name } => {
                let loc_idx = Self::resolve_location_by_name(name, game_data)
                    .ok_or(EngineError::LocationNotFound { name: name.clone() })?;
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

    /// Public wrapper over the private `apply_filter`: returns the subset of
    /// `card_ids` that satisfies `filter`. Used by the combo-source quantifier
    /// to validate a player's chosen set on resume.
    pub fn filter_card_ids(
        filter: &FilterExpr,
        card_ids: &[usize],
        game_data: &GameData,
    ) -> Result<Vec<usize>, EngineError> {
        Self::apply_filter(filter, card_ids, game_data)
    }

    fn apply_filter(
        filter: &FilterExpr,
        card_ids: &[usize],
        game_data: &GameData,
    ) -> Result<Vec<usize>, EngineError> {
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
                        .ok_or(EngineError::PrecedenceNotFound {
                            name: precedence.clone(),
                        })?;
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
                        .ok_or(EngineError::PrecedenceNotFound {
                            name: precedence.clone(),
                        })?;
                    let target_value = Self::eval_string(value, game_data)?;
                    let target_idx = prec.values.iter().position(|v| v == &target_value).ok_or(
                        EngineError::ValueNotFoundInPrecedence {
                            value: target_value,
                            precedence: precedence.clone(),
                        },
                    )?;
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
                        .ok_or(EngineError::PrecedenceNotFound {
                            name: precedence.clone(),
                        })?;
                    let target_value = Self::eval_string(value, game_data)?;
                    let target_idx = prec.values.iter().position(|v| v == &target_value).ok_or(
                        EngineError::ValueNotFoundInPrecedence {
                            value: target_value,
                            precedence: precedence.clone(),
                        },
                    )?;
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
                        .ok_or(EngineError::ComboNotFound {
                            name: combo.clone(),
                        })?;
                    // Group-wise, like `where` (D-5): the combo's filter is
                    // applied to the current set, not per card.
                    Self::apply_filter(&combo_filter, card_ids, game_data)
                }
                AggregateFilter::NotCombo { combo } => {
                    let combo_filter = game_data
                        .combos
                        .iter()
                        .find(|c| c.name == *combo)
                        .map(|c| c.filter.clone())
                        .ok_or(EngineError::ComboNotFound {
                            name: combo.clone(),
                        })?;
                    let matched = Self::apply_filter(&combo_filter, card_ids, game_data)?;
                    let result: Vec<usize> = card_ids
                        .iter()
                        .copied()
                        .filter(|id| !matched.contains(id))
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

    /// Best-effort location inference for a *non-empty* card set: the first
    /// location containing all of them, else the location of the first card,
    /// else the location-0 sentinel (I-14). `eval_group` avoids calling this
    /// with an empty set (D-11: it reports the base location instead).
    fn infer_location_from_cards(
        card_ids: &[usize],
        game_data: &GameData,
    ) -> Result<usize, EngineError> {
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

    pub fn eval_card_position(
        expr: &CardPosition,
        game_data: &GameData,
    ) -> Result<usize, EngineError> {
        match expr {
            CardPosition::Query { query } => match query {
                QueryCardPosition::At { location, int_expr } => {
                    let loc_idx = Self::resolve_location_by_name(location, game_data).ok_or(
                        EngineError::LocationNotFound {
                            name: location.clone(),
                        },
                    )?;
                    let idx = Self::eval_int(int_expr, game_data)? as usize;
                    let card_id = *game_data
                        .locations
                        .get(loc_idx)
                        .and_then(|l| l.cards.get(idx))
                        .ok_or(EngineError::CardAtIndexNotFound {
                            idx,
                            location: location.clone(),
                        })?;
                    Ok(card_id)
                }
                QueryCardPosition::Top { location } => {
                    let loc_idx = Self::resolve_location_by_name(location, game_data).ok_or(
                        EngineError::LocationNotFound {
                            name: location.clone(),
                        },
                    )?;
                    let card_id = *game_data
                        .locations
                        .get(loc_idx)
                        .and_then(|l| l.cards.first())
                        .ok_or(EngineError::CardAtTopNotFound {
                            location: location.clone(),
                        })?;
                    Ok(card_id)
                }
                QueryCardPosition::Bottom { location } => {
                    let loc_idx = Self::resolve_location_by_name(location, game_data).ok_or(
                        EngineError::LocationNotFound {
                            name: location.clone(),
                        },
                    )?;
                    let card_id = *game_data
                        .locations
                        .get(loc_idx)
                        .and_then(|l| l.cards.last())
                        .ok_or(EngineError::CardAtBottomNotFound {
                            location: location.clone(),
                        })?;
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
                        .ok_or(EngineError::PointMapNotFound {
                            name: pointmap.clone(),
                        })?;
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
                    best_card_id.ok_or(EngineError::NoCardForExtremaPointMap)
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
                        .ok_or(EngineError::PrecedenceNotFound {
                            name: precedence.clone(),
                        })?;
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
                    best_card_id.ok_or(EngineError::NoCardForExtremaPrecedence)
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
