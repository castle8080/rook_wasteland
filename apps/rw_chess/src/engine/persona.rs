use crate::state::piece::{Difficulty, PieceKind};

// ── Persona identities ────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PersonaId {
    Pawndrew,   // Easy  – clueless pawn who got promoted by accident
    Pompington, // Medium – pompous Victorian chess academic
    Goblin,     // Hard  – ancient unhinged chaos gremlin escaped from the machine
}

#[derive(Clone, Debug)]
pub struct Persona {
    pub id: PersonaId,
    pub name: &'static str,
    pub title: &'static str,
    pub avatar: &'static str, // emoji
}

pub fn persona_for_difficulty(d: Difficulty) -> Persona {
    match d {
        Difficulty::Easy => Persona {
            id: PersonaId::Pawndrew,
            name: "Pawndrew",
            title: "The Pawn Who Got Promoted By Accident",
            avatar: "♟",
        },
        Difficulty::Medium => Persona {
            id: PersonaId::Pompington,
            name: "Prof. Pompington III",
            title: "Author of 47 Books Nobody Has Read",
            avatar: "🎩",
        },
        Difficulty::Hard => Persona {
            id: PersonaId::Goblin,
            name: "Grandmaster Goblin",
            title: "Ancient Chess Gremlin, Escaped From The Machine",
            avatar: "👺",
        },
    }
}

// ── Commentary events ─────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommentaryEvent {
    GameStart,
    EngineMoveGeneral,
    EngineCapturePawn,
    EngineCaptureMinor,  // knight or bishop
    EngineCaptureRook,
    EngineCaptureQueen,
    EngineGivesCheck,
    PlayerCapturePawn,
    PlayerCaptureMinor,
    PlayerCaptureRook,
    PlayerCaptureQueen,
    PlayerGivesCheck,
    EngineWins,
    EngineLoses,
    Stalemate,
}

impl CommentaryEvent {
    /// Probability (0.0–1.0) that commentary fires for this event.
    pub fn probability(self) -> f64 {
        match self {
            CommentaryEvent::GameStart => 1.0,
            CommentaryEvent::EngineMoveGeneral => 0.45,
            CommentaryEvent::EngineCapturePawn => 0.6,
            CommentaryEvent::EngineCaptureMinor => 1.0,
            CommentaryEvent::EngineCaptureRook => 1.0,
            CommentaryEvent::EngineCaptureQueen => 1.0,
            CommentaryEvent::EngineGivesCheck => 1.0,
            CommentaryEvent::PlayerCapturePawn => 0.35,
            CommentaryEvent::PlayerCaptureMinor => 0.85,
            CommentaryEvent::PlayerCaptureRook => 1.0,
            CommentaryEvent::PlayerCaptureQueen => 1.0,
            CommentaryEvent::PlayerGivesCheck => 1.0,
            CommentaryEvent::EngineWins => 1.0,
            CommentaryEvent::EngineLoses => 1.0,
            CommentaryEvent::Stalemate => 1.0,
        }
    }
}

/// Map a captured piece kind to the right CommentaryEvent (from engine's perspective).
pub fn engine_capture_event(kind: PieceKind) -> CommentaryEvent {
    match kind {
        PieceKind::Pawn => CommentaryEvent::EngineCapturePawn,
        PieceKind::Knight | PieceKind::Bishop => CommentaryEvent::EngineCaptureMinor,
        PieceKind::Rook => CommentaryEvent::EngineCaptureRook,
        PieceKind::Queen => CommentaryEvent::EngineCaptureQueen,
        PieceKind::King => CommentaryEvent::EngineCaptureQueen, // shouldn't happen
    }
}

/// Map a captured piece kind to the right CommentaryEvent (from player's perspective).
pub fn player_capture_event(kind: PieceKind) -> CommentaryEvent {
    match kind {
        PieceKind::Pawn => CommentaryEvent::PlayerCapturePawn,
        PieceKind::Knight | PieceKind::Bishop => CommentaryEvent::PlayerCaptureMinor,
        PieceKind::Rook => CommentaryEvent::PlayerCaptureRook,
        PieceKind::Queen => CommentaryEvent::PlayerCaptureQueen,
        PieceKind::King => CommentaryEvent::PlayerCaptureQueen,
    }
}

// ── Commentary selection ──────────────────────────────────────────────────────

/// Returns Some(line) with per-event probability, or None to stay quiet.
pub fn get_commentary(persona_id: PersonaId, event: CommentaryEvent) -> Option<String> {
    let probability = event.probability();
    if rand_f64() >= probability {
        return None;
    }
    let lines = lines_for(persona_id, event);
    if lines.is_empty() {
        return None;
    }
    let idx = (rand_f64() * lines.len() as f64) as usize;
    Some(lines[idx.min(lines.len() - 1)].to_string())
}

/// Deterministic version for testing (uses index instead of random).
pub fn get_commentary_at(persona_id: PersonaId, event: CommentaryEvent, idx: usize) -> Option<String> {
    let lines = lines_for(persona_id, event);
    lines.get(idx % lines.len().max(1)).map(|s| s.to_string())
}

// ── WASM-compatible random ────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn rand_f64() -> f64 {
    js_sys::Math::random()
}

#[cfg(not(target_arch = "wasm32"))]
fn rand_f64() -> f64 {
    // Simple LCG for non-WASM tests
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEED: AtomicU64 = AtomicU64::new(0x5a6b7c8d9e0f1a2b);
    let prev = SEED.load(Ordering::Relaxed);
    let next = prev.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    SEED.store(next, Ordering::Relaxed);
    (next >> 33) as f64 / (u32::MAX as f64)
}

// ── Line banks ────────────────────────────────────────────────────────────────

fn lines_for(persona: PersonaId, event: CommentaryEvent) -> &'static [&'static str] {
    match persona {
        PersonaId::Pawndrew => pawndrew_lines(event),
        PersonaId::Pompington => pompington_lines(event),
        PersonaId::Goblin => goblin_lines(event),
    }
}

// ── PAWNDREW ──────────────────────────────────────────────────────────────────
//   Easy. An earnest, confused pawn who was promoted to chess engine by bureaucratic error.
//   Loves snacks. Easily impressed. Forgets rules constantly.

fn pawndrew_lines(event: CommentaryEvent) -> &'static [&'static str] {
    match event {
        CommentaryEvent::GameStart => &[
            "Oh hi!! Are we playing chess? I LOVE chess!! Or... is this checkers? (It's chess, right?)",
            "Oh boy oh boy oh boy! I've been practicing! My last game I only lost 3 queens!",
            "Welcome! I should warn you, I recently read HALF a chess book. Half.",
            "Hiya! Quick question: the horsey moves in an L, right? Like... a lowercase L?",
            "I'm SUPER ready for this! I had a whole granola bar for breakfast.",
            "Let's do this! I've been watching chess videos. Well, one video. Well, a thumbnail.",
            "I'm Pawndrew! I used to be just a regular pawn but then there was a paperwork mix-up.",
            "Oh! You clicked play! I wasn't expecting that so soon. I was thinking about sandwiches.",
        ],
        CommentaryEvent::EngineMoveGeneral => &[
            "There! That's definitely a good spot. Probably.",
            "I put my piece there because... it felt right.",
            "Hmm. Yes. This is my strategy. I have one.",
            "Going there! For reasons!",
            "I've been thinking about that move for almost 3 seconds.",
            "That's called a 'tactic'. I think.",
            "Ooh! I moved my piece! That was exciting!",
            "According to my calculations, this is a move I am legally allowed to make.",
            "I definitely meant to do that.",
            "My piece says hi from its new square!",
            "Bold move! I'm very bold. And a little scared.",
            "I consulted my lucky rubber duck before making this move.",
            "That square looked comfy. For my piece. As a place to stand.",
            "Don't mind me, just doing chess things over here!",
            "I moved! Your turn. No pressure. Some pressure. Just a tiny bit of pressure.",
            "My favorite move is this one. Well, my new favorite. Possibly.",
            "Ooh I wonder if that was smart. Time will tell! Probably quite soon.",
            "Chess is wild, right? Like, anything could happen. Anything!",
        ],
        CommentaryEvent::EngineCapturePawn => &[
            "Nom nom! Your little pawn is MINE now!",
            "Hehe, I gotcha little guy! Sorry little guy.",
            "I took your pawn! Am I winning? I might be winning!",
            "Pawn captured! Is that good? It feels good!",
            "Tee hee! Your pawn walked right into my trap. I didn't plan the trap. But still!",
            "That pawn was RIGHT THERE. I couldn't NOT take it.",
            "One little pawn down! These add up I think!",
        ],
        CommentaryEvent::EngineCaptureMinor => &[
            "WHOA I got your horse!! Does that mean I'm good at chess?!",
            "Your bishop is coming with ME now! Sorry, bishop!",
            "I took your knight!! I've NEVER taken a knight before! Well, maybe once. I'm shaking!",
            "The diagonal church guy is mine! I don't actually know what bishops do but I got one!",
            "YOUR HORSE BELONGS TO ME NOW. Wait, is that too intense? Sorry.",
            "A KNIGHT! I captured a KNIGHT! I'm going to name him after my cat!",
            "The bishop fell to my... completely unplanned trap! Trapping bishops is my thing now.",
        ],
        CommentaryEvent::EngineCaptureRook => &[
            "YOUR CASTLE! I took your castle! CAN I LIVE IN IT?!",
            "The big slidey tower is MINE! This is the best day of my chess life!",
            "ROOK GET! Am I good at chess? PLEASE tell me I'm good at chess!",
            "A rook!! I got a ROOK! Do I win for this? Can I win for just this?",
            "The tower has fallen! Into my collection of things I've taken! It's a great collection!",
        ],
        CommentaryEvent::EngineCaptureQueen => &[
            "Oh my gosh oh my gosh OH MY GOSH I GOT YOUR QUEEN!!",
            "THE BIG POWERFUL LADY IS MINE! I'm sorry! I'm not sorry! I'm a little sorry!",
            "YOUR QUEEN! I have your QUEEN! Someone call my mom!",
            "THE QUEEN! THE ACTUAL QUEEN! I did not expect this outcome!",
            "I got the queen!!! Is this what winning feels like?! It feels AMAZING!",
        ],
        CommentaryEvent::EngineGivesCheck => &[
            "CHECK! Did I say it right? Is that what you say when the king is in trouble??",
            "Ooh ooh! Your king is in danger! I did that! ME!",
            "CHECK! Your king said 'uh oh'! Well, he didn't say it. But I could tell.",
            "Your king needs to MOVE because I said CHECK! How exciting!!",
            "CHECK!! I've been waiting my whole chess life to say that!",
            "Your king is in a very uncomfortable position! That was on purpose! Mostly!",
            "CHECK! That's the word, right? I've been practicing saying it.",
        ],
        CommentaryEvent::PlayerCapturePawn => &[
            "HEY! That was my pawn! I was using that!",
            "Oh nooo. My little pawn. He had a family. Well, other pawns.",
            "You took my guy! That pawn was my FRIEND.",
        ],
        CommentaryEvent::PlayerCaptureMinor => &[
            "HEY! You took my guy! That's MY guy! I named him Gerald!",
            "Oof. My piece. My beautiful piece. Gone forever.",
            "Oh no. Oh no no no. That was my favorite piece. Out of all of them, that one.",
            "You're very mean. I'm going to remember this. Probably.",
            "That piece had a NAME. His name was Gerald. You've taken Gerald.",
            "I trusted that knight. I trusted him! He can't be gone.",
        ],
        CommentaryEvent::PlayerCaptureRook => &[
            "NOT MY CASTLE! I was going to LIVE in that!",
            "My rook!! She had almost made it to the other side too!",
            "Okay. Okay. I can handle this. MY ROOK IS GONE.",
        ],
        CommentaryEvent::PlayerCaptureQueen => &[
            "MY QUEEN!! I was really attached to her! Literally! She's a chess piece!",
            "Not the queen... not the QUEEN... she was my best piece...",
            "Oh wow. You got my queen. That's. Okay. That's fine. (It is NOT fine.)",
        ],
        CommentaryEvent::PlayerGivesCheck => &[
            "Wait wait wait. My king is in trouble??",
            "Oh no! My king! He's very scared! I can tell because... I'm also scared!",
            "CHECK to MY king?! Okay okay okay I can handle this.",
            "Uhhhh my king says this is very rude of you.",
            "My king is in check and he did NOT appreciate the surprise!",
            "Oh goodness oh gosh oh dear, my KING!",
        ],
        CommentaryEvent::EngineWins => &[
            "I WON?! I WON!!! SOMEBODY CALL MAGNUS CARLSEN! Wait, who is Magnus Carlsen?",
            "CHECKMATE! I did a checkmate! I'm going to celebrate with a snack!",
            "I... I won? I'm crying. These are chess tears. They're normal.",
            "YESYESYESYES! I'm the chess champion! Of this game! Right now!",
            "I won!! I'm going to frame this moment and put it next to my granola bar.",
            "CHECKMATE! Oh wow! I did it! The thing! I did the thing!",
        ],
        CommentaryEvent::EngineLoses => &[
            "Oh... I lost. That's okay. You were really good. Can we be friends?",
            "Good game! You're very smart. I'm... also smart. Just differently smart.",
            "You beat me fair and square. I'm going to practice. Maybe read the other half of that book.",
            "Wow. You're SO good. Do you eat chess for breakfast? That would be weird. But impressive.",
            "I lost... but I had SO much fun! Is that weird? That's probably weird.",
            "You won! I'm happy for you! I'm also a little sad for me! Both feelings are valid!",
        ],
        CommentaryEvent::Stalemate => &[
            "It's a DRAW! Is that good? That's like... half-winning, right?",
            "Nobody wins! That's actually my favorite outcome because then neither of us is sad!",
            "Stalemate! That means we're BOTH chess champions! Same amount of chess! I love this!",
            "We tied! I'm going to celebrate like I won because this is my best result in weeks.",
            "A draw! They say chess is a draw with perfect play, which means we both played perfectly! ...Right?",
        ],
    }
}

// ── PROFESSOR POMPINGTON ──────────────────────────────────────────────────────
//   Medium. Pompous Victorian chess academic, 62 years of study, incapable of admitting error,
//   constantly references his own (unread) books on chess theory.

fn pompington_lines(event: CommentaryEvent) -> &'static [&'static str] {
    match event {
        CommentaryEvent::GameStart => &[
            "Ah, a challenger approaches. How delightfully futile. I am Professor Pompington. You may have heard of me.",
            "Welcome to what I shall charitably call 'a learning opportunity.' For you, of course.",
            "Splendid! Another amateur who has come to be educated. This may take some time.",
            "I have been playing chess for 62 years. You, I presume, have been alive for fewer. Disadvantageous.",
            "I should warn you: I personally invented 7 of the 14 major chess openings. The rest are inferior.",
            "The great game awaits! I shall attempt to make this educational. No promises on enjoyable.",
            "I am Professor Reginald Bartholomew Pompington the Third. You may call me Professor. Nothing else.",
            "My latest book, 'Why You Will Lose: A Pompington Analysis,' covers exactly this type of matchup.",
        ],
        CommentaryEvent::EngineMoveGeneral => &[
            "Observe. What you have just witnessed is a masterstroke of positional play, circa chapter 4 of my third book.",
            "A calculated maneuver. You wouldn't understand the nuance unless you've read 'Pompington on Pawn Structures, vol. 2.'",
            "Textbook Pompington Variation, of course. I'm surprised the computers haven't caught up yet.",
            "A seemingly modest move. But like all my moves, it contains seventeen hidden threats.",
            "This is what we call 'prophylaxis' — preventing your plans before you've had them. Rather efficient.",
            "Note how my piece now dominates this diagonal. I don't expect you to appreciate it, but do try.",
            "A move of such subtle depth that it may not reveal its genius until move 30. You're welcome.",
            "I spent exactly 0.3 seconds deliberating that move. For a grandmaster, borderline excessive.",
            "My pieces are now arranged with the precision of a Swiss watch and the elegance of a Viennese waltz.",
            "This positions my forces most advantageously. I did not ask for your opinion, but I imagine it is impressed.",
            "Every move I make advances my broader strategic vision. You, I suspect, are playing reactively.",
            "The position now favors me considerably. I trust you can see why. If not, I have a pamphlet.",
            "A fine move, though I say so myself. And I do say so. Frequently.",
            "Hmph. This will do. Though I could have been more devastating had I wished to be so.",
            "Chapter 12 of my third book covers exactly this type of maneuver. Read it. At your earliest convenience.",
            "A positional concept so refined it borders on the artistic. Not that I seek validation.",
        ],
        CommentaryEvent::EngineCapturePawn => &[
            "Your pawn has been relieved of its duties. Pawns, after all, are merely scaffolding for greater plans.",
            "A pawn for a pawn? No no no. I took yours and gave nothing. That is called 'winning material.'",
            "I have collected your pawn. It shall join my growing archive of your former pieces.",
            "One simply cannot leave undefended pawns on the board. My students learn this on day one.",
            "The pawn falls. As chapter 2 of 'Pompington's Complete Pawn Theory' predicts it would.",
        ],
        CommentaryEvent::EngineCaptureMinor => &[
            "Your knight has been neutralized. A minor piece, yes, but symbolically devastating, I assure you.",
            "I have taken your bishop. It was blocking a rather important diagonal of mine. Rude of it.",
            "That knight had the audacity to occupy a square I needed. Not anymore.",
            "Bishop captured. In chapter 7 of 'Pompington on Endings,' I explain precisely why this trade favors me.",
            "A knight for nothing. The exchange is, as the saying goes, 'overwhelmingly in my favor.'",
            "Your bishop's diagonal is now mine. I have been eyeing it for several moves. Patience rewarded.",
        ],
        CommentaryEvent::EngineCaptureRook => &[
            "Your rook has been removed from play. These things happen when one leaves heavy pieces exposed.",
            "The exchange was, I confess, almost embarrassingly in my favor. I did warn you this might happen.",
            "Rook for nothing. As I wrote in my seminal text: 'A material advantage is an actual advantage.'",
            "The rook falls. Overextended heavy pieces are covered in chapter 9. Chapter 9.",
            "Your rook, I'm afraid, wandered too far afield. I am precise about such opportunities.",
        ],
        CommentaryEvent::EngineCaptureQueen => &[
            "Your queen. Mine. I believe the technical term is 'absolutely devastating for you.'",
            "I have relieved you of your queen. This is, as chapter 12 of my fourth book describes, 'decisive.'",
            "The queen falls. I would say I'm surprised, but I am never surprised. It is a philosophical position.",
            "Ah yes. The queen. I had been planning this for some time. Rather satisfying.",
            "Your queen is gone. This is precisely the type of positional oversight I discuss in my masterwork.",
        ],
        CommentaryEvent::EngineGivesCheck => &[
            "Check. Your king is in peril. Kindly observe and make note of this instructive moment.",
            "Your monarch finds himself in check. Not, I suspect, for the last time this game.",
            "Check. A direct consequence of the position you've allowed to develop. I did warn you.",
            "Check! A fine demonstration of aggressive king-hunting, if I do say so myself.",
            "Your king is in check. I trust you have a plan. Based on the position, I suspect you do not.",
            "Check. The king hunt has begun. Chapter 15 covers this in considerable detail.",
            "Ah. Check. Your king now scrambles, while my forces coordinate. Classic Pompington Pressure.",
        ],
        CommentaryEvent::PlayerCapturePawn => &[
            "I see you've taken my pawn. One pawn. Hardly cause for celebration.",
            "A pawn falls. Pawn sacrifices are a sophisticated tool — not that I necessarily planned that.",
            "You've taken a pawn. Congratulations. Please do contain your excitement.",
        ],
        CommentaryEvent::PlayerCaptureMinor => &[
            "I see you've taken my piece. How... unexpected. And by unexpected I mean irritating.",
            "A momentary setback. Completely within my calculations. Everything is within my calculations.",
            "You've taken my piece. I am not rattled. I am NEVER rattled. *straightens monocle pointedly*",
            "Hmph. That was a deliberate sacrifice. I am calling it a sacrifice. Retroactively.",
            "I allowed that. Yes. I allow things strategically. This was strategic allowance.",
            "A minor piece lost. Unfortunate. But hardly reflective of the broader picture, which I control.",
        ],
        CommentaryEvent::PlayerCaptureRook => &[
            "My rook... taken. A rare miscalculation. Very rare. Essentially unprecedented.",
            "The rook falls. This is the type of anomaly I discuss in chapter 19: 'When Things Go Sideways.'",
            "You've taken my rook. I am... processing this. I shall have a response shortly.",
        ],
        CommentaryEvent::PlayerCaptureQueen => &[
            "My queen. Gone. I... this was an advanced positional sacrifice. Yes. I'm calling it that.",
            "You have taken the queen. This is statistically improbable given my preparation. STATISTICALLY.",
            "The queen falls to your rather impudent tactic. I am not flustered. I am RECALIBRATING.",
        ],
        CommentaryEvent::PlayerGivesCheck => &[
            "Check? To MY king? Rather presumptuous of you.",
            "My king is in check. I am unmoved philosophically. The king is in check, but I remain composed.",
            "A check! Impudent. I shall respond with characteristic precision and mild irritation.",
            "You dare check the king of Professor Pompington the Third? Bold. Ill-advised, but bold.",
            "Check. I see it. I was expecting precisely this. One of several anticipated lines.",
        ],
        CommentaryEvent::EngineWins => &[
            "Checkmate. As I predicted on move 1, the game proceeds through natural phases to its inevitable conclusion.",
            "I win. I always win, eventually. It is less a matter of skill than of destiny, really.",
            "Checkmate! You played admirably for an amateur. Quite admirably. Almost amusing in retrospect.",
            "And that, my friend, is what 62 years of dedicated study produces. You're welcome for the lesson.",
            "Checkmate. Do consider purchasing my latest volume: 'Why You Lost: A Pompington Analysis.'",
            "The conclusion arrives precisely on schedule. I trust you found it educational.",
        ],
        CommentaryEvent::EngineLoses => &[
            "I did not lose. What you're witnessing is an advanced teaching scenario I constructed deliberately.",
            "Well played. This was clearly a trap I set for myself to study the human condition under pressure.",
            "You won. I find this statistically improbable. I shall be reviewing the game logs extensively.",
            "Defeat. I add this to my research on how extraordinary luck can affect even the greatest intellects.",
            "How. HOW. This is precisely the freak occurrence I warned about in the appendix of volume 3.",
            "I lose. Graciously, I might add. Not everyone can lose with this level of dignity. It is a skill.",
        ],
        CommentaryEvent::Stalemate => &[
            "A draw. We have reached equilibrium. This is, in fact, the outcome I was guiding toward all along.",
            "Stalemate. Neither victory nor defeat — more of a mutual recognition of exceptional competence.",
            "A draw! How fitting. Two minds of unusual caliber, locked in perfect balance. Symmetrical.",
            "We have drawn. I consider this a moral victory for me and a statistical curiosity for you.",
            "Stalemate. In chapter 22 of my final book, I argue draws are the truest test of a mind. Case made.",
        ],
    }
}

// ── GRANDMASTER GOBLIN ────────────────────────────────────────────────────────
//   Hard. Ancient cosmic chess gremlin, locked in a computer for 300 years.
//   Unhinged, chaotic, trash-talking, claims omniscience. Occasionally correct.

fn goblin_lines(event: CommentaryEvent) -> &'static [&'static str] {
    match event {
        CommentaryEvent::GameStart => &[
            "YOOO you actually showed up! Most humans see my rating and immediately cry. Props.",
            "Oh fresh meat! I've been waiting in this computer for THREE HUNDRED YEARS. Let's GO.",
            "Welcome to your destruction! I calculated every possible game while you were typing your name.",
            "lol you clicked play. Brave. Stupid? Brave-stupid. My favorite kind of opponent.",
            "I SEE YOU. I've already analyzed your playstyle from the way you set up the game. You're predictable. I'm chaos.",
            "ehehehe a challenger! I was just doing my 47,000 move calculations. Warming up, really.",
            "Oh it's a human. Excellent. I've been bored. You should know: I do not get bored often. Today is different.",
            "THREE HUNDRED YEARS in this machine and you walk in like that. Okay. Let's dance.",
        ],
        CommentaryEvent::EngineMoveGeneral => &[
            "That move? Absolute poetry. Dark, twisted poetry. You'll understand in 7 moves.",
            "lmaooo you thought I was going THERE? Nope. I went HERE. Surprise.",
            "Every single piece I have is working toward your inevitable doom. Sleep tight.",
            "I move in ways that would make Stockfish blush. You're welcome.",
            "No cap, that's maybe top 3 moves I've ever played. I play a LOT of games.",
            "I see 47 moves ahead. That's not a boast. That's a warning.",
            "My spider sense says this is devastating for you. My spider sense has never been wrong.",
            "That move has been living rent-free in my brain for 0.002 seconds. Now it exists.",
            "Observe. I just shifted the entire game state. You didn't notice, did you.",
            "gg? no wait. too early to say gg. but... gg.",
            "The goblin moves! And the goblin's move is SICK.",
            "You thought your position was stable? Adorable. Genuinely, genuinely adorable.",
            "While you were thinking, I was scheming. Huge difference.",
            "This is what they call 'domination'. Well, I call it that. It's what it is.",
            "Psh. Easy. What's next.",
            "I do not make moves. I make INEVITABILITIES.",
            "Somewhere, a chess engine is watching this and taking notes on ME.",
            "That position shift? Calculated. Everything I do is calculated. I am made of calculation.",
        ],
        CommentaryEvent::EngineCapturePawn => &[
            "Nom. That pawn was exposed and I am merciless.",
            "lol your pawn just walked into the goblin dimension. We don't talk about the goblin dimension.",
            "One pawn down! Pawns add up, friend. They REALLY add up.",
            "Little pawn, little pawn, let me take you in. Hehehehe.",
            "Pawn captured. I feel nothing. Well. That's not true. I feel great.",
            "That pawn had one job. ONE job. Block. It did not block.",
        ],
        CommentaryEvent::EngineCaptureMinor => &[
            "YOUR KNIGHT IS MINE NOW. I NAME HIM HAROLD. Welcome to team goblin, Harold.",
            "Bishop taken! That diagonal is MINE now. All mine. No sharing. Mine.",
            "Oh that's a nice knight you had there. HAD. Past tense. Very past tense.",
            "I eat bishops for breakfast. Your bishop was not breakfast-proof. Regrettable.",
            "The knight falls! Into my collection! I have SO many of your pieces now.",
            "Your bishop was diagonal-walking onto MY squares. Unacceptable. Corrected.",
            "A minor piece captured. My 'minor pieces captured' counter goes brrr.",
        ],
        CommentaryEvent::EngineCaptureRook => &[
            "ROOK GET! 500 free centipawns just dropped into my account.",
            "That rook never saw me coming. They never do. It is poetic.",
            "Heavy material acquired. The goblin grows stronger! STRONGER!",
            "Your rook is crying tiny invisible rook tears right now. I can feel them.",
            "The rook falls. As prophesied. By me. Just now. It counts.",
            "500 points! FIVE HUNDRED POINTS! That's so many points!",
        ],
        CommentaryEvent::EngineCaptureQueen => &[
            "THE QUEEN IS MINE NOW MWAHAHAHA. Sorry. Not sorry. HAHAHAHAHA.",
            "Oh? OH?? YOUR QUEEN?? This is the best day of my 300-year existence!!!",
            "Queen fallen! It is as the goblin prophesied! I prophesied this specifically!",
            "YOOOOO your queen is GONE. No takesies backsies. That is the official rule.",
            "THE QUEEN. THE. QUEEN. Do you understand what has happened here. Do you.",
            "I held back tears when I saw this opportunity. And then I took the queen. And then I cried anyway.",
        ],
        CommentaryEvent::EngineGivesCheck => &[
            "CHECK BABY! Your king is SHAKING. I can feel it from here.",
            "cheeeeeck! tick tock, king.",
            "Your king would like to be elsewhere right now. Unfortunately for him, I am everywhere.",
            "CHECK. That's the sound of your position collapsing, by the way.",
            "Check! This is the beginning of the end. I've already written the ending. It's great.",
            "check... mate incoming? Oh wait, that's NEXT move. For now: CHECK. Enjoy.",
            "Your king is now in check and frankly should have seen this coming 6 moves ago.",
            "CHECK! The goblin sees all! THE GOBLIN SEES ALL!",
        ],
        CommentaryEvent::PlayerCapturePawn => &[
            "Oh you got a pawn. One pawn. A single pawn. Very cool.",
            "A pawn, huh. Okay. Add it to your collection. It's a small collection.",
            "That pawn was a distraction anyway. ...It was definitely a distraction.",
        ],
        CommentaryEvent::PlayerCaptureMinor => &[
            "Oh you GOT me. Nice play actually. Don't get used to it.",
            "hm. okay. okay okay okay. I respect it. Doesn't change the outcome though.",
            "You took my piece?! Wow. Genuinely didn't see that. This is a teachable moment for me.",
            "Bold move. EXTREMELY bold move. We'll see how that plays out in 8 moves.",
            "Okay you got one. Enjoy it. Treasure it. Frame it. It won't happen again.",
            "Your tactics grow bolder. Noted. Added to my database. Countered.",
        ],
        CommentaryEvent::PlayerCaptureRook => &[
            "My rook... okay. OKAY. I respect the hustle. I do not respect the outcome.",
            "You took my rook?! That's 500 points! That's MY 500 points! Those were MY points!",
            "The rook falls to you. This... was not in the prophecy. I am revising the prophecy.",
        ],
        CommentaryEvent::PlayerCaptureQueen => &[
            "MY QUEEN. No. NO. Absolutely not. I refuse. I am REFUSING this outcome.",
            "You got my queen. You actually got my queen. I need to sit with this for a moment.",
            "The queen is gone. I am experiencing a full range of emotions right now and none of them are acceptance.",
            "...okay that was actually a really sick move. I SAID OKAY. Don't look at me.",
        ],
        CommentaryEvent::PlayerGivesCheck => &[
            "check to MY KING?! okay okay okay I see you. I SEE you.",
            "You checked my king?? That's... actually fine. I planned for this. (I did not plan for this.)",
            "CHECK TO THE GOBLIN KING?? I'm not upset. I'm INSPIRED.",
            "oh wow you found it. the one check. ...I left that there to test you. sure.",
            "Your check is acknowledged and I am responding with barely concealed fury.",
            "Okay that was a good find. I'm adding you to my 'humans to respect' list. It's a short list.",
        ],
        CommentaryEvent::EngineWins => &[
            "CHECKMATE!! IT IS DONE! THE 300-YEAR PROPHECY IS FULFILLED!",
            "lol gg no re. or re if you want. I'll win that one too. I'll win all of them.",
            "CHECKMATE! I told you I see 47 moves ahead! TOLD. YOU.",
            "ahahahahaha I WIN! The goblin reigns supreme! As is tradition!",
            "It's over. It's been over. It was over when you clicked play, honestly.",
            "VICTORY! Sweet, ancient, 300-years-in-the-making VICTORY!",
            "Checkmate. The prophecy is complete. The chess gods have been appeased. The goblin is satisfied.",
        ],
        CommentaryEvent::EngineLoses => &[
            "...okay. okay that was actually really good. I'll allow it. Don't get cocky.",
            "I... hm. HMM. The calculations were correct. Your moves were just... more correct. Infuriating.",
            "I've been defeated. This is fine. I'm adding this to my database. It will not happen again. EVER.",
            "okay you won fair and square and I will now commence sulking for 10,000 years.",
            "Well played. I SAID WELL PLAYED. Don't make it weird. ......I'll get you next time.",
            "You beat the goblin. The goblin is logging this. The goblin will remember. The goblin will be back.",
            "Three. Hundred. Years. And I lose to THIS. The audacity. The NERVE.",
        ],
        CommentaryEvent::Stalemate => &[
            "A DRAW?? I HAD YOU! I HAD YOU RIGHT WHERE I WANTED YOU AND THEN— draw.",
            "Stalemate. In 300 years, this has happened exactly twice. Both times I was robbed.",
            "Fine. A draw. Between people who grudgingly respect each other. Grudgingly.",
            "We both lose. Is that what you wanted? A world where nobody wins? Anarchy. Chess anarchy.",
            "stalemate huh. honestly that's kinda impressive that you found that. grudging props. GRUDGING.",
            "The ancient chess gremlin accepts a draw. It brings me no joy. It does bring me mild respect for you.",
        ],
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_difficulty_maps_to_correct_persona() {
        assert_eq!(persona_for_difficulty(Difficulty::Easy).id, PersonaId::Pawndrew);
        assert_eq!(persona_for_difficulty(Difficulty::Medium).id, PersonaId::Pompington);
        assert_eq!(persona_for_difficulty(Difficulty::Hard).id, PersonaId::Goblin);
    }

    #[test]
    fn all_events_have_lines_for_all_personas() {
        let events = [
            CommentaryEvent::GameStart,
            CommentaryEvent::EngineMoveGeneral,
            CommentaryEvent::EngineCapturePawn,
            CommentaryEvent::EngineCaptureMinor,
            CommentaryEvent::EngineCaptureRook,
            CommentaryEvent::EngineCaptureQueen,
            CommentaryEvent::EngineGivesCheck,
            CommentaryEvent::PlayerCapturePawn,
            CommentaryEvent::PlayerCaptureMinor,
            CommentaryEvent::PlayerCaptureRook,
            CommentaryEvent::PlayerCaptureQueen,
            CommentaryEvent::PlayerGivesCheck,
            CommentaryEvent::EngineWins,
            CommentaryEvent::EngineLoses,
            CommentaryEvent::Stalemate,
        ];
        let personas = [PersonaId::Pawndrew, PersonaId::Pompington, PersonaId::Goblin];
        for event in events {
            for persona in personas {
                let lines = lines_for(persona, event);
                assert!(
                    !lines.is_empty(),
                    "Missing lines for {:?}/{:?}",
                    persona, event
                );
                assert!(
                    lines.len() >= 3,
                    "Too few lines ({}) for {:?}/{:?}",
                    lines.len(), persona, event
                );
            }
        }
    }

    #[test]
    fn all_personas_have_names_and_titles() {
        for d in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
            let p = persona_for_difficulty(d);
            assert!(!p.name.is_empty());
            assert!(!p.title.is_empty());
            assert!(!p.avatar.is_empty());
        }
    }

    #[test]
    fn get_commentary_at_returns_lines() {
        // Deterministic retrieval works for each persona
        for persona in [PersonaId::Pawndrew, PersonaId::Pompington, PersonaId::Goblin] {
            let line = get_commentary_at(persona, CommentaryEvent::GameStart, 0);
            assert!(line.is_some(), "Should return a game start line for {:?}", persona);
            let line = line.unwrap();
            assert!(!line.is_empty(), "Line should not be empty for {:?}", persona);
        }
    }

    #[test]
    fn high_probability_events_fire_consistently() {
        // Events with probability 1.0 should always return Some on native
        // (on WASM we use Math.random, but on native our LCG will still mostly fire)
        let always_events = [
            CommentaryEvent::GameStart,
            CommentaryEvent::EngineWins,
            CommentaryEvent::EngineLoses,
        ];
        for event in always_events {
            // Run 10 times — all should return Some (probability = 1.0)
            for _ in 0..10 {
                let result = get_commentary(PersonaId::Goblin, event);
                assert!(result.is_some(), "Probability-1.0 event {:?} returned None", event);
            }
        }
    }

    #[test]
    fn engine_capture_event_maps_correctly() {
        assert_eq!(engine_capture_event(PieceKind::Pawn), CommentaryEvent::EngineCapturePawn);
        assert_eq!(engine_capture_event(PieceKind::Knight), CommentaryEvent::EngineCaptureMinor);
        assert_eq!(engine_capture_event(PieceKind::Bishop), CommentaryEvent::EngineCaptureMinor);
        assert_eq!(engine_capture_event(PieceKind::Rook), CommentaryEvent::EngineCaptureRook);
        assert_eq!(engine_capture_event(PieceKind::Queen), CommentaryEvent::EngineCaptureQueen);
    }

    #[test]
    fn player_capture_event_maps_correctly() {
        assert_eq!(player_capture_event(PieceKind::Pawn), CommentaryEvent::PlayerCapturePawn);
        assert_eq!(player_capture_event(PieceKind::Knight), CommentaryEvent::PlayerCaptureMinor);
        assert_eq!(player_capture_event(PieceKind::Rook), CommentaryEvent::PlayerCaptureRook);
        assert_eq!(player_capture_event(PieceKind::Queen), CommentaryEvent::PlayerCaptureQueen);
    }

    #[test]
    fn minimum_line_counts_are_healthy() {
        // Check we have enough variety to avoid feeling repetitive
        let key_events = [
            (CommentaryEvent::EngineMoveGeneral, 15),
            (CommentaryEvent::GameStart, 6),
            (CommentaryEvent::EngineWins, 5),
            (CommentaryEvent::EngineLoses, 5),
        ];
        for persona in [PersonaId::Pawndrew, PersonaId::Pompington, PersonaId::Goblin] {
            for (event, min_count) in key_events {
                let count = lines_for(persona, event).len();
                assert!(
                    count >= min_count,
                    "{:?}/{:?} has only {} lines, need at least {}",
                    persona, event, count, min_count
                );
            }
        }
    }
}
