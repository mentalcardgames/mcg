# Poker Rule Violation Analysis Report

## Game Log Summary
Based on the provided game log from a showdown stage, multiple critical poker rule violations have been identified that indicate serious bugs in the game engine.

## Identified Issues

### 1. **CRITICAL: Invalid "Bet 0" Actions**

**Issue Description:**
The game log shows players making "bets 0" actions during post-flop betting rounds:
- "Bot 3 bets 0" during flop
- "Bot 2 bets 0" during turn

**Root Cause Analysis:**
This is caused by the bot AI logic in `/native_mcg/src/bot.rs` where:
```rust
let min_raise = (context.current_bet as f64 * 0.5) as u32; // Line 99
```
When `current_bet` is very small (like 1-2 chips), the calculation `(1 * 0.5) as u32` rounds down to 0.

**Impact:**
- Violates poker rules - betting 0 is not a valid action
- Should be converted to a "check" action instead
- Corrupts the betting round logic

**Recommended Fix:**
1. In bot AI logic, ensure minimum bet is at least the big blind:
   ```rust
   let min_raise = ((context.current_bet as f64 * 0.5) as u32).max(context.big_blind);
   ```
2. Add validation in betting logic to reject Bet(0) actions
3. Auto-convert Bet(0) to CheckCall in the action handler

### 2. **CRITICAL: Stack Management Corruption**

**Issue Description:**
Multiple players show stack=0 at showdown but participated in the hand:
- "You" has stack 0 but made bets throughout the hand
- Bot 2 and Bot 3 both have stack 0 but reached showdown
- Pot shows 0 at showdown despite 4000 chips being wagered

**Root Cause Analysis:**
The stack management system appears to be corrupted, likely due to:
1. All-in detection logic not working properly
2. Pot calculation errors when players go all-in
3. Side-pot logic may be missing or broken

**Impact:**
- Game state becomes inconsistent
- Players can continue betting with 0 stacks
- Pot distribution is incorrect

**Recommended Fix:**
1. Add comprehensive stack validation before allowing actions
2. Fix all-in detection and handling
3. Implement proper side-pot calculation
4. Add invariant checks: `sum(player_stacks) + pot == total_chips`

### 3. **MAJOR: Hand Evaluation Display Errors**

**Issue Description:**
The showdown display shows incorrect hand descriptions:
- "You" with J♣, 7♥ + board K♥, T♠, 9♥, 9♣, 4♣ shows as "Pair" but should be "Two Pair" (9s and Ks)
- Hand rankings don't match the actual best 5-card combinations

**Root Cause Analysis:**
The issue appears to be in the display logic in `/frontend/src/game/screens/poker/ui_components.rs` rather than the evaluation itself. The evaluation tests pass, but the presentation of results is incorrect.

**Impact:**
- Confusing for players
- May indicate deeper evaluation bugs
- Reduces trust in the game

**Recommended Fix:**
1. Verify the `pick_best_five` function correctly identifies the actual cards used
2. Fix the display logic to show the correct hand category
3. Add validation that displayed hands match evaluation results

### 4. **MAJOR: Missing Hole Card Visibility**

**Issue Description:**
Bot hole cards are not revealed at showdown, making it impossible to verify hand evaluations:
- Bot 2 shows "Two Pair [4♠, K♥, 9♥, 9♣, 4♣]" but hole cards unknown
- Bot 3 shows "Two Pair [J♥, K♦, K♥, 9♥, 9♣]" but hole cards unknown

**Impact:**
- Cannot verify correctness of hand evaluations
- Violates poker showdown rules
- Makes debugging impossible

**Recommended Fix:**
1. Ensure all players' hole cards are revealed at showdown
2. Display format: "Bot X (A♠ 2♥): Two Pair [...]"

### 5. **MINOR: Inconsistent Action Logging**

**Issue Description:**
Some betting actions logged inconsistently:
- "Bot 3 calls 10" vs "You calls 867" - should be consistent format

**Recommended Fix:**
Standardize action logging format across all actions.

## Test Recommendations

### Immediate Tests Needed:
1. **Bot AI Bet 0 Test**: Verify bots never generate Bet(0) actions
2. **Stack Integrity Test**: Verify stacks + pot = constant throughout hand  
3. **All-in Handling Test**: Test proper all-in detection and side-pot creation
4. **Hand Evaluation Display Test**: Verify displayed hands match actual evaluations

### Integration Test:
Create a test that reproduces the exact scenario from the game log to verify all fixes.

## Priority Assessment

**P0 (Critical - Fix Immediately):**
- Bet 0 actions - breaks fundamental poker rules
- Stack corruption - breaks game integrity

**P1 (High - Fix Soon):**
- Hand evaluation display errors
- Missing hole card revelation

**P2 (Medium):**
- Action logging inconsistencies

## Code Files Requiring Changes

1. `/native_mcg/src/bot.rs` - Fix bet 0 generation
2. `/native_mcg/src/game/betting.rs` - Add bet 0 validation  
3. `/native_mcg/src/game/engine.rs` - Fix stack management
4. `/frontend/src/game/screens/poker/ui_components.rs` - Fix hand display
5. `/native_mcg/src/game/showdown.rs` - Ensure hole card revelation

## Conclusion

The game log reveals serious bugs that would make the poker game unplayable and unfair. The "bet 0" issue is the most critical as it violates fundamental poker rules. All identified issues should be addressed before the game can be considered stable for actual play.