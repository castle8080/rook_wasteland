# rw_sixzee — Product Requirements Document

## 1. Problem Statement

Standard Sixzee is a satisfying game of dice probability and scored placement,
but its single-column scorecard limits strategic depth — each row can only be
filled once, so every decision is a permanent commitment with no room to recover
from a bad sequence. A house variant played with a 6-column scorecard (all columns
following standard Sixzee rules) dramatically expands the strategic space: players
must optimize placement across 78 cells, balancing short-term opportunity against
long-term column structure. A game takes at least 78 turns to complete — one per
cell — but bonus Sixzee rolls award points without consuming a cell, so games
with strong Sixzee luck run longer. rw_sixzee brings this variant to the browser
as a clean, solitaire experience focused on score maximization.

---

## 2. Goals

- A player can complete a full game (all 78 cells filled) entirely within the
  browser, with no installation or account required. Bonus Sixzee rolls extend
  the game beyond 78 turns without consuming a cell.
- The scorecard correctly applies standard Sixzee scoring rules (including upper
  section bonus per column and Sixzee bonus rolls) across all 6 columns.
- At the end of a game, the player sees their final total score (including any
  Sixzee bonus pool) and can immediately start a new game.
- The UI clearly communicates which cells are available, what score a current roll
  would yield in each cell, and which column/row was just filled.
- The game is fully playable on modern mobile browsers (iOS Safari, Android Chrome)
  with touch input, in addition to desktop.
- The game is playable on desktop browsers without a mouse (keyboard or pointer only).

---

## 3. Non-Goals

- **No multiplayer.** This is a solitaire game only. No networking, no turn
  management, no shared state.
- **No accounts or server.** All persistence is via browser `localStorage` only —
  no data ever leaves the device.
- **No undo.** Once a score is placed, it is permanent for that game.
- **No variants on the 6-column rule.** The column count is fixed at 6. No
  configurable column count, no "classic" single-column mode.
- **No animations or sound.** Visual clarity is the priority; dice roll animations
  and sound effects are out of scope.

---

## 4. User Stories

**Primary happy path:**
- As a player, I want to roll 5 dice up to 3 times per turn so that I can build
  the best possible combination before scoring.
- As a player, I want to hold individual dice between rolls so that I can pursue
  a target combination.
- As a player, I want to see the potential score for my current dice in every
  open cell across all 6 columns so that I can make an informed placement decision.
- As a player, I want to click any open cell on the scorecard to lock in my score
  for that turn so that I can advance to the next turn.

- As a player who closed the browser mid-game, I want to be offered a Resume
  option when I return so that I can continue exactly where I left off.
- As a player resuming a game, I want to see my dice and scorecard restored to
  their exact state so that no progress is lost.

**History & high scores:**
- As a player who just finished a game, I want it saved automatically so that I
  don't have to do anything to preserve my score.
- As a player, I want to navigate to a History screen and see all my past games
  sorted by final score (highest first) so that I can track my personal bests.
- As a player, I want to click any past game in the history list to see its full
  scorecard so that I can review how I scored each column.
- As a player, I want history entries older than one year to be automatically
  pruned so that `localStorage` doesn't grow unboundedly.

- As a mobile player, I want to tap a die to hold or unhold it so that I can
  select which dice to keep between rolls without a mouse.
- As a mobile player, I want the scorecard to fit on my phone screen in a
  condensed grid so that I can tap cells without zooming or scrolling awkwardly.

**Themes:**
- As a player who wants a specific vibe, I want to open a Settings screen and
  choose one of 6 artistic themes so that the dice and UI match my mood.
- As a returning player, I want my chosen theme remembered so that I don't have
  to reselect it every session.

**Advisor:**
- As a player who is unsure what to do on a turn, I want to open an advisor panel
  and see the top 5 recommended actions with probability estimates and projected
  end-game scores so that I can make a more informed decision.
- As a player who agrees with a recommendation, I want to click it in the advisor
  panel to apply it immediately (hold dice or place a score) so that I don't have
  to manually replicate the advice.

**Edge cases:**
- As a player, I want to place a zero in any open cell so that I can strategically
  sacrifice a row when no good scoring option exists.
- As a player, I want to see the upper section bonus status per column (running
  total vs. the 63-point threshold) so that I can track whether a bonus is within
  reach.
- As a player who rolls a second (or third) Sixzee, I want the Sixzee bonus
  applied correctly so that I am not penalised for good luck.
- As a player who has just finished turn 78, I want to see my final total score
  clearly so that I know how I did before deciding whether to play again.

---

## 5. Complete Game Rules

This section is the authoritative reference for all game rules. It is
self-contained — no prior knowledge of the source game (traditional dice-and-scorecard games) is assumed.

---

### 5.1 Equipment

- **5 dice**, each a standard six-sided die with face values 1–6.
- **1 scorecard** containing **6 identical columns**, each with **13 scoring
  rows** (78 cells total).
- **1 Sixzee Bonus Pool** counter, shared across all columns.

---

### 5.2 Objective

Fill all 78 cells on the scorecard to maximise the **grand total score**:
the sum of all 6 column totals plus the Sixzee Bonus Pool.

---

### 5.3 Turn Structure

Each turn follows this sequence:

1. **Roll phase — Roll 1.** All 5 dice are rolled (random values 1–6).
2. **Hold phase.** The player may tap/click any dice to mark them as
   **held**. Held dice keep their current value on the next roll.
   Held status can be toggled freely before each roll.
3. **Roll phase — Roll 2** *(optional)*. All un-held dice are re-rolled.
   Held dice are unchanged.
4. **Hold phase.** The player may adjust which dice are held.
5. **Roll phase — Roll 3** *(optional)*. All un-held dice are re-rolled.
   This is the final roll; no further rolling is possible this turn.
6. **Score phase.** The player clicks one open cell anywhere on the
   scorecard to record a score. The cell is permanently filled.

The player may skip to the score phase after Roll 1 or Roll 2 if they
choose not to use all three rolls.

**Exception — Bonus Sixzee turn:** If, at any point during Roll 1, 2, or 3,
all five dice show the same value AND all 6 Sixzee cells are already filled
(with any value, including a scratch of 0), the app immediately detects this
as a bonus Sixzee and ends the turn automatically — no score phase occurs, no
cell is consumed. If the Sixzee Bonus Pool has not been forfeited, 100 points
are added to it. If the pool has been forfeited, no points are awarded but the
turn still ends automatically. The player begins a fresh turn.

---

### 5.4 Scoring Categories

Each column contains the following 13 rows, divided into an upper and lower
section.

#### Upper Section

| Row | Score |
|-----|-------|
| **Ones** | Sum of all dice showing **1** |
| **Twos** | Sum of all dice showing **2** |
| **Threes** | Sum of all dice showing **3** |
| **Fours** | Sum of all dice showing **4** |
| **Fives** | Sum of all dice showing **5** |
| **Sixes** | Sum of all dice showing **6** |

*Example: dice showing [1, 1, 3, 1, 5] scored in Ones = 3.*

**Upper Section Bonus:** If the sum of the six upper rows within a single
column is **63 or more**, that column earns a **+35 bonus**. The 63
threshold is equivalent to scoring exactly 3 of each number (3×1 + 3×2 +
3×3 + 3×4 + 3×5 + 3×6 = 63). Each column's bonus is calculated
independently; a maximum of 6 × 35 = 210 bonus points is possible.

#### Lower Section

| Row | Required combination | Score |
|-----|----------------------|-------|
| **3 of a Kind** | At least three dice showing the same value | Sum of **all 5 dice** |
| **4 of a Kind** | At least four dice showing the same value | Sum of **all 5 dice** |
| **Full House** | Three of one value **and** two of another | **25** |
| **Small Straight** | Any four sequential values (1-2-3-4, 2-3-4-5, or 3-4-5-6) | **30** |
| **Large Straight** | All five dice sequential (1-2-3-4-5 or 2-3-4-5-6) | **40** |
| **Sixzee** | All five dice showing the same value | **50** |
| **Chance** | Any combination (no requirement) | Sum of **all 5 dice** |

*If the dice do not satisfy the requirement for a row, the score is 0.*

---

### 5.5 Placing a Score

On every turn, after rolling at least once, the player **must** place a
score in exactly one open cell. Rules for placement:

- Any open cell in any of the 6 columns may be chosen — there is no
  restriction on which column or row must be used next.
- If the current dice satisfy the row's requirement, the calculated score
  is recorded. If they do not, **0** is recorded (this is called a
  **scratch**).
- A cell may only be filled once per game. Filled cells cannot be changed.
- Placing 0 in any cell requires confirmation. Placing 0 in a **Sixzee
  cell** additionally shows a warning that the Sixzee Bonus Pool will be
  permanently forfeited (see §5.6).

---

### 5.6 Sixzee Bonus Pool

The Sixzee Bonus Pool is a special accumulator separate from the scorecard
columns. It operates as follows:

**Phase 1 — Filling the 6 Sixzee cells (rolls 1–6):**
- The first 6 times the player rolls a Sixzee (all five dice the same
  value), each roll scores **50 points** in the Sixzee row of one column.
- The player chooses which column's Sixzee cell to fill on each of these
  rolls. No bonus pool points are awarded during this phase.

**Phase 2 — Bonus Sixzees (roll 7 onward):**
- Once all 6 Sixzee cells are filled and the player rolls another
  Sixzee, **100 points** are added to the Sixzee Bonus Pool.
- **The bonus turn does not consume a board cell.** The turn ends
  immediately after the bonus is awarded; the player begins a fresh new
  turn with all 5 dice unrolled.
- There is **no cap** on consecutive bonus Sixzees. Each qualifying roll
  adds another 100 points. A game lasts a minimum of 78 cell-filling turns
  but may last longer if bonus Sixzees occur.

**Forfeiture:**
- If the player ever scratches a Sixzee cell (places 0 in any of the 6
  Sixzee rows), the entire Sixzee Bonus Pool is **permanently forfeited**
  for the rest of that game.
- Forfeiture means: the accumulated bonus is set to 0 and no future
  Sixzee rolls (however many) will add to the pool.
- The forfeiture is irreversible. There is no way to restore the pool once
  any Sixzee cell has been scratched.

---

### 5.7 Column Scoring

Each column is scored independently:

```
Upper Section Score  = Ones + Twos + Threes + Fours + Fives + Sixes
Upper Section Bonus  = 35  (if Upper Section Score ≥ 63, else 0)
Lower Section Score  = 3K + 4K + FH + SS + LS + SZ + Chance
Column Total         = Upper Section Score + Upper Section Bonus
                       + Lower Section Score
```

---

### 5.8 Grand Total

```
Grand Total = Column 1 Total + Column 2 Total + Column 3 Total
            + Column 4 Total + Column 5 Total + Column 6 Total
            + Sixzee Bonus Pool
```

The Sixzee Bonus Pool contributes 0 to the Grand Total if it was
forfeited at any point during the game.

---

### 5.9 Game End

The game ends when all 78 cells are filled. A summary is shown with the
final Grand Total. A new game may be started at any time (this discards
the current game permanently).

---

## 6. Functional Requirements

**Dice**

1. The app shall display 5 dice at the start of each turn, all in the "unrolled" state.
2. On the first roll of a turn, all 5 dice shall be rolled (random values 1–6).
3. After the first roll, the player may toggle individual dice as "held"; held dice
   shall not change value on subsequent rolls.
4. The player shall be able to roll up to 3 times per turn. After the third roll,
   the roll button shall be disabled for that turn.
5. The player may score after any roll (1st, 2nd, or 3rd); scoring ends the turn.

**Scorecard**

6. The scorecard shall display 6 columns, each with the 13 standard Sixzee rows:
   Ones, Twos, Threes, Fours, Fives, Sixes (upper section);
   Three of a Kind, Four of a Kind, Full House (25), Small Straight (30),
   Large Straight (40), Sixzee (50), Chance (lower section). Columns shall be
   numbered 1–6 and visually distinguished by alternating between two background
   tones so adjacent columns are always easy to tell apart.
7. Each column shall display an upper section subtotal, an upper section bonus
   (35 points if that column's subtotal ≥ 63, else 0), a lower section subtotal,
   and a column grand total. Each column's bonus is calculated independently; up
   to 6 × 35 = 210 bonus points are possible across the full game.
8. The app shall display a game grand total as the sum of all 6 column totals
   plus the Sixzee Bonus Pool (or 0 if forfeited).
9. While the player has rolled at least once this turn, the app shall display the
   potential score for the current dice in every open cell.
10. Clicking an open cell shall record that score (including 0 for a zero-score
    placement), mark the cell as filled, and advance to the next turn.
11. Clicking a filled cell shall have no effect.

**Sixzee bonus**

12. The first 6 Sixzee rolls a player achieves during a game fill the Sixzee
    cell in one column each (50 pts per cell); no bonus points are awarded for
    these rolls.
13. From the **7th Sixzee roll onward**, if every Sixzee cell across all 6
    columns is filled with 50 (none scratched), 100 points are added to a shared
    "Sixzee Bonus" pool for each such roll.
14. **If any Sixzee cell was ever scratched (scored as 0), the bonus pool is
    permanently forfeited for the rest of that game** — no bonus points are
    awarded on any subsequent Sixzee roll, and the pool displays 0.
15. A bonus Sixzee roll (7th onward) **does not consume a board cell**. The
    app auto-detects a bonus Sixzee as soon as five-of-a-kind appears on any
    roll within a turn (Roll 1, 2, or 3) when all 6 Sixzee cells are already
    filled. The bonus is awarded immediately and the turn ends — no score phase,
    no cell consumed, no further rolling that turn. There is no cap on
    consecutive bonus Sixzee turns — each one awards another 100 pts to the
    pool. A game therefore takes a minimum of 78 turns to complete but may take
    significantly more if bonus Sixzees occur.
16. The Sixzee Bonus pool shall be displayed in a dedicated box separate from
    the main scorecard columns, showing the current accumulated total (or 0 if
    forfeited).

**Game flow**

17. A new game shall begin with all 78 cells empty and the dice in the unrolled state.
18. The game shall end when all 78 cells are filled. The total number of turns
    taken may exceed 78 if bonus Sixzee rolls occurred during the game.
19. At game end, the app shall display a score summary overlay showing: the final
    grand total, the Sixzee bonus pool amount (or 0 if forfeited), and a single
    highlight callout for the best-performing column (highest column grand total).
    The overlay shall offer two actions: **New Game** and **View Full Scorecard**
    (which navigates to the completed game's History detail view). The overlay is
    dismissed by either action; the tab bar remains accessible beneath it.
20. The player shall be able to start a new game at any time; doing so shall reset
    all dice, all scorecard cells, and the Sixzee bonus pool.
21. When the player clicks an open cell that would score 0 for the current dice,
    the app shall show a confirmation prompt before recording the zero. If the
    cell is a **Sixzee cell**, the prompt shall additionally display a bold
    warning: "This will permanently forfeit your entire Sixzee bonus pool."

**Mobile & touch**

22. The layout shall be responsive: on viewports narrower than 600 px the scorecard
    shall render in a condensed grid that fits within the screen width without
    horizontal scrolling, using smaller cell sizes and abbreviated row labels.
23. All interactive targets (dice, scorecard cells, Roll button, Advisor button,
    navigation links) shall have a minimum touch target size of 44 × 44 px on
    mobile viewports.
24. Tapping a die shall toggle its held state, identical in effect to clicking on
    desktop.
25. The app shall not rely on hover states to convey essential information; all
    potential-score previews and held indicators must be visible via tap/click
    state or always-visible UI elements.
26. The advisor panel and the zero-score confirmation prompt shall be displayed as
    full-screen or near-full-screen overlays on mobile viewports, ensuring they
    are readable and dismissible without precise pointer control.

**Move advisor**

27. The app shall display an "Advisor" button during a player's turn, available
    after any roll (1st, 2nd, or 3rd). Before the first roll of a turn the button
    shall be disabled.
28. Pressing the Advisor button shall open an on-demand panel or overlay showing
    the top 5 recommended actions for the current dice and scorecard state, ranked
    by estimated end-game score (highest first).
29. Each recommended action shall be one of two types:
    - **Reroll:** hold a specific set of dice and roll the remaining ones (only
      shown when rolls remain). Described as e.g. "Hold [3, 3, 5] — reroll 2 dice".
    - **Score now:** place the current dice score in a specific cell of a specific
      column. Described as e.g. "Score Fours → Column 3 (16 pts)".
30. Each recommended action shall display:
    - A plain-language description of the action.
    - For reroll actions: the approximate probability (~X%) of achieving the
      target combination on the next roll.
    - For score-now actions: the points that would be recorded.
    - An estimated end-game score — the projected final grand total if this action
      is taken, computed via client-side expected-value calculation over the
      remaining open cells and turns.
31. Probabilities shall be displayed as rounded approximations (e.g. "~28%"), not
    exact decimals.
32. Clicking a recommended action in the advisor panel shall apply it immediately:
    - A reroll action sets the held dice accordingly and closes the panel; the
      player then presses Roll as normal.
    - A score-now action places the score in the specified cell (subject to the
      same zero-score confirmation prompt if applicable) and closes the panel.
33. The advisor computation shall run entirely client-side (no network requests)
    using a precomputed **dynamic programming value table** (one 32 KB table
    covering all single-column states, embedded in the WASM binary) combined
    with Monte Carlo sampling for reroll candidates where exact enumeration is
    too expensive. Score-now candidates are evaluated with a single table
    lookup per cell; reroll candidates with 3 or more dice unheld sample 300
    random outcomes. If computation takes more than ~200 ms the panel shall
    show a loading indicator.
34. The advisor panel shall be dismissible without taking any action.

**Themes & settings**

35. The app shall offer 6 selectable visual themes, each applying a coordinated
    color palette, typography style, and custom SVG dice face art throughout the
    entire UI. The 6 themes are:

    | # | Name | Vibe | Dice face symbols | Palette |
    |---|------|------|-------------------|---------|
    | 1 | **Devil Rock** | 80s heavy metal / occult | Pentagrams, inverted crosses, flames; pip count shown in gothic numerals | Near-black background, neon red and acid yellow accents, distressed textures |
    | 2 | **Borg** | Sci-fi cybernetic / alien collective | Hexagonal circuit nodes, assimilation glyphs, tally marks in binary | Dark charcoal and steel, cold green or cyan grid lines, monospace type |
    | 3 | **Horror** | Classic horror / gothic dread | Skulls, dripping blood drops, claw marks, eyeballs | Deep crimson and black, sickly green highlights, cracked stone texture |
    | 4 | **Renaissance** | 15th–16th century European painting | Illuminated manuscript flourishes, gilded rosettes, Roman numerals | Warm parchment, burnished gold, deep ultramarine, serif calligraphy type |
    | 5 | **Nordic Minimal** | Northern European design / Scandinavian | Clean geometric dots arranged as runes or snowflakes, stark and precise | Off-white, slate grey, single muted accent colour (moss or rust), sans-serif |
    | 6 | **Pacific Northwest** | PNW nature / coastal forest | Cedar rings, salal leaves, salmon silhouettes, mountain outlines | Forest green, driftwood tan, slate river-stone, earthy ink-wash textures |

36. Each theme shall provide a complete set of 6 SVG die face designs (faces
    showing 1 through 6), where pips are replaced by theme-specific symbols
    scaled and arranged to communicate the pip count clearly.
37. The app shall provide a Settings screen accessible via navigation from the
    main game view. The Settings screen shall display a preview of each theme
    (showing a sample die face and a colour swatch) and allow the player to
    select their preferred theme.
38. The active theme shall apply immediately on selection without requiring a
    page reload.
39. The selected theme shall be persisted in `localStorage` and restored on every
    subsequent app load. If no preference is stored, the app shall default to
    **Nordic Minimal** (theme 5).

**Persistence**

40. The app shall auto-save the full game state to `localStorage` after every roll
    and after every score placement. The saved state shall include: all 78 cell
    values (filled or empty), current dice values, which dice are held, current
    roll count (0–3), current turn number, whether the current turn is a bonus
    Sixzee turn, and Sixzee bonus pool (accumulated
    total and forfeited flag).
41. On completion of every game (all 78 cells filled), the app shall move the
    finished game out of the in-progress slot and append a completed game record
    to the history list. The completed record shall contain: timestamp (ISO 8601),
    final grand total score, and the full scorecard snapshot.
42. Game records in the completed history older than 365 days shall be pruned from
    `localStorage` automatically on app load and after each game is saved. The
    in-progress game slot is not subject to pruning.
43. The app shall handle `localStorage` being unavailable (e.g. private browsing
    with storage blocked) gracefully — the game remains fully playable; a non-
    blocking notice informs the player that state will not be saved or resumed.

**Resume on load**

44. On app load, if an in-progress game is found in `localStorage`, the app shall
    present the player with a choice: **Resume** the saved game or **Start New**
    (discarding the saved game).
45. Selecting Resume shall restore the game exactly as it was left — scorecard
    cells, dice values, held state, roll count, and turn number all restored.
46. Selecting Start New shall discard the in-progress save and begin a fresh game.

**Navigation**

47. The app shall display a persistent tab bar at the bottom of the screen with
    three destinations: **Game**, **History**, and **Settings**. The tab bar
    shall be visible on all screens except the following full-screen overlays:
    advisor panel, zero-score confirmation, and resume prompt. The end-of-game
    summary overlay keeps the tab bar visible so the player can navigate freely.
48. Navigating away from the Game tab and back shall not reset or interrupt an
    in-progress game.

**History screen**

49. The History screen (reachable via the History tab) shall list all stored
    completed game records sorted by final score descending (highest score first).
    Each row shall show: rank, date played, final score, and Sixzee bonus pool amount.
50. Clicking a game record shall display that game's full scorecard snapshot in a
    read-only view showing all 6 columns and every cell value, identical in layout
    to the active scorecard.
51. The player shall be able to navigate back from the scorecard snapshot to the
    History list via a back button or swipe gesture.
52. If no completed history exists yet, the History screen shall show an appropriate
    empty-state message.

---

## 7. Out of Scope / Future Work

- **Export / import of history** — no way to transfer history between browsers or devices. Deferred.
- **Statistics dashboard** — win rate, average score, score distribution over time. Interesting but not needed for v1.
- **Keyboard shortcuts** — hold dice with number keys (1–5), roll with Space.
  Useful accessibility feature; deferred to polish milestone.
- **Additional themes** — beyond the 6 shipped themes, community or user-created
  themes are not supported in v1.

---

## 8. Open Questions

*All questions resolved — no open items.*
