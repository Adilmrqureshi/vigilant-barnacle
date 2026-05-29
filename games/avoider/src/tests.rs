#[cfg(test)]
mod tests {
    use macroquad::math::Rect;
    use shared_v2::{EnemyManager, Entity, GameState, Input, Physics, Tag, World};

    use crate::*;

    // -----------------------------
    // Helpers
    // -----------------------------

    fn dummy_input() -> Input {
        Input {
            dt: 1.0 / 60.0,
            spacebar: false,
            a: false,
            screen_width: 800.0,
        }
    }

    fn player_entity() -> Entity {
        Entity::new(Rect {
            x: DEFAULT_PLAYER_POSITION,
            y: GROUND,
            w: GAME_SPRITE_SIZE,
            h: GAME_SPRITE_SIZE,
        })
        .with_tag(Tag::Player)
        .with_physics(Physics::new())
    }

    fn enemy_entity(x: f32) -> Entity {
        Entity::new(Rect {
            x,
            y: GROUND,
            w: GAME_SPRITE_SIZE,
            h: GAME_SPRITE_SIZE,
        })
        .with_tag(Tag::Enemy)
    }

    fn world_with_manager() -> World {
        let mut world = World::new();
        world.insert_resource(EnemyManager { active_enemy: None });
        world
    }

    // -----------------------------
    // Gravity Tests
    // -----------------------------

    #[test]
    fn jump_sets_positive_velocity() {
        let mut world = World::new().spawn(player_entity());
        let mut state = GameState::new();

        let input = Input {
            dt: 0.016,
            spacebar: true,
            a: false,
            screen_width: 100.0,
        };

        gravity_system(&mut world, &mut state, &input);

        let vel = world.entities[0].physics.as_ref().unwrap().velocity.y;

        assert!(vel > 0.0);
    }

    #[test]
    fn gravity_pulls_down_when_airborne() {
        let mut p = player_entity();
        p.physics.as_mut().unwrap().is_grounded = false;
        p.physics.as_mut().unwrap().velocity.y = 0.0;

        let mut world = World::new().spawn(p);
        let mut state = GameState::new();

        gravity_system(&mut world, &mut state, &dummy_input());

        let vel = world.entities[0].physics.as_ref().unwrap().velocity.y;

        assert!(vel < 0.0);
    }

    #[test]
    fn player_lands_on_ground() {
        let mut p = player_entity();
        p.transform.y = GROUND - 10.0;
        p.physics.as_mut().unwrap().is_grounded = false;

        let mut world = World::new().spawn(p);
        let mut state = GameState::new();

        gravity_system(&mut world, &mut state, &dummy_input());

        let player = &world.entities[0];

        assert_eq!(player.transform.y, GROUND);
        assert!(player.physics.as_ref().unwrap().is_grounded);
    }

    // -----------------------------
    // Movement Tests
    // -----------------------------

    #[test]
    fn enemies_move_left() {
        let mut world = World::new().spawn(enemy_entity(500.0));
        let mut state = GameState::new();

        let input = Input {
            dt: 1.0,
            spacebar: false,
            a: false,
            screen_width: 800.0,
        };

        move_enemy_system(&mut world, &mut state, &input);

        assert!(world.entities[0].transform.x < 500.0);
    }

    // -----------------------------
    // Collision Tests
    // -----------------------------

    #[test]
    fn collision_triggers_game_over() {
        let player = player_entity();
        let enemy = enemy_entity(DEFAULT_PLAYER_POSITION);

        let mut world = World::new().spawn(player).spawn(enemy);
        let mut state = GameState::new();

        collision_system(&mut world, &mut state, &dummy_input());

        assert!(state.game_over);
    }

    // -----------------------------
    // Enemy Spawn Tests
    // -----------------------------

    #[test]
    fn spawns_enemy_when_none_active() {
        let mut world = world_with_manager()
            .spawn(enemy_entity(-1000.0))
            .spawn(enemy_entity(-2000.0));

        let mut state = GameState::new();

        enemy_spawn_system(&mut world, &mut state, &dummy_input());

        let manager = world.get_resource::<EnemyManager>().unwrap();
        assert!(manager.active_enemy.is_some());
    }

    #[test]
    fn does_not_spawn_if_enemy_still_visible() {
        let mut world = world_with_manager().spawn(enemy_entity(100.0));

        {
            let mgr = world.get_resource_mut::<EnemyManager>().unwrap();
            mgr.active_enemy = Some(0);
        }

        let mut state = GameState::new();

        enemy_spawn_system(&mut world, &mut state, &dummy_input());

        let mgr = world.get_resource::<EnemyManager>().unwrap();
        assert_eq!(mgr.active_enemy, Some(0));
    }

    #[test]
    fn respawns_when_enemy_leaves_screen() {
        let mut world = world_with_manager()
            .spawn(enemy_entity(-500.0))
            .spawn(enemy_entity(-600.0));

        {
            let mgr = world.get_resource_mut::<EnemyManager>().unwrap();
            mgr.active_enemy = Some(0);
        }

        let mut state = GameState::new();

        enemy_spawn_system(&mut world, &mut state, &dummy_input());

        let mgr = world.get_resource::<EnemyManager>().unwrap();
        assert!(mgr.active_enemy.is_some());
    }
}
