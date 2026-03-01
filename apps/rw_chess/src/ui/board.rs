use leptos::prelude::*;
use crate::{
    state::{game::GameState, piece::{Color, Pos}},
    ui::square::SquareView,
};

/// Renders the full 8×8 chess board.
#[component]
pub fn BoardView() -> impl IntoView {
    let game = expect_context::<GameState>();

    let flipped = move || game.player_color.get() == Color::Black;

    // Build all squares; order depends on board orientation
    let squares = move || {
        let flip = flipped();
        let mut ranks: Vec<u8> = (0..8).collect();
        let mut files: Vec<u8> = (0..8).collect();
        if flip {
            ranks.reverse();
        } else {
            ranks.reverse(); // Rank 8 at top visually (rank index 7 first)
        }
        if flip {
            files.reverse();
        }

        let rank_labels: Vec<&'static str> = if flip {
            vec!["1","2","3","4","5","6","7","8"]
        } else {
            vec!["8","7","6","5","4","3","2","1"]
        };
        let file_labels: Vec<&'static str> = if flip {
            vec!["h","g","f","e","d","c","b","a"]
        } else {
            vec!["a","b","c","d","e","f","g","h"]
        };

        (ranks, files, rank_labels, file_labels)
    };

    view! {
        <div class="board-wrapper">
            <div class="board-with-coords">
                // Rank labels on the left
                <div class="rank-labels">
                    {move || {
                        let (_, _, rank_labels, _) = squares();
                        rank_labels.into_iter().map(|l| view! { <span>{l}</span> }).collect_view()
                    }}
                </div>

                // Board column: board + file labels sit in the same column,
                // so file-labels always matches the board width exactly.
                <div class="board-column">
                    <div class="board">
                        {move || {
                            let flip = flipped();
                            let mut all_squares = Vec::with_capacity(64);
                            let mut ranks: Vec<u8> = (0..8).collect();
                            ranks.reverse();
                            if flip { ranks.reverse(); }

                            let files: Vec<u8> = if flip {
                                (0..8u8).rev().collect()
                            } else {
                                (0..8u8).collect()
                            };

                            for rank in ranks {
                                for &file in &files {
                                    let pos = Pos::new(file, rank);
                                    all_squares.push(view! { <SquareView pos=pos flipped=flip /> });
                                }
                            }
                            all_squares
                        }}
                    </div>

                    <div class="file-labels">
                        {move || {
                            let (_, _, _, file_labels) = squares();
                            file_labels.into_iter().map(|l| view! { <span>{l}</span> }).collect_view()
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}
