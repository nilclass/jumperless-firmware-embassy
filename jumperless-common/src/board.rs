#[cfg(feature = "board-v4")]
mod v4;
#[cfg(feature = "board-v4")]
pub use v4::{init_board, Board, BoardSpec, Node};

#[cfg(feature = "board-v5")]
mod v5;
#[cfg(feature = "board-v5")]
pub use v5::{init_board, Board, BoardSpec, Node};
