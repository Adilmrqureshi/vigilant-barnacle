#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn movement_moves_entity_when_not_colliding() {
        let mut entities = vec![Entity::new(0.0, 0.0).with_move(10.0, 0.0)];

        let input = components::Input {
            dt: 1.0,
            is_jump: false,
        };

        movement_system(&mut entities, &input);

        assert_eq!(entities[0].transform.x, 10.0);
        assert_eq!(entities[0].transform.y, 0.0);
    }

    #[test]
    fn movement_stops_when_collided() {
        let mut entity = Entity::new(0.0, 0.0).with_move(10.0, 0.0).with_collide();

        entity.collide.as_mut().unwrap().is_collided = true;

        let mut entities = vec![entity];
        let input = components::Input {
            dt: 1.0,
            is_jump: false,
        };

        movement_system(&mut entities, &input);

        assert_eq!(entities[0].transform.x, 0.0);
    }

    #[test]
    fn jump_applies_force_when_grounded() {
        let mut entities = vec![Entity::new(0.0, 100.0).with_jump(300.0, 100.0)];

        let input = components::Input {
            dt: 1.0,
            is_jump: true,
        };

        jump_system(&mut entities, &input);

        // y decreases because you subtract velocity.y
        assert!(entities[0].transform.y < 100.0);
    }

    #[test]
    fn gravity_applies_when_airborne() {
        let mut entity = Entity::new(0.0, 90.0).with_jump(300.0, 100.0);

        // Simulate upward velocity
        entity.jump.as_mut().unwrap().velocity.y = 100.0;

        let mut entities = vec![entity];
        let input = components::Input {
            dt: 1.0,
            is_jump: false,
        };

        jump_system(&mut entities, &input);

        let jump = entities[0].jump.as_ref().unwrap();
        assert!(jump.velocity.y < 100.0);
    }

    #[test]
    fn collision_detects_overlap() {
        let mut entities = vec![
            Entity::new(0.0, 0.0).with_collide(),
            Entity::new(32.0, 32.0).with_collide(), // overlaps 64x64
        ];

        collide_system(&mut entities);

        assert!(entities[0].collide.as_ref().unwrap().is_collided);
        assert!(entities[1].collide.as_ref().unwrap().is_collided);
    }

    #[test]
    fn collision_not_detected_when_separated() {
        let mut entities = vec![
            Entity::new(0.0, 0.0).with_collide(),
            Entity::new(200.0, 200.0).with_collide(),
        ];

        collide_system(&mut entities);

        assert!(!entities[0].collide.as_ref().unwrap().is_collided);
        assert!(!entities[1].collide.as_ref().unwrap().is_collided);
    }

    #[test]
    fn world_update_runs_all_systems() {
        let mut world = World::new();

        world.spawn(
            Entity::new(0.0, 100.0)
                .with_jump(200.0, 100.0)
                .with_move(10.0, 0.0)
                .with_collide(),
        );

        let input = components::Input {
            dt: 1.0,
            is_jump: true,
        };

        world.update(&input);

        let e = &world.entities[0];
        assert!(e.transform.x > 0.0);
        assert!(e.transform.y < 100.0);
    }
}
