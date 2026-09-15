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

    #[test]
    fn despawn_dead_removes_only_dead_entities() {
        let mut world = World::new()
            .spawn(make_entity(Tag::Player))
            .spawn(make_entity(Tag::Enemy))
            .spawn(make_entity(Tag::Enemy));

        world.entities[1].alive = false;
        world.despawn_dead();

        assert_eq!(world.entities.len(), 2);
        assert!(world.entities.iter().all(|e| e.alive));
    }

    #[test]
    fn add_assigns_unique_ids_and_find_survives_despawn() {
        let mut world = World::new();
        let a = world.add(make_entity(Tag::Player));
        let b = world.add(make_entity(Tag::Enemy));
        let c = world.add(make_entity(Tag::Enemy));
        assert!(a != b && b != c);

        world.find_mut(b).unwrap().alive = false;
        world.despawn_dead();

        // Indices shifted, but ids still resolve to the right entities.
        assert!(world.find(b).is_none());
        assert_eq!(world.find(a).unwrap().tag, Some(Tag::Player));
        assert_eq!(world.find(c).unwrap().tag, Some(Tag::Enemy));
    }

    // -----------------------------
    // Event Tests
    // -----------------------------

    struct Ping(u32);

    #[test]
    fn events_reach_multiple_readers() {
        let mut world = World::new();
        world.emit(Ping(1));
        world.emit(Ping(2));

        // Two independent reads see the same events.
        assert_eq!(world.events::<Ping>().len(), 2);
        assert_eq!(world.events::<Ping>()[1].0, 2);
    }

    #[test]
    fn events_live_exactly_one_update() {
        fn emitter(world: &mut World, _s: &mut GameState, _i: &Input) {
            world.emit(Ping(7));
        }
        fn counter(world: &mut World, state: &mut GameState, _i: &Input) {
            state.score += world.events::<Ping>().len() as f32;
        }

        let mut game = Game::new(World::new())
            .with_update_system(emitter)
            .with_update_system(counter);

        let input = Input::default();
        game.update(&input); // emit + read in the same frame
        assert_eq!(game.state.score, 1.0);

        game.systems.update.remove(0); // stop emitting
        game.update(&input); // last frame's events were cleared
        assert_eq!(game.state.score, 1.0);
    }

    #[test]
    fn no_events_reads_as_empty() {
        let world = World::new();
        assert!(world.events::<Ping>().is_empty());
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
            screen_width: 800.0,
            ..Default::default()
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
            screen_width: 800.0,
            ..Default::default()
        };

        game.update(&input);

        // (0 + 1) * 2 = 2
        assert_eq!(game.state.score, 2.0);
    }

    // -----------------------------
    // Health, kind, velocity, Timer
    // -----------------------------

    #[test]
    fn health_kind_and_velocity_builders() {
        let e = make_entity(Tag::Enemy)
            .with_kind(3)
            .with_health(40.0)
            .with_velocity(2.0, -1.0);
        assert_eq!(e.kind, Some(3));
        let h = e.health.unwrap();
        assert_eq!(h.hp, 40.0);
        assert_eq!(h.max, 40.0);
        assert!(!h.is_dead());
        let v = &e.physics.as_ref().unwrap().velocity;
        assert_eq!((v.x, v.y), (2.0, -1.0));
        assert_eq!(e.center(), (5.0, 5.0));
    }

    #[test]
    fn health_fraction_clamps_and_reports_death() {
        let mut h = Health::new(50.0);
        h.hp = 25.0;
        assert_eq!(h.fraction(), 0.5);
        h.hp = -10.0;
        assert_eq!(h.fraction(), 0.0);
        assert!(h.is_dead());
    }

    #[test]
    fn timer_fires_on_period_and_rearms() {
        let mut t = Timer::new(1.0);
        assert!(!t.tick(0.6));
        assert!(t.tick(0.6));
        // Carries the overshoot into the next period.
        assert!((t.remaining - 0.8).abs() < 1e-5);
        assert!(!t.tick(0.5));
        assert!(t.tick(0.5));
    }

    #[test]
    fn ready_timer_fires_on_first_tick() {
        let mut t = Timer::ready(2.0);
        assert!(t.tick(0.016));
        assert!(!t.tick(0.016));
        assert!(t.fraction() > 0.0 && t.fraction() < 0.1);
    }

    #[test]
    fn timer_huge_step_does_not_stay_negative() {
        let mut t = Timer::new(1.0);
        assert!(t.tick(10.0));
        assert_eq!(t.remaining, 1.0);
    }
}
