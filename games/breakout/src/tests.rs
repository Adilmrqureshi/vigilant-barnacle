#[cfg(test)]
mod tests {
    use shared_v2::{GameState, Input, Tag, World};

    use crate::*;

    // -----------------------------
    // Helpers
    // -----------------------------

    fn breakout_world() -> World {
        let mut world = World::new();
        spawn_initial(&mut world);
        world.insert_resource(BreakoutState {
            lives: START_LIVES,
            won: false,
        });
        world
    }

    fn frame_input() -> Input {
        Input {
            dt: 1.0 / 60.0,
            ..Default::default()
        }
    }

    fn ball_mut(world: &mut World) -> &mut Entity {
        world.with_tag_mut(Tag::Ball).next().unwrap()
    }

    fn brick_count(world: &World) -> usize {
        world.with_tag(Tag::Brick).count()
    }

    // -----------------------------
    // Paddle Tests
    // -----------------------------

    #[test]
    fn paddle_moves_right_and_clamps() {
        let mut world = breakout_world();
        let mut state = GameState::new();

        let input = Input {
            right: true,
            dt: 10.0,
            ..Default::default()
        };
        paddle_input_system(&mut world, &mut state, &input);

        let paddle = world.with_tag(Tag::Player).next().unwrap().transform;
        assert_eq!(paddle.x, BOARD_W - PADDLE_W);
    }

    // -----------------------------
    // Ball Tests
    // -----------------------------

    #[test]
    fn ball_bounces_off_side_wall() {
        let mut world = breakout_world();
        let mut state = GameState::new();

        {
            let ball = ball_mut(&mut world);
            ball.transform.x = 0.0;
            ball.transform.y = BOARD_H / 2.0;
            let physics = ball.physics.as_mut().unwrap();
            physics.velocity.x = -100.0;
            physics.velocity.y = 0.0;
        }

        ball_system(&mut world, &mut state, &frame_input());

        let vx = ball_mut(&mut world).physics.as_ref().unwrap().velocity.x;
        assert!(vx > 0.0);
    }

    #[test]
    fn losing_ball_costs_a_life() {
        let mut world = breakout_world();
        let mut state = GameState::new();

        {
            let ball = ball_mut(&mut world);
            ball.transform.y = BOARD_H * 2.0;
        }

        ball_system(&mut world, &mut state, &frame_input());

        let breakout = world.get_resource::<BreakoutState>().unwrap();
        assert_eq!(breakout.lives, START_LIVES - 1);
        assert!(!state.game_over);

        // Ball is back in play above the paddle.
        let ball = world.with_tag(Tag::Ball).next().unwrap();
        assert!(ball.transform.y < PADDLE_Y);
    }

    #[test]
    fn losing_last_life_ends_game() {
        let mut world = breakout_world();
        let mut state = GameState::new();

        world.get_resource_mut::<BreakoutState>().unwrap().lives = 1;
        {
            let ball = ball_mut(&mut world);
            ball.transform.y = BOARD_H * 2.0;
        }

        ball_system(&mut world, &mut state, &frame_input());

        assert!(state.game_over);
    }

    // -----------------------------
    // Brick Tests
    // -----------------------------

    #[test]
    fn hitting_brick_removes_it_and_scores() {
        let mut world = breakout_world();
        let mut state = GameState::new();
        let before = brick_count(&world);

        let brick = world.with_tag(Tag::Brick).next().unwrap().transform;
        {
            let ball = ball_mut(&mut world);
            ball.transform.x = brick.x + brick.w / 2.0;
            ball.transform.y = brick.y + brick.h / 2.0;
        }

        brick_collision_system(&mut world, &mut state, &frame_input());

        assert_eq!(brick_count(&world), before - 1);
        assert_eq!(state.score, 1.0);
    }

    #[test]
    fn brick_hit_reflects_ball() {
        let mut world = breakout_world();
        let mut state = GameState::new();

        let brick = world.with_tag(Tag::Brick).next().unwrap().transform;
        {
            let ball = ball_mut(&mut world);
            ball.transform.x = brick.x + brick.w / 2.0;
            ball.transform.y = brick.y + brick.h / 2.0;
            ball.physics.as_mut().unwrap().velocity.y = -200.0;
        }

        brick_collision_system(&mut world, &mut state, &frame_input());

        let vy = ball_mut(&mut world).physics.as_ref().unwrap().velocity.y;
        assert!(vy > 0.0);
    }

    // -----------------------------
    // Win Tests
    // -----------------------------

    #[test]
    fn clearing_all_bricks_wins() {
        let mut world = breakout_world();
        let mut state = GameState::new();

        for brick in world.with_tag_mut(Tag::Brick) {
            brick.alive = false;
        }
        world.despawn_dead();

        win_system(&mut world, &mut state, &frame_input());

        assert!(state.game_over);
        assert!(world.get_resource::<BreakoutState>().unwrap().won);
    }

    #[test]
    fn no_win_while_bricks_remain() {
        let mut world = breakout_world();
        let mut state = GameState::new();

        win_system(&mut world, &mut state, &frame_input());

        assert!(!state.game_over);
        assert!(!world.get_resource::<BreakoutState>().unwrap().won);
    }
}
