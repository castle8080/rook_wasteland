# rw_sixzee — UI Wireframes

ASCII wireframes for all screens and overlays. Columns in the scorecard alternate
between two background tones (odd columns light, even columns slightly shaded) —
shown in real UI via CSS, represented here by plain cells for readability.

---

## Screen 0: Opening Quote Overlay

Shown at the start of every new game, before the first roll. Grandma delivers an oracular
opening line. Player dismisses to begin. Tab bar is not shown.

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│                         SIXZEE                             │
│                                                             │
│                                                             │
│         ┌───────────────────────────────────────┐          │
│         │  👵                                   │          │
│         │                                       │          │
│         │  "The rice knows when it is ready     │          │
│         │   to be eaten. Do you?"               │          │
│         │                                       │          │
│         │                    — Grandma          │          │
│         └───────────────────────────────────────┘          │
│                                                             │
│                    [ Let's play. ]                         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Notes:**
- Quote is selected randomly from the `opening` pool in `grandma_quotes.json`; different each game
- Tab bar is NOT shown on this screen
- Tapping "Let's play." (or anywhere outside the card) dismisses the overlay and begins the game
- If `QuoteBank` failed to load, this overlay is skipped entirely and the game starts directly
- No dice, no scorecard visible yet

---

## Screen 1: Game Screen — Between Turns (no dice rolled yet)

The default state at the start of a fresh turn. Roll button is active; Ask Grandma
button is disabled until the first roll.

```
┌─────────────────────────────────────────────────────────────┐
│  SIXZEE                              Turn 14  ●●● 3 rolls   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌───┐   ┌───┐   ┌───┐   ┌───┐   ┌───┐                   │
│   │   │   │   │   │   │   │   │   │   │                   │
│   │ ? │   │ ? │   │ ? │   │ ? │   │ ? │                   │
│   │   │   │   │   │   │   │   │   │   │                   │
│   └───┘   └───┘   └───┘   └───┘   └───┘                   │
│                                                             │
│           [ 🎲  ROLL ]        [ 👵 ASK GRANDMA ░░ ]           │
│                                ^^^^ disabled until rolled   │
├─────────────────────────────────────────────────────────────┤
│            ── SCORECARD ──                                  │
│                  │ C1  │ C2  │ C3  │ C4  │ C5  │ C6  │     │
│ ─────────────────┼─────┼─────┼─────┼─────┼─────┼─────┤    │
│ Ones             │  3  │     │  2  │  4  │     │  1  │    │
│ Twos             │  6  │     │  8  │ 10  │     │  4  │    │
│ Threes           │     │  9  │ 12  │  6  │  9  │     │    │
│ Fours            │ 12  │  8  │     │     │ 12  │  4  │    │
│ Fives            │     │ 15  │  5  │ 10  │     │ 20  │    │
│ Sixes            │ 18  │     │     │ 24  │ 18  │     │    │
│ ─────────────────┼─────┼─────┼─────┼─────┼─────┼─────┤    │
│ Upper Sub        │ 39  │ 32  │ 27  │ 54  │ 39  │ 29  │    │
│ Bonus (+35≥63)   │     │     │     │ +35 │     │     │    │
│ ─────────────────┼─────┼─────┼─────┼─────┼─────┼─────┤    │
│ 3 of a Kind      │ 22  │     │     │ 18  │     │  0  │    │
│ 4 of a Kind      │     │     │  0  │     │ 24  │     │    │
│ Full House       │ 25  │     │     │ 25  │     │  0  │    │
│ Sm. Straight     │  0  │ 30  │     │     │ 30  │     │    │
│ Lg. Straight     │     │  0  │ 40  │     │     │ 40  │    │
│ SIXZEE          │ 50  │     │ 50  │     │ 50  │     │    │
│ Chance           │ 18  │     │     │ 22  │     │ 27  │    │
│ ─────────────────┼─────┼─────┼─────┼─────┼─────┼─────┤    │
│ Lower Sub        │115  │ 30  │ 90  │ 65  │104  │ 67  │    │
│ ─────────────────┼─────┼─────┼─────┼─────┼─────┼─────┤    │
│ Col Total        │154  │ 62  │117  │154  │143  │ 96  │    │
│                                                             │
│  ┌─────────────────────────┐   GRAND TOTAL:   726          │
│  │  SIXZEE BONUS POOL     │                               │
│  │       +100              │                               │
│  │  (3 of 6 6z filled ✓)  │                               │
│  └─────────────────────────┘                               │
├─────────────────────────────────────────────────────────────┤
│      [ 🎮 Game ]      [ 📋 History ]    [ ⚙️ Settings ]    │
└─────────────────────────────────────────────────────────────┘
```

**Notes:**
- Empty cells (`     `) are open and available for scoring.
- `0` is a deliberately scratched cell (player chose to place zero).
- Turn counter and roll pips (●●●) show remaining rolls for this turn.
- Sixzee bonus pool box shows running total and a progress note on how many
  Sixzee cells are filled (helps the player track bonus eligibility).
- Grand Total sums all 6 column totals plus the bonus pool.

---

## Screen 2: Game Screen — After Rolling (score preview mode)

After the first roll, open cells show the score the current dice would yield in
brackets. Held dice shown with double border. Ask Grandma button now enabled.

```
┌─────────────────────────────────────────────────────────────┐
│  SIXZEE                              Turn 14  ●●○ 2 rolls   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌───┐   ╔═══╗   ╔═══╗   ┌───┐   ╔═══╗                   │
│   │   │   ║   ║   ║   ║   │   │   ║   ║                   │
│   │ 3 │   ║ 5 ║   ║ 5 ║   │ 2 │   ║ 5 ║                   │
│   │   │   ║   ║   ║   ║   │   │   ║   ║                   │
│   └───┘   ╚═══╝   ╚═══╝   └───┘   ╚═══╝                   │
│           HELD     HELD            HELD    tap to hold/unhold│
│                                                             │
│           [ 🎲  ROLL ]        [ 👵 ASK GRANDMA ]              │
├─────────────────────────────────────────────────────────────┤
│            ── SCORECARD ──                                  │
│                  │ C1  │ C2  │ C3  │ C4  │ C5  │ C6  │     │
│ ─────────────────┼─────┼─────┼─────┼─────┼─────┼─────┤    │
│ Ones             │  3  │[  ] │  2  │  4  │[  ] │  1  │    │
│ Twos             │  6  │[ 2] │  8  │ 10  │[ 2] │  4  │    │
│ Threes           │[  ] │  9  │ 12  │  6  │  9  │[  ] │    │
│ Fours            │ 12  │  8  │[ 4] │[  ] │ 12  │  4  │    │
│ Fives            │[15] │ 15  │  5  │ 10  │[15] │ 20  │    │
│ Sixes            │ 18  │[ 6] │[ 6] │ 24  │ 18  │[ 6] │    │
│ ─────────────────┼─────┼─────┼─────┼─────┼─────┼─────┤    │
│ Upper Sub        │ 39  │ 32  │ 27  │ 54  │ 39  │ 29  │    │
│ Bonus (+35≥63)   │     │     │     │ +35 │     │     │    │
│ ─────────────────┼─────┼─────┼─────┼─────┼─────┼─────┤    │
│ 3 of a Kind      │ 22  │[20] │[20] │ 18  │[20] │  0  │    │
│ 4 of a Kind      │[  ] │[  ] │  0  │[  ] │ 24  │[  ] │    │
│ Full House       │ 25  │[  ] │[  ] │ 25  │[  ] │  0  │    │
│ Sm. Straight     │  0  │ 30  │[  ] │[  ] │ 30  │[  ] │    │
│ Lg. Straight     │[  ] │  0  │ 40  │[  ] │[  ] │ 40  │    │
│ SIXZEE          │ 50  │[50] │ 50  │[50] │ 50  │[50] │    │
│ Chance           │ 18  │[20] │[20] │ 22  │[20] │ 27  │    │
│ ─────────────────┼─────┼─────┼─────┼─────┼─────┼─────┤    │
│ Lower Sub        │115  │ 30  │ 90  │ 65  │104  │ 67  │    │
│ ─────────────────┼─────┼─────┼─────┼─────┼─────┼─────┤    │
│ Col Total        │154  │ 62  │117  │154  │143  │ 96  │    │
│                                                             │
│  ┌──────────────────────────┐   GRAND TOTAL:   726         │
│  │  SIXZEE BONUS POOL      │                              │
│  │        +100              │                              │
│  └──────────────────────────┘                              │
├─────────────────────────────────────────────────────────────┤
│      [ 🎮 Game ]      [ 📋 History ]    [ ⚙️ Settings ]   │
└─────────────────────────────────────────────────────────────┘
```

**Notes:**
- `[ n]` = open cell with score preview for current dice. Clicking places the score.
- `[  ]` = open cell that would score 0 with current dice — clicking triggers
  zero-score confirmation prompt.
- Filled cells (plain numbers) are not clickable.
- Held dice use double-border (╔═══╗). Tap/click a die to toggle held state.
- **Sixzee inline quote:** If the current dice all show the same value (a Sixzee),
  a Grandma quote appears in a small banner below the dice row, above the Roll/Ask
  Grandma buttons:

  ```
  │           [ 🎲 ROLL ]     [ 👵 ASK GRANDMA ]          │
  │                                                         │
  │  👵 "Five of one mind. Rare. Do not waste it."         │
  │                                                         │
  ├─────────────────────────────────────────────────────────┤
  ```

  - Quote selected randomly from `sixzee` pool on each such roll
  - Omitted if QuoteBank unavailable

---

## Screen 3: Ask Grandma Panel Overlay

Opens over the game screen when the Ask Grandma button is pressed after a roll.
Shows top 5 actions ranked by estimated end-game score.

```
┌─────────────────────────────────────────────────────────────┐
│  SIXZEE                              Turn 14  ●●○ 2 rolls   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ╔═════════════════════════════════════════════════════╗   │
│  ║  👵 GRANDMA'S ADVICE — Top 5 Moves          [ ✕ close ] ║   │
│  ╠═════════════════════════════════════════════════════╣   │
│  ║                                                     ║   │
│  ║  #1  Hold [5, 5, 5] — reroll 2 dice                ║   │
│  ║      Sixzee chance: ~3%   4-of-a-kind: ~22%       ║   │
│  ║      Est. final score: 1 047                        ║   │
│  ║      [ Apply this move ]                            ║   │
│  ║ ─────────────────────────────────────────────────  ║   │
│  ║  #2  Score Fives → Column 1  (15 pts)              ║   │
│  ║      Est. final score: 1 031                        ║   │
│  ║      [ Apply this move ]                            ║   │
│  ║ ─────────────────────────────────────────────────  ║   │
│  ║  #3  Hold [5, 5, 5, 2] — reroll 1 die              ║   │
│  ║      4-of-a-kind chance: ~17%   Sixzee: ~17%      ║   │
│  ║      Est. final score: 1 019                        ║   │
│  ║      [ Apply this move ]                            ║   │
│  ║ ─────────────────────────────────────────────────  ║   │
│  ║  #4  Score 3 of a Kind → Column 2  (20 pts)        ║   │
│  ║      Est. final score: 1 008                        ║   │
│  ║      [ Apply this move ]                            ║   │
│  ║ ─────────────────────────────────────────────────  ║   │
│  ║  #5  Score Fives → Column 5  (15 pts)              ║   │
│  ║      Est. final score:   994                        ║   │
│  ║      [ Apply this move ]                            ║   │
│  ║                                                     ║   │
│  ║  Based on DP value table + MC sampling               ║   │
│  ╚═════════════════════════════════════════════════════╝   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Notes:**
- Tab bar is **hidden** while Grandma's Advice panel is open (per PRD Req 53).
- Reroll actions show probability estimates for the most likely target outcomes.
- Every action shows an estimated final game score via DP value table + Monte Carlo sampling.
- "Apply this move" sets held dice (reroll) or places score (score-now) and closes
  the panel. Score-now that would place 0 still triggers the confirmation prompt.
- The footer note gives transparency on the computation method.

---

## Screen 4a: Zero-Score Confirmation Prompt

Shown when the player clicks a cell that would score 0. Normal row.

```
        ┌───────────────────────────────────────┐
        │                                       │
        │  Place 0 in  Sm. Straight — Col 3?   │
        │                                       │
        │  This cell would score  0 points      │
        │  with your current dice.              │
        │                                       │
        │  👵 "Sometimes you give back          │
        │     what was never yours."            │
        │                                       │
        │   [ Cancel ]      [ Confirm Zero ]   │
        │                                       │
        └───────────────────────────────────────┘
```

**Notes:**
- Scratch quote appears between the score text and the action buttons
- Quote is random from `scratch` pool; shown whenever the prompt appears
- If QuoteBank unavailable, the quote area is simply omitted

## Screen 4b: Zero-Score Confirmation — Sixzee Cell Warning

Same prompt, but when the cell being scratched is a Sixzee cell.
Grandma scratch quote appears above the forfeit warning.

```
        ┌───────────────────────────────────────┐
        │                                       │
        │   Place 0 in  SIXZEE — Col 2?        │
        │                                       │
        │   This cell would score  0 points     │
        │   with your current dice.             │
        │                                       │
        │  👵 "Sometimes you give back          │
        │     what was never yours."            │
        │                                       │
        │  ┌─────────────────────────────────┐  │
        │  │ ⚠️  WARNING                     │  │
        │  │ Scratching a Sixzee cell will  │  │
        │  │ permanently forfeit your entire │  │
        │  │ Sixzee Bonus Pool.             │  │
        │  └─────────────────────────────────┘  │
        │                                       │
        │   [ Cancel ]      [ Confirm Zero ]   │
        │                                       │
        └───────────────────────────────────────┘
```

---

## Screen 5: End-of-Game Summary Overlay

Shown when all 78 cells are filled. Appears over the completed scorecard.

```
┌─────────────────────────────────────────────────────────────┐
│  SIXZEE                                                     │
├─────────────────────────────────────────────────────────────┤
│  (completed scorecard visible behind overlay)               │
│                                                             │
│  ╔═════════════════════════════════════════════════════╗   │
│  ║           🎲  GAME COMPLETE  🎲                     ║   │
│  ╠═════════════════════════════════════════════════════╣   │
│  ║                                                     ║   │
│  ║   Scorecard Total:              1 204               ║   │
│  ║   Sixzee Bonus Pool:            +300               ║   │
│  ║   ─────────────────────────────────                ║   │
│  ║   FINAL SCORE:                  1 504               ║   │
│  ║                                                     ║   │
│  ║   ⭐  Best Column: Column 4  —  278 pts             ║   │
│  ║                                                     ║   │
│  ║   👵 "You did not embarrass the family.             ║   │
│  ║      I have seen worse."                            ║   │
│  ║                                                     ║   │
│  ║   [ 🎮  New Game ]   [ 📋  View Full Scorecard ]   ║   │
│  ║                                                     ║   │
│  ╚═════════════════════════════════════════════════════╝   │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│      [ 🎮 Game ]      [ 📋 History ]    [ ⚙️ Settings ]   │
└─────────────────────────────────────────────────────────────┘
```

**Notes:**
- "View Full Scorecard" navigates to the History detail view for this game.
- "New Game" dismisses the overlay and resets everything.
- If the Sixzee bonus was forfeited, the pool line shows "+0 (forfeited)".
- Closing quote placed between the best-column line and the action buttons.
- Quote tier determined by final grand total vs theoretical max (see §11.2 in tech_spec.md).
- Example above is a `good` tier quote. Omitted if QuoteBank unavailable.

---

## Screen 6: Resume Prompt (App Load)

Shown on startup when an in-progress game is found in localStorage.

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│                       SIXZEE                               │
│                                                             │
│  ╔═════════════════════════════════════════════════════╗   │
│  ║   🎲  Welcome back!                                 ║   │
│  ║                                                     ║   │
│  ║   You have a game in progress.                      ║   │
│  ║                                                     ║   │
│  ║   Started:       Mar 7, 2026                        ║   │
│  ║   Turn:          34 of 78+ cells                    ║   │
│  ║   Current score: 412                                ║   │
│  ║                                                     ║   │
│  ║   [ ▶  Resume Game ]                                ║   │
│  ║                                                     ║   │
│  ║   [ 🗑  Discard and Start New ]                     ║   │
│  ║                                                     ║   │
│  ╚═════════════════════════════════════════════════════╝   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Notes:**
- Shows just enough context (date started, progress, score so far) so the player
  can decide whether to resume without needing to see the full board.
- Tab bar is not shown here — the player must make this choice before entering the app.

---

## Screen 7: History Screen

Accessible via the History tab. Lists all completed games, highest score first.

```
┌─────────────────────────────────────────────────────────────┐
│  History                                                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Rank  Date           Score    Sixzee Bonus               │
│  ────  ─────────────  ───────  ─────────────               │
│  🥇 1  Mar 7, 2026    1 504         +300    [ View → ]     │
│  🥈 2  Feb 28, 2026   1 312         +100    [ View → ]     │
│  🥉 3  Feb 14, 2026   1 289           +0    [ View → ]     │
│     4  Jan 31, 2026   1 201         +200    [ View → ]     │
│     5  Jan 12, 2026   1 144           +0    [ View → ]     │
│     6  Dec 30, 2025   1 089           +0    [ View → ]     │
│     7  Dec 18, 2025   1 044         +100    [ View → ]     │
│     8  Dec 5, 2025      987           +0    [ View → ]     │
│     9  Nov 22, 2025     931           +0    [ View → ]     │
│    10  Nov 8, 2025      876           +0    [ View → ]     │
│                                                             │
│                   — 10 games shown —                       │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│      [ 🎮 Game ]      [ 📋 History ]    [ ⚙️ Settings ]   │
└─────────────────────────────────────────────────────────────┘
```

**Notes:**
- Gold/silver/bronze medals on top 3 rows are a small visual reward.
- Sixzee Bonus column is `+0` (not "forfeited") to keep the display neutral.
- Games older than 365 days are not shown (pruned on load).

---

## Screen 8: History Detail — Scorecard Snapshot

Read-only view of a completed game's full scorecard. Identical layout to the
active scorecard but all cells are filled and no score previews are shown.

```
┌─────────────────────────────────────────────────────────────┐
│  [ ← History ]    Mar 7, 2026 — Final Score: 1 504         │
├─────────────────────────────────────────────────────────────┤
│            ── SCORECARD ──                                  │
│                  │ C1  │ C2  │ C3  │ C4  │ C5  │ C6  │     │
│ ─────────────────┼─────┼─────┼─────┼─────┼─────┼─────┤    │
│ Ones             │  4  │  1  │  3  │  4  │  2  │  0  │    │
│ Twos             │ 10  │  6  │  8  │ 10  │  4  │  6  │    │
│ Threes           │  9  │ 12  │  6  │  9  │ 12  │  3  │    │
│ Fours            │ 12  │  8  │ 12  │ 16  │  4  │  8  │    │
│ Fives            │ 20  │ 15  │ 10  │ 15  │ 15  │ 20  │    │
│ Sixes            │ 18  │ 24  │  6  │ 24  │ 18  │ 12  │    │
│ ─────────────────┼─────┼─────┼─────┼─────┼─────┼─────┤    │
│ Upper Sub        │ 73  │ 66  │ 45  │ 78  │ 55  │ 49  │    │
│ Bonus (+35≥63)   │+35  │ +35 │     │ +35 │     │     │    │
│ ─────────────────┼─────┼─────┼─────┼─────┼─────┼─────┤    │
│ 3 of a Kind      │ 24  │ 22  │ 18  │ 26  │ 20  │ 14  │    │
│ 4 of a Kind      │  0  │ 24  │  0  │ 26  │ 22  │  0  │    │
│ Full House       │ 25  │  0  │ 25  │ 25  │ 25  │ 25  │    │
│ Sm. Straight     │ 30  │ 30  │  0  │ 30  │ 30  │ 30  │    │
│ Lg. Straight     │ 40  │  0  │ 40  │ 40  │ 40  │ 40  │    │
│ SIXZEE          │ 50  │ 50  │  0  │ 50  │ 50  │ 50  │    │
│ Chance           │ 22  │ 27  │ 19  │ 28  │ 21  │ 16  │    │
│ ─────────────────┼─────┼─────┼─────┼─────┼─────┼─────┤    │
│ Lower Sub        │191  │153  │102  │225  │208  │175  │    │
│ ─────────────────┼─────┼─────┼─────┼─────┼─────┼─────┤    │
│ Col Total        │299  │254  │147  │338  │263  │224  │    │
│                                                             │
│  ┌──────────────────────────┐   GRAND TOTAL:  1 504        │
│  │  SIXZEE BONUS POOL      │   (incl. bonus)              │
│  │        +300              │                              │
│  └──────────────────────────┘                              │
├─────────────────────────────────────────────────────────────┤
│      [ 🎮 Game ]      [ 📋 History ]    [ ⚙️ Settings ]   │
└─────────────────────────────────────────────────────────────┘
```

**Notes:**
- `0` cells are scratched placements (player chose zero). Distinguished visually
  from empty cells in the active game (here all cells are filled).
- The back arrow (`← History`) returns to the History list.
- No dice, no roll button, no Ask Grandma button — purely read-only.

---

## Screen 9: Settings — Theme Picker

Accessible via the Settings tab. Displays all 6 themes with a die face preview
and colour swatch. Active theme is highlighted.

```
┌─────────────────────────────────────────────────────────────┐
│  Settings                                                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  THEME                                                      │
│                                                             │
│  ┌───────────────────────┐  ┌───────────────────────┐      │
│  │ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │  │                       │      │
│  │  ╔═══╗  ⛧ ⛧ ⛧       │  │  ╔═══╗  · · ·        │      │
│  │  ║ ✦ ║  ⛧   ⛧       │  │  ║ ▣ ║  ·   ·        │      │
│  │  ╚═══╝  ⛧ ⛧ ⛧       │  │  ╚═══╝  · · ·        │      │
│  │  DEVIL ROCK           │  │  BORG                 │      │
│  │  ████ neon red/black  │  │  ▓▓▓▓ steel/cyan      │      │
│  └───────────────────────┘  └───────────────────────┘      │
│                                                             │
│  ┌───────────────────────┐  ┌───────────────────────┐      │
│  │                       │  │ ░░░░░░░░░░░░░░░░░░░░░ │      │
│  │  ╔═══╗  💀 💀 💀      │  │  ╔═══╗  ✦ ✦ ✦        │      │
│  │  ║ 💀 ║  💀   💀      │  │  ║ ✦ ║  ✦   ✦        │      │
│  │  ╚═══╝  💀 💀 💀      │  │  ╚═══╝  ✦ ✦ ✦        │      │
│  │  HORROR               │  │  RENAISSANCE          │      │
│  │  ████ crimson/black   │  │  ░░░░ parchment/gold  │      │
│  └───────────────────────┘  └───────────────────────┘      │
│                                                             │
│  ┌───────────────────────┐  ┌───────────────────────┐      │
│  │ ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒ │  │                       │      │
│  │  ╔═══╗  ◆ ◆ ◆       │  │  ╔═══╗  ❧ ❧ ❧        │      │
│  │  ║ ◆ ║  ◆   ◆       │  │  ║ ❧ ║  ❧   ❧        │      │
│  │  ╚═══╝  ◆ ◆ ◆       │  │  ╚═══╝  ❧ ❧ ❧        │      │
│  │  NORDIC MINIMAL ✓    │  │  PACIFIC NORTHWEST    │      │
│  │  ▒▒▒▒ off-white/slate│  │  ████ forest/tan      │      │
│  └───────────────────────┘  └───────────────────────┘      │
│                              ^^^^ ✓ = currently active     │
│                                                             │
│  Theme applies instantly — no reload needed.               │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│      [ 🎮 Game ]      [ 📋 History ]    [ ⚙️ Settings ]   │
└─────────────────────────────────────────────────────────────┘
```

**Notes:**
- Each theme card shows: a die face (face-6 as representative), a colour swatch
  strip, the theme name, and the active indicator (✓).
- Tapping/clicking any card immediately applies the theme to the whole app.
- Die face previews show the custom SVG symbol set for that theme.

---

## Screen 10: Mobile Layout — Game Screen (condensed)

Viewport < 600 px. Scorecard rows use abbreviated labels; cell values are
smaller. Tab bar remains pinned at the bottom.

```
┌───────────────────────────┐
│ SIXZEE        Turn 14 ●●○ │
├───────────────────────────┤
│ ┌──┐ ┌──┐ ╔══╗ ┌──┐ ╔══╗ │
│ │ 3│ │ 2│ ║ 5║ │ 2│ ║ 5║ │
│ └──┘ └──┘ ╚══╝ └──┘ ╚══╝ │
│          HELD      HELD   │
│  [ 🎲 ROLL ]  [ 👵 GRAN ] │
├───────────────────────────┤
│     │C1 │C2 │C3 │C4 │C5│C6│
│─────┼───┼───┼───┼───┼──┼──│
│ 1s  │ 3 │   │ 2 │ 4 │  │ 1│
│ 2s  │ 6 │   │ 8 │10 │  │ 4│
│ 3s  │   │ 9 │12 │ 6 │ 9│  │
│ 4s  │12 │ 8 │   │   │12│ 4│
│ 5s  │[15│ 15│ 5 │10 │15│20│
│ 6s  │18 │[6]│[6]│24 │18│[6│
│─────┼───┼───┼───┼───┼──┼──│
│ Sub │39 │32 │27 │54 │39│29│
│ Bon │   │   │   │+35│  │  │
│─────┼───┼───┼───┼───┼──┼──│
│ 3K  │22 │[20│[20│18 │20│ 0│
│ 4K  │[  │[  │ 0 │[  │24│[ │
│ FH  │25 │[  │[  │25 │[ │ 0│
│ SS  │ 0 │30 │[  │[  │30│[ │
│ LS  │[  │ 0 │40 │[  │[ │40│
│ YZ  │50 │[50│50 │[50│50│[5│
│ CH  │18 │[20│[20│22 │20│27│
│─────┼───┼───┼───┼───┼──┼──│
│ Tot │154│ 62│117│154│143│96│
├───────────────────────────┤
│ YZ BONUS: +100            │
│ GRAND TOTAL: 726          │
├───────────────────────────┤
│ [🎮 Game][📋 Hist][⚙️ Set]│
└───────────────────────────┘
```

**Notes:**
- Row labels abbreviated: 1s–6s, 3K, 4K, FH, SS, LS, YZ, CH, Sub, Bon, Tot.
- Cell values are truncated to fit — 2-digit scores are fine; 3-digit scores
  (e.g. column totals) may need horizontal scroll or a wider total row.
- `[` prefix on a preview score indicates the cell is open with a score preview;
  shown truncated in this narrow view.
- All touch targets remain ≥ 44 px via CSS padding even though the displayed
  text is small.
- Dice slightly smaller but still clearly tappable. Held state still uses
  double-border.
