#[cfg(test)]
mod tests {
    use crate::*;

    fn test_entity(tag: Option<Tag>) -> Entity {
        let mut e = Entity::new(Rect {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
        });

        if let Some(t) = tag {
            e = e.with_tag(t);
        }

        e
    }

    #[test]
    fn entity_starts_empty() {
        let e = Entity::new(Rect {
            x: 1.0,
            y: 2.0,
            w: 3.0,
            h: 4.0,
        });

        assert!(e.tag.is_none());
        assert!(e.render.is_none());
        assert!(e.sprite.is_none());
        assert!(e.physics.is_none());
    }

    #[test]
    fn physics_defaults_are_correct() {
        let physics = Physics::new();

        assert!(physics.is_grounded);
        assert_eq!(physics.velocity.x, 0.0);
        assert_eq!(physics.velocity.y, 0.0);
    }

    #[test]
    fn world_spawn_adds_entities() {
        let world = World::new()
            .spawn(test_entity(None))
            .spawn(test_entity(Some(Tag::Player)));

        assert_eq!(world.entities.len(), 2);
    }

    #[test]
    fn with_tag_filters_entities() {
        let world = World::new()
            .spawn(test_entity(Some(Tag::Player)))
            .spawn(test_entity(None))
            .spawn(test_entity(Some(Tag::Player)));

        let players: Vec<&Entity> = world.with_tag(Tag::Player).collect();

        assert_eq!(players.len(), 2);
        for p in players {
            assert_eq!(p.tag, Some(Tag::Player));
        }
    }

    #[test]
    fn with_tag_mut_allows_mutation() {
        let mut world = World::new()
            .spawn(test_entity(Some(Tag::Player)))
            .spawn(test_entity(Some(Tag::Player)));

        for e in world.with_tag_mut(Tag::Player) {
            e.physics = Some(Physics::new());
        }

        let with_physics = world
            .entities
            .iter()
            .filter(|e| e.physics.is_some())
            .count();

        assert_eq!(with_physics, 2);
    }

    #[test]
    fn game_state_defaults() {
        let state = GameState::new();

        assert_eq!(state.score, 0.0);
        assert!(!state.game_over);
    }

    #[test]
    fn systems_are_executed_in_order() {
        fn system_a(_: &mut World, state: &mut GameState, _: &Input) {
            state.score += 1.0;
        }

        fn system_b(_: &mut World, state: &mut GameState, _: &Input) {
            state.score *= 2.0;
        }

        let world = World::new();
        let mut game = Game::new(world)
            .with_update_system(system_a)
            .with_update_system(system_b);

        let input = Input {
            dt: 1.0,
            spacebar: false,
        };

        game.update(&input);

        // (0 + 1) * 2 = 2
        assert_eq!(game.state.score, 2.0);
    }
}
