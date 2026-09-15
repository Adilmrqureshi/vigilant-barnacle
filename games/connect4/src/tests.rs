#[cfg(test)]
mod tests {
    use crate::*;

    // -----------------------------
    // Helpers
    // -----------------------------

    fn empty() -> Grid {
        [[None; COLS]; ROWS]
    }

    // Builds a grid by dropping pieces column by column: (col, piece).
    fn grid_of(moves: &[(usize, Piece)]) -> Grid {
        let mut grid = empty();
        for &(col, piece) in moves {
            drop_piece(&mut grid, col, piece).expect("column full in test setup");
        }
        grid
    }

    use Piece::{Cpu, Human};

    // -----------------------------
    // Dropping
    // -----------------------------

    #[test]
    fn pieces_stack_from_the_bottom() {
        let mut grid = empty();
        assert_eq!(drop_piece(&mut grid, 3, Human), Some(ROWS - 1));
        assert_eq!(drop_piece(&mut grid, 3, Cpu), Some(ROWS - 2));
        assert_eq!(grid[ROWS - 1][3], Some(Human));
        assert_eq!(grid[ROWS - 2][3], Some(Cpu));
    }

    #[test]
    fn full_column_rejects_drops() {
        let mut grid = empty();
        for _ in 0..ROWS {
            assert!(drop_piece(&mut grid, 0, Human).is_some());
        }
        assert_eq!(drop_piece(&mut grid, 0, Human), None);
        assert_eq!(drop_piece(&mut grid, COLS, Human), None);
    }

    // -----------------------------
    // Win detection
    // -----------------------------

    #[test]
    fn detects_horizontal_win() {
        let grid = grid_of(&[(0, Human), (1, Human), (2, Human), (3, Human)]);
        assert_eq!(find_winner(&grid), Some(Human));
    }

    #[test]
    fn detects_vertical_win() {
        let grid = grid_of(&[(5, Cpu), (5, Cpu), (5, Cpu), (5, Cpu)]);
        assert_eq!(find_winner(&grid), Some(Cpu));
    }

    #[test]
    fn detects_diagonal_win() {
        // Rising diagonal for Human on columns 0-3.
        let grid = grid_of(&[
            (0, Human),
            (1, Cpu),
            (1, Human),
            (2, Cpu),
            (2, Cpu),
            (2, Human),
            (3, Cpu),
            (3, Cpu),
            (3, Cpu),
            (3, Human),
        ]);
        assert_eq!(find_winner(&grid), Some(Human));
    }

    #[test]
    fn no_winner_on_empty_board() {
        assert_eq!(find_winner(&empty()), None);
        assert!(!is_full(&empty()));
    }

    // -----------------------------
    // AI
    // -----------------------------

    #[test]
    fn ai_takes_an_immediate_win() {
        // CPU has three in a row on the bottom at columns 0-2.
        let grid = grid_of(&[
            (0, Cpu),
            (1, Cpu),
            (2, Cpu),
            (0, Human),
            (1, Human),
        ]);
        let col = best_move(&grid, AI_DEPTH);
        let mut next = grid;
        drop_piece(&mut next, col, Cpu);
        assert_eq!(find_winner(&next), Some(Cpu), "AI played column {col}");
    }

    #[test]
    fn ai_blocks_an_immediate_human_win() {
        // Bottom row: Cpu at 1, Human at 2-4. The only human win square is
        // column 5 (column 1 kills the other window), so the AI must block it.
        let grid = grid_of(&[
            (0, Cpu),
            (1, Cpu),
            (2, Human),
            (3, Human),
            (4, Human),
        ]);
        let col = best_move(&grid, AI_DEPTH);
        assert_eq!(col, 5, "AI played column {col} instead of blocking");
    }

    #[test]
    fn ai_prefers_winning_over_blocking() {
        // The CPU has a vertical three at column 6 and the human threatens
        // the bottom row at column 3. Winning at 6 beats blocking at 3.
        let grid = grid_of(&[
            (6, Cpu),
            (6, Cpu),
            (6, Cpu),
            (0, Human),
            (1, Human),
            (2, Human),
        ]);
        let col = best_move(&grid, AI_DEPTH);
        assert_eq!(col, 6, "AI must take the winning column");
    }

    // -----------------------------
    // Mouse mapping
    // -----------------------------

    #[test]
    fn column_at_maps_screen_to_columns() {
        let screen_w = 1000.0;
        let ox = (screen_w - super::super::BOARD_W) / 2.0;
        assert_eq!(column_at(ox + 8.0 + 30.0, screen_w), Some(0));
        assert_eq!(column_at(ox + super::super::BOARD_W - 30.0, screen_w), Some(COLS - 1));
        assert_eq!(column_at(10.0, screen_w), None);
    }
}
