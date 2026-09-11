#[cfg(test)]
mod tests {
    use shared_v2::{GameState, Input, Tag, World};

    use crate::*;

    // -----------------------------
    // Helpers
    // -----------------------------

    fn frogger_world() -> World {
        let mut world = World::new();
        spawn_initial(&mut world);
        world
    }

    fn frog(world: &World) -> Rect {
        world.with_tag(Tag::Player).next().unwrap().transform
    }

    fn press_up() -> Input {
        Input {
            up: true,
            dt: 1.0 / 60.0,
            ..Default::default()
        }
    }

    // -----------------------------
    // Movement Tests
    // -----------------------------

    #[test]
    fn frog_steps_one_cell() {
        let mut world = frogger_world();
        let mut state = GameState::new();
        let before = frog(&world);

        frog_input_system(&mut world, &mut state, &press_up());

        assert_eq!(frog(&world).y, before.y - CELL);
    }

    #[test]
    fn frog_cannot_leave_the_board() {
        let mut world = frogger_world();
        let mut state = GameState::new();

        for f in world.with_tag_mut(Tag::Player) {
            f.transform.x = 0.0;
        }
        let input = Input {
            left: true,
            dt: 1.0 / 60.0,
            ..Default::default()
        };
        frog_input_system(&mut world, &mut state, &input);

        assert_eq!(frog(&world).x, 0.0);
    }

    #[test]
    fn cars_move_and_wrap() {
        let mut world = frogger_world();
        let mut state = GameState::new();

        // Push one car just past the right edge and step a right-moving lane.
        let (x_before, moved_right) = {
            let car = world.with_tag_mut(Tag::Enemy).next().unwrap();
            let physics = car.physics.as_mut().unwrap();
            let moved_right = physics.velocity.x > 0.0;
            if moved_right {
                car.transform.x = BOARD_W + car.transform.w;
            } else {
                car.transform.x = -car.transform.w * 2.0;
            }
            (car.transform.x, moved_right)
        };

        car_system(&mut world, &mut state, &press_up());

        let car = world.with_tag(Tag::Enemy).next().unwrap();
        if moved_right {
            assert!(car.transform.x < x_before);
        } else {
            assert!(car.transform.x > x_before);
        }
    }

    // -----------------------------
    // Collision and Goal Tests
    // -----------------------------

    #[test]
    fn car_hit_ends_game() {
        let mut world = frogger_world();
        let mut state = GameState::new();

        let f = frog(&world);
        {
            let car = world.with_tag_mut(Tag::Enemy).next().unwrap();
            car.transform.x = f.x;
            car.transform.y = f.y;
        }

        collision_system(&mut world, &mut state, &press_up());

        assert!(state.game_over);
    }

    #[test]
    fn safe_frog_keeps_playing() {
        let mut world = frogger_world();
        let mut state = GameState::new();

        collision_system(&mut world, &mut state, &press_up());

        assert!(!state.game_over);
    }

    #[test]
    fn reaching_goal_scores_and_resets() {
        let mut world = frogger_world();
        let mut state = GameState::new();

        for f in world.with_tag_mut(Tag::Player) {
            f.transform.y = 0.0;
        }

        goal_system(&mut world, &mut state, &press_up());

        assert_eq!(state.score, 1.0);
        assert_eq!(frog(&world).y, frog_start().y);
    }
}
