#[cfg(test)]
mod tests {
    use shared_v2::{GameState, Input, Tag, World};

    use crate::*;

    // -----------------------------
    // Helpers
    // -----------------------------

    fn pong_world() -> World {
        let mut world = World::new();
        spawn_initial(&mut world);
        world.insert_resource(Scores { player: 0, cpu: 0 });
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

    // -----------------------------
    // Paddle Tests
    // -----------------------------

    #[test]
    fn player_paddle_moves_up() {
        let mut world = pong_world();
        let mut state = GameState::new();
        let before = world.with_tag(Tag::Player).next().unwrap().transform.y;

        let input = Input {
            up: true,
            dt: 1.0 / 60.0,
            ..Default::default()
        };
        player_input_system(&mut world, &mut state, &input);

        let after = world.with_tag(Tag::Player).next().unwrap().transform.y;
        assert!(after < before);
    }

    #[test]
    fn player_paddle_clamps_to_court() {
        let mut world = pong_world();
        let mut state = GameState::new();

        for paddle in world.with_tag_mut(Tag::Player) {
            paddle.transform.y = 0.0;
        }

        let input = Input {
            up: true,
            dt: 1.0,
            ..Default::default()
        };
        player_input_system(&mut world, &mut state, &input);

        let y = world.with_tag(Tag::Player).next().unwrap().transform.y;
        assert_eq!(y, 0.0);
    }

    #[test]
    fn ai_paddle_tracks_ball() {
        let mut world = pong_world();
        let mut state = GameState::new();

        {
            let ball = ball_mut(&mut world);
            ball.transform.y = 0.0;
        }
        let before = world.with_tag(Tag::Enemy).next().unwrap().transform.y;

        ai_system(&mut world, &mut state, &frame_input());

        let after = world.with_tag(Tag::Enemy).next().unwrap().transform.y;
        assert!(after < before);
    }

    // -----------------------------
    // Ball Tests
    // -----------------------------

    #[test]
    fn ball_bounces_off_top_wall() {
        let mut world = pong_world();
        let mut state = GameState::new();

        {
            let ball = ball_mut(&mut world);
            ball.transform.x = COURT_W / 2.0;
            ball.transform.y = 0.0;
            let physics = ball.physics.as_mut().unwrap();
            physics.velocity.x = 0.0;
            physics.velocity.y = -100.0;
        }

        ball_system(&mut world, &mut state, &frame_input());

        let vy = ball_mut(&mut world).physics.as_ref().unwrap().velocity.y;
        assert!(vy > 0.0);
    }

    #[test]
    fn ball_bounces_off_player_paddle() {
        let mut world = pong_world();
        let mut state = GameState::new();

        let paddle = world.with_tag(Tag::Player).next().unwrap().transform;
        {
            let ball = ball_mut(&mut world);
            ball.transform.x = paddle.x + paddle.w - 1.0;
            ball.transform.y = paddle.y + paddle.h / 2.0;
            let physics = ball.physics.as_mut().unwrap();
            physics.velocity.x = -100.0;
            physics.velocity.y = 0.0;
        }

        ball_system(&mut world, &mut state, &frame_input());

        let vx = ball_mut(&mut world).physics.as_ref().unwrap().velocity.x;
        assert!(vx > 0.0);
    }

    // -----------------------------
    // Scoring Tests
    // -----------------------------

    #[test]
    fn cpu_scores_when_ball_exits_left() {
        let mut world = pong_world();
        let mut state = GameState::new();

        {
            let ball = ball_mut(&mut world);
            ball.transform.x = -BALL_SIZE * 2.0;
        }

        score_system(&mut world, &mut state, &frame_input());

        let scores = world.get_resource::<Scores>().unwrap();
        assert_eq!(scores.cpu, 1);
        assert_eq!(scores.player, 0);

        // Ball is re-served from center.
        let ball = world.with_tag(Tag::Ball).next().unwrap();
        assert!(ball.transform.x > 0.0 && ball.transform.x < COURT_W);
    }

    #[test]
    fn reaching_win_score_ends_game() {
        let mut world = pong_world();
        let mut state = GameState::new();

        world.get_resource_mut::<Scores>().unwrap().player = WIN_SCORE - 1;
        {
            let ball = ball_mut(&mut world);
            ball.transform.x = COURT_W + BALL_SIZE;
        }

        score_system(&mut world, &mut state, &frame_input());

        let scores = world.get_resource::<Scores>().unwrap();
        assert_eq!(scores.player, WIN_SCORE);
        assert!(state.game_over);
    }

    #[test]
    fn no_score_while_ball_in_play() {
        let mut world = pong_world();
        let mut state = GameState::new();

        score_system(&mut world, &mut state, &frame_input());

        let scores = world.get_resource::<Scores>().unwrap();
        assert_eq!(scores.player, 0);
        assert_eq!(scores.cpu, 0);
        assert!(!state.game_over);
    }
}
