#[cfg(test)]
mod tests {
    use shared_v2::{GameState, Input, World};

    use crate::*;

    // -----------------------------
    // Helpers
    // -----------------------------

    fn simon_world_with(sequence: Vec<usize>) -> World {
        let mut world = World::new();
        spawn_initial(&mut world);
        let simon = world.get_resource_mut::<SimonState>().unwrap();
        simon.sequence = sequence;
        world
    }

    fn press(quadrant: usize) -> Input {
        Input {
            dt: 1.0 / 60.0,
            up: quadrant == 0,
            right: quadrant == 1,
            left: quadrant == 2,
            down: quadrant == 3,
            ..Default::default()
        }
    }

    fn step(dt: f32) -> Input {
        Input {
            dt,
            ..Default::default()
        }
    }

    // -----------------------------
    // Showing Phase Tests
    // -----------------------------

    #[test]
    fn shows_each_step_then_listens() {
        let mut world = simon_world_with(vec![2]);
        let mut state = GameState::new();

        // First tick past the timer flashes the step.
        show_system(&mut world, &mut state, &step(SHOW_TIME + 0.01));
        {
            let simon = world.get_resource::<SimonState>().unwrap();
            assert_eq!(simon.flash.map(|(q, _)| q), Some(2));
            assert_eq!(simon.phase, Phase::Showing);
        }

        // Next tick past the timer switches to listening.
        show_system(&mut world, &mut state, &step(SHOW_TIME + 0.01));
        let simon = world.get_resource::<SimonState>().unwrap();
        assert_eq!(simon.phase, Phase::Listening);
        assert_eq!(simon.input_index, 0);
    }

    #[test]
    fn listening_ignored_while_showing() {
        let mut world = simon_world_with(vec![1]);
        let mut state = GameState::new();

        listen_system(&mut world, &mut state, &press(3));

        assert!(!state.game_over);
        let simon = world.get_resource::<SimonState>().unwrap();
        assert_eq!(simon.input_index, 0);
    }

    // -----------------------------
    // Listening Phase Tests
    // -----------------------------

    fn listening_world(sequence: Vec<usize>) -> World {
        let mut world = simon_world_with(sequence);
        let simon = world.get_resource_mut::<SimonState>().unwrap();
        simon.phase = Phase::Listening;
        world
    }

    #[test]
    fn correct_press_advances() {
        let mut world = listening_world(vec![1, 3]);
        let mut state = GameState::new();

        listen_system(&mut world, &mut state, &press(1));

        assert!(!state.game_over);
        let simon = world.get_resource::<SimonState>().unwrap();
        assert_eq!(simon.input_index, 1);
        assert_eq!(simon.phase, Phase::Listening);
    }

    #[test]
    fn wrong_press_ends_game() {
        let mut world = listening_world(vec![1, 3]);
        let mut state = GameState::new();

        listen_system(&mut world, &mut state, &press(0));

        assert!(state.game_over);
    }

    #[test]
    fn completing_sequence_scores_and_replays() {
        let mut world = listening_world(vec![2]);
        let mut state = GameState::new();

        listen_system(&mut world, &mut state, &press(2));

        assert_eq!(state.score, 1.0);
        let simon = world.get_resource::<SimonState>().unwrap();
        assert_eq!(simon.sequence.len(), 2);
        assert_eq!(simon.phase, Phase::Showing);
        assert_eq!(simon.show_index, 0);
    }

    // -----------------------------
    // Flash Tests
    // -----------------------------

    #[test]
    fn flash_fades_out() {
        let mut world = simon_world_with(vec![0]);
        let mut state = GameState::new();

        world.get_resource_mut::<SimonState>().unwrap().flash = Some((0, 0.01));
        flash_system(&mut world, &mut state, &step(0.02));

        let simon = world.get_resource::<SimonState>().unwrap();
        assert!(simon.flash.is_none());
    }

    #[test]
    fn input_maps_to_quadrants() {
        assert_eq!(pressed_quadrant(&press(0)), Some(0));
        assert_eq!(pressed_quadrant(&press(1)), Some(1));
        assert_eq!(pressed_quadrant(&press(2)), Some(2));
        assert_eq!(pressed_quadrant(&press(3)), Some(3));
        assert_eq!(pressed_quadrant(&step(0.016)), None);
    }
}
