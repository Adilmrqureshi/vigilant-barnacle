#[cfg(test)]
mod tests {
    use shared_v2::{GameState, Input, Tag, World};

    use crate::*;

    // -----------------------------
    // Helpers
    // -----------------------------

    fn climber_world() -> World {
        let mut world = World::new();
        spawn_initial(&mut world);
        world
    }

    fn frame() -> Input {
        Input {
            dt: 1.0 / 60.0,
            ..Default::default()
        }
    }

    fn player(world: &World) -> Rect {
        world.with_tag(Tag::Player).next().unwrap().transform
    }

    fn player_vy(world: &World) -> f32 {
        let p = world.with_tag(Tag::Player).next().unwrap();
        p.physics.as_ref().unwrap().velocity.y
    }

    // -----------------------------
    // Physics Tests
    // -----------------------------

    #[test]
    fn gravity_accelerates_downward() {
        let mut world = climber_world();
        let mut state = GameState::new();

        // Mid-air, horizontally clear of every platform.
        for p in world.with_tag_mut(Tag::Player) {
            p.transform.x = -200.0;
            p.transform.y = 100.0;
        }
        physics_system(&mut world, &mut state, &frame());

        assert!(player_vy(&world) > 0.0);
    }

    #[test]
    fn bounces_off_platform_when_falling() {
        let mut world = climber_world();
        let mut state = GameState::new();

        // The player starts resting on the bottom platform; without a bounce,
        // ten frames of gravity would leave it falling (vy > 0).
        for _ in 0..10 {
            physics_system(&mut world, &mut state, &frame());
        }

        assert!(player_vy(&world) < 0.0);
    }

    #[test]
    fn no_bounce_when_moving_upward() {
        let mut world = climber_world();
        let mut state = GameState::new();

        for p in world.with_tag_mut(Tag::Player) {
            p.physics.as_mut().unwrap().velocity.y = JUMP_VY;
        }
        physics_system(&mut world, &mut state, &frame());

        // Still moving up (gravity only slowed it slightly).
        assert!(player_vy(&world) < 0.0);
    }

    #[test]
    fn falling_off_the_bottom_ends_game() {
        let mut world = climber_world();
        let mut state = GameState::new();

        for p in world.with_tag_mut(Tag::Player) {
            p.transform.y = BOARD_H + 1.0;
            p.transform.x = -500.0; // away from any platform
        }
        physics_system(&mut world, &mut state, &frame());

        assert!(state.game_over);
    }

    // -----------------------------
    // Steering Tests
    // -----------------------------

    #[test]
    fn wraps_around_the_side_edges() {
        let mut world = climber_world();
        let mut state = GameState::new();

        for p in world.with_tag_mut(Tag::Player) {
            p.transform.x = -p.transform.w - 1.0;
        }
        steer_system(&mut world, &mut state, &frame());

        assert_eq!(player(&world).x, BOARD_W);
    }

    // -----------------------------
    // Scrolling Tests
    // -----------------------------

    #[test]
    fn climbing_scrolls_the_world_down() {
        let mut world = climber_world();
        let mut state = GameState::new();

        let platform_before = world.with_tag(Tag::Brick).next().unwrap().transform.y;
        for p in world.with_tag_mut(Tag::Player) {
            p.transform.y = SCROLL_LINE - 100.0;
        }

        scroll_system(&mut world, &mut state, &frame());

        assert_eq!(player(&world).y, SCROLL_LINE);
        let platform_after = world.with_tag(Tag::Brick).next().unwrap().transform.y;
        assert_eq!(platform_after, platform_before + 100.0);
        assert_eq!(world.get_resource::<Climb>().unwrap().height, 100.0);
        assert_eq!(state.score, 10.0);
    }

    #[test]
    fn platforms_recycle_to_the_top() {
        let mut world = climber_world();
        let mut state = GameState::new();

        // Put one platform just above the bottom so a scroll pushes it off.
        {
            let platform = world.with_tag_mut(Tag::Brick).next().unwrap();
            platform.transform.y = BOARD_H - 10.0;
        }
        for p in world.with_tag_mut(Tag::Player) {
            p.transform.y = SCROLL_LINE - 50.0;
        }

        scroll_system(&mut world, &mut state, &frame());

        let platform = world.with_tag(Tag::Brick).next().unwrap();
        assert!(platform.transform.y < BOARD_H - 10.0);
    }

    #[test]
    fn no_scroll_below_the_line() {
        let mut world = climber_world();
        let mut state = GameState::new();

        let before = world.with_tag(Tag::Brick).next().unwrap().transform.y;
        scroll_system(&mut world, &mut state, &frame());

        let after = world.with_tag(Tag::Brick).next().unwrap().transform.y;
        assert_eq!(before, after);
    }
}
