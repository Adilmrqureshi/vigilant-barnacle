#[cfg(test)]
mod tests {
    use shared_v2::{GameState, Input, Tag, World};

    use crate::*;

    // -----------------------------
    // Helpers
    // -----------------------------

    fn invaders_world() -> World {
        let mut world = World::new();
        spawn_initial(&mut world);
        world.insert_resource(Fleet::new());
        world
    }

    fn frame_input() -> Input {
        Input {
            dt: 1.0 / 60.0,
            ..Default::default()
        }
    }

    fn bullet_count(world: &World) -> usize {
        world.with_tag(Tag::Bullet).count()
    }

    fn invader_count(world: &World) -> usize {
        world.with_tag(Tag::Enemy).count()
    }

    // -----------------------------
    // Ship Tests
    // -----------------------------

    #[test]
    fn ship_moves_left_and_clamps() {
        let mut world = invaders_world();
        let mut state = GameState::new();

        let input = Input {
            left: true,
            dt: 10.0,
            ..Default::default()
        };
        player_input_system(&mut world, &mut state, &input);

        let ship = world.with_tag(Tag::Player).next().unwrap().transform;
        assert_eq!(ship.x, 0.0);
    }

    // -----------------------------
    // Shooting Tests
    // -----------------------------

    #[test]
    fn spacebar_spawns_single_bullet() {
        let mut world = invaders_world();
        let mut state = GameState::new();

        let input = Input {
            spacebar: true,
            dt: 1.0 / 60.0,
            ..Default::default()
        };
        player_shoot_system(&mut world, &mut state, &input);
        assert_eq!(bullet_count(&world), 1);

        // Second press while the bullet is in flight does nothing.
        player_shoot_system(&mut world, &mut state, &input);
        assert_eq!(bullet_count(&world), 1);
    }

    #[test]
    fn bullets_despawn_off_screen() {
        let mut world = invaders_world();
        let mut state = GameState::new();

        let input = Input {
            spacebar: true,
            dt: 1.0 / 60.0,
            ..Default::default()
        };
        player_shoot_system(&mut world, &mut state, &input);

        for bullet in world.with_tag_mut(Tag::Bullet) {
            bullet.transform.y = -BULLET_H * 2.0;
        }
        bullet_system(&mut world, &mut state, &frame_input());

        assert_eq!(bullet_count(&world), 0);
    }

    #[test]
    fn enemy_fire_respects_cooldown() {
        let mut world = invaders_world();
        let mut state = GameState::new();

        // Not enough time elapsed: no shot.
        enemy_fire_system(&mut world, &mut state, &frame_input());
        assert_eq!(bullet_count(&world), 0);

        // Cooldown expired: one shot.
        let input = Input {
            dt: ENEMY_FIRE_INTERVAL,
            ..Default::default()
        };
        enemy_fire_system(&mut world, &mut state, &input);
        assert_eq!(bullet_count(&world), 1);
    }

    // -----------------------------
    // Fleet Tests
    // -----------------------------

    #[test]
    fn fleet_marches_in_current_direction() {
        let mut world = invaders_world();
        let mut state = GameState::new();
        let before = world.with_tag(Tag::Enemy).next().unwrap().transform.x;

        fleet_system(&mut world, &mut state, &frame_input());

        let after = world.with_tag(Tag::Enemy).next().unwrap().transform.x;
        assert!(after > before);
    }

    #[test]
    fn fleet_reverses_and_drops_at_edge() {
        let mut world = invaders_world();
        let mut state = GameState::new();

        let top_before = world.with_tag(Tag::Enemy).next().unwrap().transform.y;
        for invader in world.with_tag_mut(Tag::Enemy) {
            invader.transform.x += BOARD_W; // push the fleet past the right edge
        }

        fleet_system(&mut world, &mut state, &frame_input());

        let fleet = world.get_resource::<Fleet>().unwrap();
        assert_eq!(fleet.dir, -1.0);
        let top_after = world.with_tag(Tag::Enemy).next().unwrap().transform.y;
        assert_eq!(top_after, top_before + FLEET_DROP);
    }

    #[test]
    fn fleet_reaching_ship_ends_game() {
        let mut world = invaders_world();
        let mut state = GameState::new();

        for invader in world.with_tag_mut(Tag::Enemy) {
            invader.transform.y = SHIP_Y;
        }

        fleet_system(&mut world, &mut state, &frame_input());

        assert!(state.game_over);
    }

    // -----------------------------
    // Hit Tests
    // -----------------------------

    fn run_hit_pipeline(world: &mut World, state: &mut GameState) {
        let input = frame_input();
        hit_detection_system(world, state, &input);
        hit_scoring_system(world, state, &input);
        hit_despawn_system(world, state, &input);
        player_hit_system(world, state, &input);
    }

    #[test]
    fn overlap_emits_a_bullet_hit_event() {
        let mut world = invaders_world();
        let mut state = GameState::new();

        let target = world.with_tag(Tag::Enemy).next().unwrap().transform;
        spawn_bullet(
            &mut world,
            target.x + target.w / 2.0,
            target.y,
            -PLAYER_BULLET_SPEED,
            PLAYER_BULLET_COLOR,
        );

        hit_detection_system(&mut world, &mut state, &frame_input());

        let hits = world.events::<BulletHit>();
        assert_eq!(hits.len(), 1);
        // The event references entities by stable id, not index.
        assert!(world.find(hits[0].invader).is_some());
        assert!(world.find(hits[0].bullet).is_some());
    }

    #[test]
    fn player_bullet_kills_invader_and_scores() {
        let mut world = invaders_world();
        let mut state = GameState::new();
        let before = invader_count(&world);

        let target = world.with_tag(Tag::Enemy).next().unwrap().transform;
        spawn_bullet(
            &mut world,
            target.x + target.w / 2.0,
            target.y,
            -PLAYER_BULLET_SPEED,
            PLAYER_BULLET_COLOR,
        );

        run_hit_pipeline(&mut world, &mut state);

        assert_eq!(invader_count(&world), before - 1);
        assert_eq!(state.score, 1.0);
        assert_eq!(world.get_resource::<Fleet>().unwrap().kills, 1);
        assert_eq!(bullet_count(&world), 0); // the shot is spent
        assert!(!state.game_over);
    }

    #[test]
    fn one_bullet_kills_at_most_one_invader() {
        let mut world = invaders_world();
        let mut state = GameState::new();

        // Stack two invaders on the same cell, one bullet through them.
        let target = world.with_tag(Tag::Enemy).next().unwrap().transform;
        {
            let mut enemies = world.with_tag_mut(Tag::Enemy);
            enemies.next();
            let second = enemies.next().unwrap();
            second.transform.x = target.x;
            second.transform.y = target.y;
        }
        spawn_bullet(
            &mut world,
            target.x + target.w / 2.0,
            target.y,
            -PLAYER_BULLET_SPEED,
            PLAYER_BULLET_COLOR,
        );

        hit_detection_system(&mut world, &mut state, &frame_input());

        assert_eq!(world.events::<BulletHit>().len(), 1);
    }

    #[test]
    fn enemy_bullet_hitting_ship_ends_game() {
        let mut world = invaders_world();
        let mut state = GameState::new();

        let ship = world.with_tag(Tag::Player).next().unwrap().transform;
        spawn_bullet(
            &mut world,
            ship.x + ship.w / 2.0,
            ship.y,
            ENEMY_BULLET_SPEED,
            ENEMY_BULLET_COLOR,
        );

        run_hit_pipeline(&mut world, &mut state);

        assert!(state.game_over);
    }

    // -----------------------------
    // Win Tests
    // -----------------------------

    #[test]
    fn clearing_fleet_wins() {
        let mut world = invaders_world();
        let mut state = GameState::new();

        for invader in world.with_tag_mut(Tag::Enemy) {
            invader.alive = false;
        }
        world.despawn_dead();

        win_system(&mut world, &mut state, &frame_input());

        assert!(state.game_over);
        assert!(world.get_resource::<Fleet>().unwrap().won);
    }

    #[test]
    fn no_win_while_invaders_remain() {
        let mut world = invaders_world();
        let mut state = GameState::new();

        win_system(&mut world, &mut state, &frame_input());

        assert!(!state.game_over);
        assert!(!world.get_resource::<Fleet>().unwrap().won);
    }
}
