#[cfg(test)]
mod tests {
    use crate::*;

    // -----------------------------
    // Helpers
    // -----------------------------

    fn state_with(mines: &[(usize, usize)]) -> MinesState {
        let mut state = MinesState::new();
        place_mines_at(&mut state, mines);
        state
    }

    // -----------------------------
    // Adjacency
    // -----------------------------

    #[test]
    fn counts_adjacent_mines() {
        let state = state_with(&[(0, 0), (0, 2), (2, 2)]);
        assert_eq!(adjacent_mines(&state.cells, 1, 1), 3);
        assert_eq!(adjacent_mines(&state.cells, 0, 1), 2);
        assert_eq!(adjacent_mines(&state.cells, 9, 9), 0);
    }

    #[test]
    fn corner_has_three_neighbours() {
        assert_eq!(neighbours(0, 0).len(), 3);
        assert_eq!(neighbours(0, 5).len(), 5);
        assert_eq!(neighbours(5, 5).len(), 8);
    }

    // -----------------------------
    // Reveal
    // -----------------------------

    #[test]
    fn revealing_a_mine_reports_hit() {
        let mut state = state_with(&[(4, 4)]);
        assert!(reveal(&mut state, 4, 4));
        assert!(state.cells[4][4].revealed);
    }

    #[test]
    fn revealing_a_numbered_cell_reveals_only_it() {
        let mut state = state_with(&[(0, 0)]);
        assert!(!reveal(&mut state, 1, 1));
        assert!(state.cells[1][1].revealed);
        assert_eq!(revealed_count(&state.cells), 1);
    }

    #[test]
    fn revealing_a_zero_cell_floods() {
        // One mine in the corner: revealing the far corner should flood
        // everything except the mine.
        let mut state = state_with(&[(0, 0)]);
        assert!(!reveal(&mut state, 9, 9));
        assert_eq!(revealed_count(&state.cells), ROWS * COLS - 1);
        assert!(!state.cells[0][0].revealed);
        assert!(all_safe_revealed(&state.cells));
    }

    #[test]
    fn flood_stops_at_numbered_border() {
        // A wall of mines down column 5 splits the board; flooding the left
        // side must not reveal anything right of the wall.
        let wall: Vec<(usize, usize)> = (0..ROWS).map(|r| (r, 5)).collect();
        let mut state = state_with(&wall);
        assert!(!reveal(&mut state, 0, 0));
        assert!(state.cells[0][4].revealed);
        assert!(!state.cells[0][6].revealed);
    }

    // -----------------------------
    // Flags
    // -----------------------------

    #[test]
    fn flagged_cell_cannot_be_revealed() {
        let mut state = state_with(&[(4, 4)]);
        toggle_flag(&mut state, 4, 4);
        assert!(!reveal(&mut state, 4, 4));
        assert!(!state.cells[4][4].revealed);
    }

    #[test]
    fn flag_toggles_and_ignores_revealed_cells() {
        let mut state = state_with(&[(0, 0)]);
        toggle_flag(&mut state, 5, 5);
        assert!(state.cells[5][5].flagged);
        toggle_flag(&mut state, 5, 5);
        assert!(!state.cells[5][5].flagged);

        reveal(&mut state, 1, 1);
        toggle_flag(&mut state, 1, 1);
        assert!(!state.cells[1][1].flagged);
        assert_eq!(flag_count(&state.cells), 0);
    }

    // -----------------------------
    // Win condition
    // -----------------------------

    #[test]
    fn win_when_every_safe_cell_is_revealed() {
        let mut state = state_with(&[(0, 0)]);
        assert!(!all_safe_revealed(&state.cells));
        reveal(&mut state, 9, 9);
        assert!(all_safe_revealed(&state.cells));
    }

    // -----------------------------
    // Mouse mapping
    // -----------------------------

    #[test]
    fn cell_at_maps_screen_to_grid() {
        // Board centered on an 800x800 screen; probe the middle of a few tiles.
        let probe = |r: usize, c: usize| {
            let ox = (800.0 - super::super::BOARD_W) / 2.0;
            let oy = (800.0 - super::super::BOARD_H) / 2.0;
            let x = ox + 4.0 + c as f32 * 48.0 + 22.0;
            let y = oy + 4.0 + r as f32 * 48.0 + 22.0;
            cell_at(x, y, 800.0, 800.0)
        };
        assert_eq!(probe(0, 0), Some((0, 0)));
        assert_eq!(probe(9, 9), Some((9, 9)));
        assert_eq!(probe(3, 7), Some((3, 7)));
        assert_eq!(cell_at(0.0, 0.0, 800.0, 800.0), None);
    }
}
