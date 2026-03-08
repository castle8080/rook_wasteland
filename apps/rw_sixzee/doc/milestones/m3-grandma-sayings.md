# M3 — Grandma's Sayings

<!-- MILESTONE: M3 -->
<!-- STATUS: COMPLETE -->

**Status:** ✅ COMPLETE
**Depends on:** [M2 — Game State & Scoring Engine](m2-scoring-engine.md) *(for performance tier understanding)*
**Required by:** [M5 — Core Game UI](m5-core-game-ui.md) *(quote display components need the JSON file)*

---

## Overview

This is a **content creation milestone**, not a typical development task. Its output is a single file:
`assets/grandma_quotes.json` — the complete set of Grandma's quotes for every scenario in the game.

The milestone is placed early so the quotes can be drafted and refined throughout development.
The file can be updated at any time without recompiling the app — it is fetched at runtime.

**This milestone is completed through a structured agent-assisted creative conversation**, not
through coding. The process is:
1. Establish Grandma's voice and personality in a dedicated session
2. Use an agent to generate a large pool of quotes for each scenario
3. Review, curate, and edit down to final sets
4. Commit the resulting `assets/grandma_quotes.json`

---

## Grandma's Character

Before generating quotes, the following aspects of her voice should be established and agreed upon:

- **Tone:** Hard-edged oracle. Unimpressed. Has seen things. Does not explain herself.
- **Register:** Short, declarative sentences. No warmth-for-warmth's-sake. But not cruel — she
  is honest, not mean. She has earned the right to say what she means.
- **Source material:** A woman who played dice seriously and took it as a window into character.
  She believes how you play reveals who you are. She believes in luck but not in forcing it.
- **Reference:** Half-Filipino grandmother. The quotes should not be overtly ethnic or
  stereotyped — draw from her *worldview*, not a caricature. Universal wisdom delivered bluntly.
- **Format:** Each quote should be 1–3 short sentences. No more. No explaining. No hedging.

---

## Success Criteria

- [x] `assets/grandma_quotes.json` exists and is valid JSON matching the schema below
- [x] Each quote pool meets the minimum size (see below)
- [x] All quotes reviewed for voice consistency — no quote sounds like a different character
- [x] `really_bad` and `great` closing quotes feel meaningfully different in emotional weight
- [x] Sixzee quotes feel celebratory but still characteristically Grandma (grudging acknowledgment,
  not enthusiasm)
- [x] Scratch quotes feel like quiet judgment, not cruelty
- [x] Opening quotes are genuinely oracular — vague enough to apply to life, not just dice
- [x] File parses successfully with `python -c "import json; json.load(open('assets/grandma_quotes.json'))"`

---

## JSON Schema

File path: `assets/grandma_quotes.json`

```json
{
  "version": 1,
  "opening": [
    "..."
  ],
  "closing": {
    "really_bad": ["..."],
    "bad":        ["..."],
    "ok":         ["..."],
    "good":       ["..."],
    "great":      ["..."]
  },
  "sixzee":  ["..."],
  "scratch": ["..."]
}
```

---

## Minimum Pool Sizes

| Key | Minimum quotes | Notes |
|-----|----------------|-------|
| `opening` | 15 | Shown every game; variety is important |
| `closing.really_bad` | 8 | |
| `closing.bad` | 8 | |
| `closing.ok` | 8 | |
| `closing.good` | 8 | |
| `closing.great` | 8 | |
| `sixzee` | 10 | Shown when all 5 dice match |
| `scratch` | 10 | Shown on zero-score confirmation |
| **Total** | **~75+** | |

---

## Tasks

### Voice Establishment

- [x] Run a dedicated conversation session to establish Grandma's voice:
  - Define 5–10 example quotes as a baseline to lock in the register
  - Agree on what she would and would not say
  - Note any phrases, rhythms, or reference points that feel right
  - Document the resulting voice guide as a prompt preamble for the generation step

### Quote Generation

- [x] Generate `opening` pool (≥15 quotes) — oracular, vague, life observations; the kind of thing
  an old woman says that you don't fully understand until later
- [x] Generate `closing.really_bad` pool (≥8) — not cruel, but honest; something about not
  understanding the fundamentals, about waste
- [x] Generate `closing.bad` pool (≥8) — the dice did not cooperate OR you made poor choices;
  Grandma is not sure which
- [x] Generate `closing.ok` pool (≥8) — adequate; she has seen better; she has seen much worse;
  she will not say so in that order
- [x] Generate `closing.good` pool (≥8) — quiet approval; she does not celebrate, she acknowledges
- [x] Generate `closing.great` pool (≥8) — highest praise is still restrained; perhaps she simply
  notes that the dice were respected
- [x] Generate `sixzee` pool (≥10) — all five same; rare; Grandma notes it without theatrics
- [x] Generate `scratch` pool (≥10) — placing a zero; quiet judgment; "sometimes you must give
  up something"; not scolding, just witness

### Review & Curation

- [x] Read all generated quotes aloud — does each one sound like the same person?
- [x] Remove any quote that is too cheerful, too wordy, or too on-the-nose
- [x] Remove any quote that sounds generic (fortune cookie, inspirational poster)
- [x] Ensure each closing tier has clearly different emotional valence from adjacent tiers
- [x] Confirm all quotes are 1–3 sentences maximum

### Performance Tier Calibration

- [x] Play 5–10 test games and record scores
- [x] Review score distribution; confirm `THEORETICAL_MAX_SCORE` placeholder (1200) is reasonable
  or adjust the constant
- [x] Update the tier threshold percentages in `src/state/quotes.rs` if needed based on real data

### File Creation

- [x] Create `assets/grandma_quotes.json` with the final curated quotes
- [x] Run JSON validation check
- [x] Commit `assets/grandma_quotes.json`

---

## Notes & Risks

- **This milestone is creative work.** It cannot be completed by code generation alone. A human
  must review every quote before it ships. The agent generates candidates; the human decides.
- **Voice drift:** Generating quotes in separate sessions risks inconsistent voice. Use the
  voice guide established in the first task as the prompt preamble for every subsequent session.
- **Iteration is expected.** This file can be updated at any time in subsequent milestones as
  the app comes together and quotes feel wrong in context. Mark the milestone complete when the
  first complete draft is committed — revision is ongoing.
- **Grandma does not explain herself.** Any quote that explains its meaning should be cut.
  If the quote needs context to land, it is not the right quote.
- **Score calibration:** The `THEORETICAL_MAX_SCORE` used for tier computation is a placeholder
  until real gameplay data is collected. The tier thresholds can be adjusted without changing the
  quote content — they are separate constants.

---

## Implementation Notes

### Voice development

The character was established through a detailed biographical brief from the project owner rather
than a generic creative brief. Key decisions made during voice development:

- **Survival as the lens, not harshness.** Early drafts risked making Grandma simply blunt.
  The key framing was that everything hard she says is said *because she thinks it will help you
  survive* — not to hurt you. This distinction is the difference between mean and honest, and it
  is documented explicitly in `doc/grandma_soul.md`.

- **"Nice is different than good."** The character was explicitly aligned with this distinction.
  She is not warm. She is good. Warmth-for-its-own-sake was identified as a disqualifying pattern
  for any quote and is listed in the soul document's "what she would never say" section.

- **Spirit over strategy.** Her belief that luck follows spirit, not tactics, is the core of
  her relationship with dice. This came directly from the owner's description of her correcting
  how he rolled — "you're fighting luck." Every quote pool was checked against this lens: does
  this sound like someone who believes dice reveal character?

- **Observation, not instruction.** She states what she sees. She does not advise. Quotes written
  as instructions ("you should...") were rewritten as observations ("you didn't..."). This is
  documented in the generation checklist in `doc/grandma_soul.md`.

### Quote generation

- **Double pass.** An initial set of ~81 quotes was generated, reviewed, and then doubled to
  161 in a second pass. The second pass was specifically asked to find *new angles* rather than
  repeat themes. This ensured variety within each pool without diluting voice consistency.

- **Terse is correct.** Some of the most effective quotes are a single sentence or even a single
  word ("Adequate."). The temptation to add a second sentence for clarity was identified as a
  voice error early and resisted throughout.

- **Sixzee restraint.** The rare all-five-same outcome was the hardest to write. The instinct is
  to make these celebratory. The decision was to make them *witnessing* instead — she notes it,
  she does not perform excitement. "There it is." and "Hm. All five." became the anchors.

- **Scratch as acceptance, not punishment.** The scratch pool was intentionally kept free of
  judgment. Grandma respects the decision to cut a losing round. The tone is quiet acknowledgment
  of a hard choice, not a lecture about failure.

### `doc/grandma_soul.md`

A character soul document was created as an explicit deliverable (not in the original milestone
spec) because the owner intends to generate additional quotes in future sessions. Without a
written voice guide, voice drift across sessions is nearly inevitable. The document covers:
- Worldview, what she respects, what she has no use for
- Voice register, vocabulary, rhythm with concrete examples
- An 8-question generation checklist
- Per-tier emotional tone descriptions

This document is referenced in `.github/copilot-instructions.md` and should be loaded as a
prompt preamble in any future quote-generation session.

### Score calibration

The original `THEORETICAL_MAX_SCORE` placeholder of 1200 was discovered to be **below the
median game score** from simulation — it would have placed most games in the `great` tier,
making Grandma's highest praise the default experience.

Calibration was done via a Python simulation (5,000 games, greedy strategy):

| Percentile | Score |
|------------|-------|
| min        |   683 |
| 10th       |   882 |
| median     | 1,005 |
| 90th       | 1,146 |
| 95th       | 1,190 |
| max        | 1,681 |

The original symmetric 20/40/60/80% thresholds were replaced with asymmetric 48/57/67/80%
thresholds to match the actual score distribution (which is tightly clustered around 950–1150,
not spread evenly across a wide range). Final values:

- `THEORETICAL_MAX_SCORE = 1500`
- `great ≥ 1200` · `good 1005–1199` · `ok 855–1004` · `bad 720–854` · `really_bad < 720`

The design intent is that median play earns "Adequate" from Grandma (`ok`), which fits her
character — she is not easily impressed. Exceptional play (~top 15% for a skilled player)
earns `great`.
