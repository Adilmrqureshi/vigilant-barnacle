#[cfg(test)]
mod tests {

    // -----------------------------
    // Helpers
    // -----------------------------

    use crate::*;

    fn make_entity(tag: Tag) -> Entity {
        Entity::new(Rect {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
        })
        .with_tag(tag)
    }

    // -----------------------------
    // Entity Tests
    // -----------------------------

    #[test]
    fn entity_builder_sets_fields() {
        let e = Entity::new(Rect {
            x: 1.0,
            y: 2.0,
            w: 3.0,
            h: 4.0,
        })
        .with_tag(Tag::Player)
        .with_render(WHITE)
        .with_physics(Physics::new())
        .with_sprite(5)
        .with_attack(10);

        assert_eq!(e.tag, Some(Tag::Player));
        assert!(e.render.is_some());
        assert!(e.physics.is_some());
        assert_eq!(e.current_sprite, Some(5));
        assert_eq!(e.original_sprite, Some(5));
        assert!(e.attack.is_some());
    }

    // -----------------------------
    // World Tests
    // -----------------------------

    #[test]
    fn world_spawns_entities() {
        let world = World::new()
            .spawn(make_entity(Tag::Player))
            .spawn(make_entity(Tag::Enemy));

        assert_eq!(world.entities.len(), 2);
    }

    #[test]
    fn with_tag_filters_correctly() {
        let world = World::new()
            .spawn(make_entity(Tag::Player))
            .spawn(make_entity(Tag::Enemy))
            .spawn(make_entity(Tag::Enemy));

        let enemies: Vec<_> = world.with_tag(Tag::Enemy).collect();

        assert_eq!(enemies.len(), 2);
    }

    #[test]
    fn with_tag_mut_allows_modification() {
        let mut world = World::new()
            .spawn(make_entity(Tag::Enemy))
            .spawn(make_entity(Tag::Enemy));

        for e in world.with_tag_mut(Tag::Enemy) {
            e.transform.x = 42.0;
        }

        for e in world.with_tag(Tag::Enemy) {
            assert_eq!(e.transform.x, 42.0);
        }
    }

    // -----------------------------
    // Resource Tests
    // -----------------------------

    #[test]
    fn insert_and_get_resource() {
        let mut world = World::new();

        world.insert_resource(EnemyManager {
            active_enemy: Some(3),
        });

        let res = world.get_resource::<EnemyManager>().unwrap();

        assert_eq!(res.active_enemy, Some(3));
    }

    #[test]
    fn get_resource_mut_updates_value() {
        let mut world = World::new();

        world.insert_resource(EnemyManager {
            active_enemy: None,
        });

        {
            let res = world.get_resource_mut::<EnemyManager>().unwrap();
            res.active_enemy = Some(99);
        }

        let res = world.get_resource::<EnemyManager>().unwrap();
        assert_eq!(res.active_enemy, Some(99));
    }

    #[test]
    fn missing_resource_returns_none() {
        let world = World::new();

        let res = world.get_resource::<EnemyManager>();

        assert!(res.is_none());
    }

    // -----------------------------
    // Physics Tests
    // -----------------------------

    #[test]
    fn physics_initial_state() {
        let p = Physics::new();

        assert!(p.is_grounded);
        assert_eq!(p.velocity.x, 0.0);
        assert_eq!(p.velocity.y, 0.0);
    }

    // -----------------------------
    // Systems Tests
    // -----------------------------

    fn dummy_system(world: &mut World, state: &mut GameState, _input: &Input) {
        world.entities.push(Entity::new(Rect {
            x: 1.0,
            y: 1.0,
            w: 1.0,
            h: 1.0,
        }));

        state.score += 1.0;
    }

    fn dummy_render(_world: &World, state: &GameState) {
        assert!(state.score >= 0.0);
    }

    #[test]
    fn game_update_runs_systems() {
        let world = World::new();
        let mut game = Game::new(world).with_update_system(dummy_system);

        let input = Input {
            dt: 0.016,
            spacebar: false,
            a: false,
            screen_width: 800.0
        };

        game.update(&input);

        assert_eq!(game.world.entities.len(), 1);
        assert_eq!(game.state.score, 1.0);
    }

    #[test]
    fn game_render_runs_render_systems() {
        let world = World::new();
        let game = Game::new(world).with_render_system(dummy_render);

        // Just ensures no panic
        game.render();
    }

    #[test]
    fn multiple_systems_run_in_order() {
        fn system_a(_w: &mut World, s: &mut GameState, _i: &Input) {
            s.score += 1.0;
        }

        fn system_b(_w: &mut World, s: &mut GameState, _i: &Input) {
            s.score *= 2.0;
        }

        let world = World::new();
        let mut game = Game::new(world)
            .with_update_system(system_a)
            .with_update_system(system_b);

        let input = Input {
            dt: 0.016,
            spacebar: false,
            a: false,
            screen_width: 800.0
        };

        game.update(&input);

        // (0 + 1) * 2 = 2
        assert_eq!(game.state.score, 2.0);
    }
}
