use leptos::prelude::*;
use crate::{
    state::{game::GameState, piece::{Color, GamePhase}},
};

/// Side panel: turn indicator, captured pieces, move history.
#[component]
pub fn InfoPanel() -> impl IntoView {
    view! {
        <div class="info-panel">
            <TurnSection />
            <CapturedSection />
            <MoveHistorySection />
        </div>
    }
}

#[component]
fn TurnSection() -> impl IntoView {
    let game = expect_context::<GameState>();

    let turn_text = move || {
        let color = game.active_color.get();
        let phase = game.phase.get();
        let name = game.player_name.get();
        let player_color = game.player_color.get();

        let whose = if color == player_color { name } else { "Engine".to_string() };

        match phase {
            GamePhase::Check => format!("{whose} (CHECK!)"),
            GamePhase::Playing => format!("{whose}'s turn"),
            _ => String::new(),
        }
    };

    let dot_class = move || {
        match game.active_color.get() {
            Color::White => "turn-dot white",
            Color::Black => "turn-dot black",
        }
    };

    let is_check = move || game.phase.get() == GamePhase::Check;

    view! {
        <div class="info-section">
            <h3>"Turn"</h3>
            <div class="turn-indicator">
                <div class=dot_class />
                <span>{turn_text}</span>
            </div>
            <Show when=is_check>
                <div class="check-badge">"⚠ CHECK"</div>
            </Show>
        </div>
    }
}

#[component]
fn CapturedSection() -> impl IntoView {
    let game = expect_context::<GameState>();

    let white_captured = move || {
        let mut pieces = game.captured_white.get();
        pieces.sort_by_key(|p| -(p.kind.value()));
        pieces.iter().map(|p| p.glyph().to_string()).collect::<Vec<_>>().join("")
    };

    let black_captured = move || {
        let mut pieces = game.captured_black.get();
        pieces.sort_by_key(|p| -(p.kind.value()));
        pieces.iter().map(|p| p.glyph().to_string()).collect::<Vec<_>>().join("")
    };

    view! {
        <div class="info-section">
            <h3>"Captured"</h3>
            <div style="font-size:0.75rem; color:var(--text-dim);">"White took:"</div>
            <div class="captured-pieces">{white_captured}</div>
            <div style="font-size:0.75rem; color:var(--text-dim); margin-top:0.4rem;">"Black took:"</div>
            <div class="captured-pieces">{black_captured}</div>
        </div>
    }
}

#[component]
fn MoveHistorySection() -> impl IntoView {
    let game = expect_context::<GameState>();

    view! {
        <div class="info-section" style="flex:1">
            <h3>"Moves"</h3>
            <div class="move-history">
                {move || {
                    let history = game.move_history.get();
                    let mut pairs: Vec<_> = Vec::new();
                    let mut i = 0;
                    let mut move_num = 1;
                    while i < history.len() {
                        let white_notation = history[i].notation.clone();
                        let black_notation = history.get(i + 1).map(|r| r.notation.clone()).unwrap_or_default();
                        pairs.push((move_num, white_notation, black_notation));
                        move_num += 1;
                        i += 2;
                    }
                    pairs.into_iter().map(|(num, w, b)| view! {
                        <div class="move-pair">
                            <span class="move-number">{num}"."</span>
                            <span class="move-white">{w}</span>
                            <span class="move-black">{b}</span>
                        </div>
                    }).collect_view()
                }}
            </div>
        </div>
    }
}
