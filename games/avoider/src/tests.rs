#[cfg(test)]
mod tests {
    use macroquad::math::Rect;
    use shared_v2::*;

    use crate::*;

    const EPS: f32 = 0.001;

    fn player_entity(y: f32, grounded: bool) -> Entity {
        Entity {
            transform: Rect {
                x: 0.0,
                y,
                w: 10.0,
                h: 10.0,
            },
            tag: Some(Tag::Player),
            render: None,
            sprite: None,
            physics: Some(Physics {
                is_grounded: grounded,
                velocity: Velocity { x: 0.0, y: 0.0 },
            }),
        }
    }

    #[test]
    fn gravity_applies_when_airborne() {
        let mut world = World {
            entities: vec![player_entity(GROUND + 100.0, false)],
        };

        let mut state = GameState::new();
        let input = Input {
            dt: 0.016,
            spacebar: false,
        };

        gravity_engine(&mut world, &mut state, &input);

        let e = &world.entities[0];
        let physics = e.physics.as_ref().unwrap();

        assert!(physics.velocity.y < 0.0);
        assert!(e.transform.y < GROUND + 100.0);
    }

    #[test]
    fn jump_sets_upward_velocity_when_grounded() {
        let mut world = World {
            entities: vec![player_entity(GROUND, true)],
        };

        let mut state = GameState::new();
        let input = Input {
            dt: 0.016,
            spacebar: true,
        };

        gravity_engine(&mut world, &mut state, &input);

        let physics = world.entities[0].physics.as_ref().unwrap();

        assert!(!physics.is_grounded);
        assert!(physics.velocity.y > 0.0);
    }

    #[test]
    fn jump_does_not_trigger_midair() {
        let mut world = World {
            entities: vec![player_entity(GROUND + 50.0, false)],
        };

        let mut state = GameState::new();
        let input = Input {
            dt: 0.016,
            spacebar: true,
        };

        gravity_engine(&mut world, &mut state, &input);

        let physics = world.entities[0].physics.as_ref().unwrap();

        // gravity applied, but no jump impulse
        assert!(physics.velocity.y < 0.0);
    }

    #[test]
    fn entity_lands_and_resets_velocity() {
        let mut world = World {
            entities: vec![player_entity(GROUND - 1.0, false)],
        };

        let mut state = GameState::new();
        let input = Input {
            dt: 0.016,
            spacebar: false,
        };

        gravity_engine(&mut world, &mut state, &input);

        let physics = world.entities[0].physics.as_ref().unwrap();

        assert!(physics.is_grounded);
        assert!((physics.velocity.y).abs() < EPS);
    }
}
