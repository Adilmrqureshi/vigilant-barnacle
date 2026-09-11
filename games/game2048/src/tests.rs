#[cfg(test)]
mod tests {
    use shared_v2::{GameState, Input, World};

    use crate::*;

    // -----------------------------
    // Slide/Merge Tests
    // -----------------------------

    #[test]
    fn slides_tiles_together() {
        let (row, score) = slide_row([2, 0, 0, 2]);
        assert_eq!(row, [4, 0, 0, 0]);
        assert_eq!(score, 4.0);
    }

    #[test]
    fn merges_each_pair_only_once() {
        let (row, score) = slide_row([2, 2, 2, 2]);
        assert_eq!(row, [4, 4, 0, 0]);
        assert_eq!(score, 8.0);

        let (row, _) = slide_row([4, 2, 2, 0]);
        assert_eq!(row, [4, 4, 0, 0]);
    }

    #[test]
    fn does_not_merge_through_gaps_of_different_values() {
        let (row, score) = slide_row([2, 4, 2, 0]);
        assert_eq!(row, [2, 4, 2, 0]);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn moves_work_in_all_directions() {
        let mut cells = [[0; 4]; 4];
        cells[0][0] = 2;
        cells[0][3] = 2;

        let mut right = cells;
        apply_move(&mut right, Dir::Right);
        assert_eq!(right[0], [0, 0, 0, 4]);

        let mut left = cells;
        apply_move(&mut left, Dir::Left);
        assert_eq!(left[0], [4, 0, 0, 0]);

        let mut down = cells;
        apply_move(&mut down, Dir::Down);
        assert_eq!(down[3][0], 2);
        assert_eq!(down[3][3], 2);
    }

    #[test]
    fn unmoved_grid_reports_no_change() {
        let mut cells = [[0; 4]; 4];
        cells[0][0] = 2;

        let (score, moved) = apply_move(&mut cells, Dir::Left);
        assert_eq!(score, 0.0);
        assert!(!moved);
    }

    // -----------------------------
    // Board State Tests
    // -----------------------------

    #[test]
    fn spawn_fills_an_empty_cell() {
        let mut cells = [[0; 4]; 4];
        spawn_tile(&mut cells);

        let filled: u32 = cells.iter().flatten().filter(|v| **v != 0).count() as u32;
        assert_eq!(filled, 1);
    }

    #[test]
    fn detects_when_no_moves_remain() {
        // Checkerboard of alternating values: full, no merges.
        let mut cells = [[0; 4]; 4];
        for r in 0..4 {
            for c in 0..4 {
                cells[r][c] = if (r + c) % 2 == 0 { 2 } else { 4 };
            }
        }
        assert!(!has_moves(&cells));

        cells[0][0] = 4; // now two 4s are adjacent
        assert!(has_moves(&cells));
    }

    // -----------------------------
    // System Tests
    // -----------------------------

    #[test]
    fn move_system_scores_and_spawns() {
        let mut world = World::new();
        let mut cells = [[0; 4]; 4];
        cells[0][0] = 2;
        cells[0][1] = 2;
        world.insert_resource(Board2048 { cells });
        let mut state = GameState::new();

        let input = Input {
            left: true,
            dt: 1.0 / 60.0,
            ..Default::default()
        };
        move_system(&mut world, &mut state, &input);

        let board = world.get_resource::<Board2048>().unwrap();
        assert_eq!(board.cells[0][0], 4);
        assert_eq!(state.score, 4.0);
        // The merge left one tile; a new one spawned somewhere.
        let filled = board.cells.iter().flatten().filter(|v| **v != 0).count();
        assert_eq!(filled, 2);
    }

    #[test]
    fn ineffective_move_does_nothing() {
        let mut world = World::new();
        let mut cells = [[0; 4]; 4];
        cells[0][0] = 2;
        world.insert_resource(Board2048 { cells });
        let mut state = GameState::new();

        let input = Input {
            left: true,
            dt: 1.0 / 60.0,
            ..Default::default()
        };
        move_system(&mut world, &mut state, &input);

        let board = world.get_resource::<Board2048>().unwrap();
        let filled = board.cells.iter().flatten().filter(|v| **v != 0).count();
        assert_eq!(filled, 1); // no spawn on a move that changed nothing
        assert_eq!(state.score, 0.0);
    }
}
