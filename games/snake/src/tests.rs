#[cfg(test)]
mod tests {
    use shared_v2::{GameState, Input, Tag, World};

    use crate::*;

    // -----------------------------
    // Helpers
    // -----------------------------

    fn step_input() -> Input {
        Input {
            dt: STEP_TIME,
            ..Default::default()
        }
    }

    fn snake_world() -> World {
        let mut world = World::new();
        spawn_initial(&mut world);
        world.insert_resource(SnakeState::new());
        world
    }

    fn head_pos(world: &World) -> (f32, f32) {
        let head = world.with_tag(Tag::Player).next().unwrap();
        (head.transform.x, head.transform.y)
    }

    fn segment_count(world: &World) -> usize {
        world.entities.iter().filter(|e| is_segment(e)).count()
    }

    // -----------------------------
    // Direction Tests
    // -----------------------------

    #[test]
    fn direction_changes_on_input() {
        let mut world = snake_world();
        let mut state = GameState::new();

        let input = Input {
            up: true,
            ..Default::default()
        };

        direction_system(&mut world, &mut state, &input);

        let snake = world.get_resource::<SnakeState>().unwrap();
        assert_eq!(snake.next_dir, Dir::Up);
    }

    #[test]
    fn cannot_reverse_into_itself() {
        let mut world = snake_world();
        let mut state = GameState::new();

        // Snake starts moving right; pressing left must be ignored.
        let input = Input {
            left: true,
            ..Default::default()
        };

        direction_system(&mut world, &mut state, &input);

        let snake = world.get_resource::<SnakeState>().unwrap();
        assert_eq!(snake.next_dir, Dir::Right);
    }

    // -----------------------------
    // Movement Tests
    // -----------------------------

    #[test]
    fn head_advances_one_cell_per_step() {
        let mut world = snake_world();
        let mut state = GameState::new();
        let (x, y) = head_pos(&world);

        movement_system(&mut world, &mut state, &step_input());

        assert_eq!(head_pos(&world), (x + CELL, y));
    }

    #[test]
    fn does_not_move_before_step_time() {
        let mut world = snake_world();
        let mut state = GameState::new();
        let before = head_pos(&world);

        let input = Input {
            dt: STEP_TIME / 2.0,
            ..Default::default()
        };
        movement_system(&mut world, &mut state, &input);

        assert_eq!(head_pos(&world), before);
    }

    #[test]
    fn body_follows_head() {
        let mut world = snake_world();
        let mut state = GameState::new();
        let old_head = head_pos(&world);

        movement_system(&mut world, &mut state, &step_input());

        let first_body = world.with_tag(Tag::Body).next().unwrap();
        assert_eq!((first_body.transform.x, first_body.transform.y), old_head);
    }

    #[test]
    fn grows_after_eating() {
        let mut world = snake_world();
        let mut state = GameState::new();
        let before = segment_count(&world);

        world.get_resource_mut::<SnakeState>().unwrap().pending_growth = 1;
        movement_system(&mut world, &mut state, &step_input());

        assert_eq!(segment_count(&world), before + 1);
        let snake = world.get_resource::<SnakeState>().unwrap();
        assert_eq!(snake.pending_growth, 0);
    }

    // -----------------------------
    // Food Tests
    // -----------------------------

    #[test]
    fn eating_food_scores_and_queues_growth() {
        let mut world = snake_world();
        let mut state = GameState::new();

        // Put the food on the head.
        let head = world.with_tag(Tag::Player).next().unwrap().transform;
        for food in world.with_tag_mut(Tag::Food) {
            food.transform.x = head.x;
            food.transform.y = head.y;
        }

        food_system(&mut world, &mut state, &step_input());

        assert_eq!(state.score, 1.0);
        let snake = world.get_resource::<SnakeState>().unwrap();
        assert_eq!(snake.pending_growth, 1);

        // Food must have been moved off the snake.
        let food = world.with_tag(Tag::Food).next().unwrap().transform;
        assert!(!same_cell(&food, &head));
    }

    #[test]
    fn missing_food_does_nothing() {
        let mut world = snake_world();
        let mut state = GameState::new();

        food_system(&mut world, &mut state, &step_input());

        assert_eq!(state.score, 0.0);
        let snake = world.get_resource::<SnakeState>().unwrap();
        assert_eq!(snake.pending_growth, 0);
    }

    // -----------------------------
    // Collision Tests
    // -----------------------------

    #[test]
    fn hitting_wall_ends_game() {
        let mut world = snake_world();
        let mut state = GameState::new();

        for head in world.with_tag_mut(Tag::Player) {
            head.transform.x = BOARD_W;
        }

        snake_collision_system(&mut world, &mut state, &step_input());

        assert!(state.game_over);
    }

    #[test]
    fn hitting_body_ends_game() {
        let mut world = snake_world();
        let mut state = GameState::new();

        let head = world.with_tag(Tag::Player).next().unwrap().transform;
        for body in world.with_tag_mut(Tag::Body) {
            body.transform.x = head.x;
            body.transform.y = head.y;
            break;
        }

        snake_collision_system(&mut world, &mut state, &step_input());

        assert!(state.game_over);
    }

    #[test]
    fn staying_on_board_keeps_playing() {
        let mut world = snake_world();
        let mut state = GameState::new();

        snake_collision_system(&mut world, &mut state, &step_input());

        assert!(!state.game_over);
    }
}
