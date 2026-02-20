use arbitrary::{Arbitrary, Unstructured, Result};
use proptest::prelude::{Strategy, any_with, prop};
use crate::ast::*;

pub fn gen_ident(u: &mut Unstructured) -> arbitrary::Result<String> {
    // Hard fallback: If we are low on entropy, return a known safe string 
    // that is definitely NOT a keyword.
    if u.len() < 5 {
        return Ok("Identfallback".to_string());
    }

    // Attempt to generate a unique, non-keyword identifier
    for _ in 0..10 { // Limit attempts to prevent infinite loops
        let len = u.int_in_range(3..=10)?;
        let capitals = "ABCDEFGHIJKLMNOPQRSTUVXYZ";
        let alphabet = "abcdefghijklmnopqrstuvwxyz";
        let mut s = String::with_capacity(len);
        s.push(*u.choose(capitals.as_bytes())? as char);
        for _ in 0..len {
            s.push(*u.choose(alphabet.as_bytes())? as char);
        }
    }

    // If we failed 10 times to find a non-keyword, 
    // append a suffix to the last generated string to break the keyword match.
    Ok("Id".to_owned() + &u.int_in_range(100..=999)?.to_string())
}

// Helper function to generate a safe alphanumeric identifier
pub fn gen_player_name(u: &mut Unstructured) -> Result<String> {
    Ok(format!("P{}", gen_ident(u)?))
}

// Helper function to generate a safe alphanumeric identifier
pub fn gen_team_name(u: &mut Unstructured) -> Result<String> {
    Ok(format!("T{}", gen_ident(u)?))
}

pub fn gen_vec_players_prefixed(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Vec<String>> {
    // 1. Force 1 to 5 players
    let count = u.int_in_range(1..=5)?;
    let mut vec = Vec::with_capacity(count);

    for _ in 0..count {
        // 2. Generate the "P" prefixed string
        let mut name = gen_player_name(u)?;

        vec.push(name);
    }
    
    Ok(vec)
}

pub fn gen_vec_strings(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Vec<String>> {
    // 1. Force 1 to 5 players
    let count = u.int_in_range(1..=5)?;
    let mut vec = Vec::with_capacity(count);

    for _ in 0..count {
        // 2. Generate the "P" prefixed string
        let mut name = gen_ident(u)?;
        vec.push(name);
    }
    
    Ok(vec)
}

pub fn gen_vec_teams_with_players(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Vec<(String, PlayerCollection)>> {
    // 1. Force 1 to 3 teams
    let count = u.int_in_range(1..=2)?;
    let mut vec = Vec::with_capacity(count);

    for _ in 0..count {
        // 2. Generate the "T" prefixed Team Name
        let name_len = u.int_in_range(1..=2)?;
        let mut team_name = gen_team_name(u)?;
        
        // 3. Generate the PlayerCollection (uses its own Arbitrary impl)
        let players = PlayerCollection::arbitrary(u)?;
        
        vec.push((team_name, players));
    }
    
    Ok(vec)
}

pub fn gen_types_and_subtypes(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Vec<(String, Vec<String>)>> {
    // 1. Force at least 1 outer entry
    let outer_count = u.int_in_range(1..=2)?;
    let mut result = Vec::with_capacity(outer_count);

    for _ in 0..outer_count {
        // 2. Generate the Main Type Name (e.g., "CategoryA")
        let main_name = gen_ident(u)?;

        // 3. Generate the Inner List (Force at least 1 subtype)
        let inner_count = u.int_in_range(1..=2)?;
        let mut sub_types = Vec::with_capacity(inner_count);
        for _ in 0..inner_count {
            sub_types.push(gen_ident(u)?);
        }

        result.push((main_name, sub_types));
    }

    Ok(result)
}

// A single generic helper function used by the attributes above
pub fn gen_vec_min_1<'a, T: Arbitrary<'a>>(u: &mut Unstructured<'a>) -> arbitrary::Result<Vec<T>> {
    let count = u.int_in_range(1..=2)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        items.push(T::arbitrary(u)?);
    }
    Ok(items)
}

pub fn gen_vec_min_1_kvs(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Vec<(String, String)>> {
    let remaining = u.len();

    // 1. Hard Fallback
    // If entropy is critically low, return a single simple pair 
    // to guarantee the "min_1" constraint is satisfied without failing.
    if remaining < 32 {
        return Ok(vec![("Key".to_string(), "Val".to_string())]);
    }

    // 2. Determine count (1 to 4 is usually plenty for AST testing)
    let count = u.int_in_range(1..=4)?;
    let mut kvs = Vec::with_capacity(count);

    for _ in 0..count {
        // Use your existing gen_ident or a simple safe string generator
        let key = gen_ident(u)?;
        let val = gen_ident(u)?;
        kvs.push((key, val));
    }

    Ok(kvs)
}

pub fn gen_vec_min_1_kvis(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Vec<(String, String, IntExpr)>> {
    let remaining = u.len();

    // 1. Hard Fallback
    // If entropy is critically low, return a single simple pair 
    // to guarantee the "min_1" constraint is satisfied without failing.
    if remaining < 32 {
        return Ok(vec![("Key".to_string(), "Val".to_string(), IntExpr::Literal { int: 1 })]);
    }

    // 2. Determine count (1 to 4 is usually plenty for AST testing)
    let count = u.int_in_range(1..=4)?;
    let mut kvs = Vec::with_capacity(count);

    for _ in 0..count {
        // Use your existing gen_ident or a simple safe string generator
        let key = gen_ident(u)?;
        let val = gen_ident(u)?;
        let int = IntExpr::arbitrary(u)?;

        kvs.push((key, val, int));
    }

    Ok(kvs)
}

pub fn gen_vec_min_1_ints(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Vec<IntExpr>> {
    let remaining = u.len();

    // 1. Hard Fallback
    // If we are low on entropy, don't even try to generate a list.
    // Return a single Literal(0) to satisfy the "min 1" requirement instantly.
    if remaining < 64 {
        return Ok(vec![IntExpr::Literal { int: 0 }]);
    }

    // 2. Controlled Generation
    // We cap the number of expressions in a collection to 1-3. 
    // Large collections of complex expressions are rarely needed for AST testing.
    let count = u.int_in_range(1..=3)?;
    let mut ints = Vec::with_capacity(count);

    for _ in 0..count {
        // Since IntExpr has its own entropy-check/fallback, 
        // this call is safe.
        ints.push(u.arbitrary()?);
    }

    Ok(ints)
}

pub fn gen_flows_safe(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Vec<FlowComponent>> {
    let remaining = u.len();

    // 1. Hard Fallback (The Dead End)
    // If we have very little gas, return a single, simple action.
    // "EndTurn" or a similar trivial rule is best here to stop the recursion.
    if remaining < 128 {
        return Ok(vec![
            FlowComponent::Rule { game_rule: GameRule::Action { action: ActionRule::EndAction { end_type: EndType::Turn } } }
        ]);
    }

    // 2. Controlled Width
    // Even with plenty of entropy, we cap the flow at 1-3 components.
    // Deeply nested logic is better than wide logic for finding bugs.
    let count = u.int_in_range(1..=3)?;
    let mut flows = Vec::with_capacity(count);

    for _ in 0..count {
        // Check entropy again inside the loop
        if u.len() < 64 {
            flows.push(FlowComponent::Rule { game_rule: GameRule::Action { action: ActionRule::EndAction { end_type: EndType::Turn } } });
            break;
        }

        // We manually steer the fuzzer away from recursive variants 
        // (like Sequences or Loops) if entropy is dropping.
        let component = if u.len() < 256 {
            // Force a non-recursive Rule or simple leaf
            FlowComponent::Rule { game_rule: u.arbitrary()? }
        } else {
            // High entropy: allow full recursive generation
            u.arbitrary()?
        };

        flows.push(component);
    }

    Ok(flows)
}

// ===========================================================================
// ===========================================================================
// Recursive Types
// ===========================================================================
// ===========================================================================
impl<'a> Arbitrary<'a> for IntExpr {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        // 1. Check available entropy to prevent infinite recursion
        // If we have fewer than 64 bytes left, we force a "Leaf" node.
        let remaining_bytes = u.len();
        
        if remaining_bytes < 64 {
            return Ok(IntExpr::Literal { int: 1 })
            // Pick between the three non-recursive leaf types
            // return match u.int_in_range(0..=2)? {
            //     0 => Ok(IntExpr::Literal { int: u.arbitrary()? }),
            //     1 => Ok(IntExpr::Runtime { runtime: u.arbitrary()? }),
            //     _ => Ok(IntExpr::Aggregate { aggregate: u.arbitrary()? }),
            // };
        }

        // 2. Weighted selection for normal generation
        // We give Literal/Runtime/Aggregate high weight so the trees don't get too deep.
        match u.int_in_range(0..=100)? {
            // 70% chance to be a Leaf (terminates recursion)
            0..=20 => Ok(IntExpr::Literal { int: u.arbitrary()? }),
            21..=40 => Ok(IntExpr::Memory { memory: gen_ident(u)? }),
            41..=55 => Ok(IntExpr::Runtime { runtime: u.arbitrary()? }),
            56..=70 => Ok(IntExpr::Aggregate { aggregate: u.arbitrary()? }),
            
            // 15% chance to be a Query
            71..=85 => Ok(IntExpr::Query { query: u.arbitrary()? }),
            
            // 15% chance to be a Binary (recursive)
            _ => Ok(IntExpr::Binary {
                int: Box::new(u.arbitrary()?),
                op: u.arbitrary()?,
                int1: Box::new(u.arbitrary()?),
            }),
        }
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        // Minimum 1 byte for the variant discriminant
        (1, None)
    }
}

impl<'a> Arbitrary<'a> for PlayerExpr {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        // We use a weighted match to prioritize Literals and Runtime players
        // over complex Queries or Aggregates, which helps keep the AST size stable.
        match u.int_in_range(0..=100)? {
            // 40% chance for a literal name
            0..=39 => Ok(PlayerExpr::Literal {
                name: gen_player_name(u)?,
            }),
            // 30% chance for simple runtime (Current, Next, etc.)
            40..=69 => Ok(PlayerExpr::Runtime {
                runtime: u.arbitrary()?,
            }),
            // 15% chance for an aggregate (Owner of card, etc.)
            70..=84 => Ok(PlayerExpr::Aggregate {
                aggregate: u.arbitrary()?,
            }),
            // 15% chance for a query (Turn order, etc.)
            _ => Ok(PlayerExpr::Query {
                query: u.arbitrary()?,
            }),
        }
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        // Minimum 1 byte for the variant discriminant
        (1, None)
    }
}

impl<'a> Arbitrary<'a> for StringExpr {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        // We prioritize Literals because they are the "base case" for 
        // string-based game logic (like checking a card's name or a location).
        match u.int_in_range(0..=100)? {
            // 70% chance for a Literal identifier
            0..=35 => Ok(StringExpr::Literal {
                value: gen_ident(u)?,
            }),
            // 70% chance for a Literal identifier
            36..=69 => Ok(StringExpr::Memory {
                memory: gen_ident(u)?,
            }),
            
            // 30% chance for a Query (KeyOf, CollectionAt, etc.)
            _ => Ok(StringExpr::Query {
                query: u.arbitrary()?,
            }),
        }
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        // Minimum 1 byte for the variant, but we know gen_ident 
        // will consume at least a few more bytes.
        (1, None)
    }
}

impl<'a> Arbitrary<'a> for BoolExpr {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let remaining = u.len();

        // 1. Forced Termination (The "Safety Valve")
        // If we have very little entropy left, we MUST pick the non-recursive branch.
        // 128 bytes is a safe buffer for a CompareBool + its inner expressions.
        if remaining < 128 {
            return Ok(BoolExpr::Aggregate {
                aggregate: AggregateBool::CardSetEmpty { 
                  card_set: CardSet::Group { 
                    group: Group::Groupable { 
                      groupable: Groupable::Location { 
                        name: "BASECASE".to_string() 
                      } 
                    } 
                  } 
                }
            });
        }

        // 2. Probability-Based Selection
        // We heavily weight towards Aggregate (75%) to keep the average tree depth low.
        match u.int_in_range(0..=100)? {
            // Base Case: Comparison/Aggregate (e.g., x > 5)
            0..=74 => Ok(BoolExpr::Aggregate {
                aggregate: u.arbitrary()?,
            }),
            
            // Recursive Case: Unary (NOT x)
            75..=84 => Ok(BoolExpr::Unary {
                op: u.arbitrary()?,
                bool_expr: Box::new(u.arbitrary()?),
            }),
            
            // Recursive Case: Binary (x AND y)
            _ => Ok(BoolExpr::Binary {
                bool_expr: Box::new(u.arbitrary()?),
                op: u.arbitrary()?,
                bool_expr1: Box::new(u.arbitrary()?),
            }),
        }
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        // (Minimum bytes, Maximum bytes)
        // At minimum, we need 1 byte for the variant choice.
        (1, None)
    }
}

impl<'a> Arbitrary<'a> for FilterExpr {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let remaining = u.len();

        // 1. Safety Valve
        // If we have less than 128 bytes, don't risk a recursive Binary branch.
        // AggregateFilter can be quite large (e.g., Higher/Lower with strings),
        // so we need a decent buffer to generate it successfully.
        if remaining < 128 {
            return Ok(FilterExpr::Aggregate {
                aggregate: AggregateFilter::Combo { 
                    combo: "BASECASE".to_string() 
                },
            });
        }

        // 2. Weighted Selection
        match u.int_in_range(0..=100)? {
            // 80% chance for a leaf (Aggregate)
            0..=79 => Ok(FilterExpr::Aggregate {
                aggregate: u.arbitrary()?,
            }),
            // 20% chance for a recursive Binary (AND/OR)
            _ => Ok(FilterExpr::Binary {
                filter: Box::new(u.arbitrary()?),
                op: u.arbitrary()?,
                filter1: Box::new(u.arbitrary()?),
            }),
        }
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (1, None)
    }
}

pub fn gen_vec_min_1_players(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Vec<PlayerExpr>> {
    // 1. Hard Fallback
    // If we are low on bytes, return a single Runtime player (Current).
    // This is the "cheapest" possible valid PlayerCollection.
    if u.len() < 32 {
        return Ok(vec![PlayerExpr::Runtime {
            runtime: RuntimePlayer::Current,
        }]);
    }

    // 2. Normal Generation
    let count = u.int_in_range(1..=3)?;
    let mut players = Vec::with_capacity(count);
    for _ in 0..count {
        players.push(u.arbitrary()?);
    }
    Ok(players)
}

impl<'a> Arbitrary<'a> for PlayerCollection {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let remaining = u.len();

        // If entropy is low, force the simplest possible variant: Runtime
        if remaining < 64 {
            return Ok(PlayerCollection::Runtime {
                runtime: RuntimePlayerCollection::PlayersIn, // Or any safe default
            });
        }

        match u.int_in_range(0..=100)? {
            // 40% chance for Runtime (Very cheap)
            0..=20 => Ok(PlayerCollection::Runtime {
                runtime: u.arbitrary()?,
            }),
            // 70% chance for a Literal identifier
            21..=39 => Ok(PlayerCollection::Memory {
                memory: gen_ident(u)?,
            }),
            
            // 40% chance for Literal (Uses our safe vec generator)
            40..=79 => Ok(PlayerCollection::Literal {
                players: gen_vec_min_1_players(u)?,
            }),
            // 20% chance for Aggregate (Potentially expensive)
            _ => Ok(PlayerCollection::Aggregate {
                aggregate: u.arbitrary()?,
            }),
        }
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (1, None)
    }
}

pub fn gen_vec_min_1_teams(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Vec<TeamExpr>> {
    // Hard Fallback: If we're almost out of bytes, return one simple literal team.
    if u.len() < 32 {
        return Ok(vec![TeamExpr::Literal {
            name: "Teamalpha".to_string(),
        }]);
    }

    let count = u.int_in_range(1..=3)?;
    let mut teams = Vec::with_capacity(count);
    for _ in 0..count {
        teams.push(u.arbitrary()?);
    }
    Ok(teams)
}

impl<'a> Arbitrary<'a> for TeamCollection {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let remaining = u.len();

        // 1. Hard Fallback: If entropy is low, return the simplest variant.
        if remaining < 64 {
            return Ok(TeamCollection::Runtime {
                runtime: RuntimeTeamCollection::OtherTeams,
            });
        }

        // 2. Weighted Selection
        match u.int_in_range(0..=100)? {
            // 40% chance for Runtime (Cheap)
            0..=20 => Ok(TeamCollection::Runtime {
                runtime: u.arbitrary()?,
            }),
            // 70% chance for a Literal identifier
            21..=39 => Ok(TeamCollection::Memory {
                memory: gen_ident(u)?,
            }),
            
            // 60% chance for Literal (Uses our safe vec generator)
            _ => Ok(TeamCollection::Literal {
                teams: gen_vec_min_1_teams(u)?,
            }),
        }
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (1, None)
    }
}

impl<'a> Arbitrary<'a> for CardPosition {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let remaining = u.len();

        // 1. Hard Fallback
        // If entropy is low, return a simple Query::Top for a generic location.
        // This is much safer than trying to generate a complex Aggregate.
        if remaining < 64 {
            return Ok(CardPosition::Query {
                query: QueryCardPosition::Top {
                    location: "Deck".to_string(),
                },
            });
        }

        // 2. Weighted Selection
        match u.int_in_range(0..=100)? {
            // 70% chance for Query (usually simpler: Top, Bottom, At)
            0..=69 => Ok(CardPosition::Query {
                query: u.arbitrary()?,
            }),
            
            // 30% chance for Aggregate (PointMaps, Precedence - heavier)
            _ => Ok(CardPosition::Aggregate {
                aggregate: u.arbitrary()?,
            }),
        }
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (1, None)
    }
}

impl<'a> Arbitrary<'a> for Group {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let remaining = u.len();

        // 1. Hard Fallback
        // If entropy is low, return the simplest Groupable (a literal location).
        // This avoids triggering FilterExpr or complex CardPosition lookups.
        if remaining < 128 {
            return Ok(Group::Groupable {
                groupable: Groupable::Location { 
                    name: "Defaultloc".to_string() 
                },
            });
        }

        // 2. Weighted Selection
        match u.int_in_range(0..=100)? {
            // 40% Simple Groupable
            0..=39 => Ok(Group::Groupable {
                groupable: u.arbitrary()?,
            }),
            // 20% Filtered Group (The "Heavy" variant)
            40..=59 => Ok(Group::Where {
                groupable: u.arbitrary()?,
                filter: u.arbitrary()?,
            }),
            // 20% Combo variants
            60..=79 => {
                if u.arbitrary()? {
                    Ok(Group::Combo {
                        combo: gen_ident(u)?,
                        groupable: u.arbitrary()?,
                    })
                } else {
                    Ok(Group::NotCombo {
                        combo: gen_ident(u)?,
                        groupable: u.arbitrary()?,
                    })
                }
            },
            // 20% CardPosition
            _ => Ok(Group::CardPosition {
                card_position: u.arbitrary()?,
            }),
        }
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (1, None)
    }
}
