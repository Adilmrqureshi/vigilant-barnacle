#[cfg(test)]
mod tests {
    use shared_v2::{GameState, Input, Tag, World};

    use crate::*;

    // -----------------------------
    // Helpers
    // -----------------------------

    const PAD_X: f32 = 200.0;

    fn lander_world() -> World {
        let mut world = World::new();
        spawn_initial(&mut world, PAD_X);
        // Deterministic tests: no starting drift.
        for ship in world.with_tag_mut(Tag::Player) {
            ship.physics.as_mut().unwrap().velocity.x = 0.0;
        }
        world
    }

    fn one_second() -> Input {
        Input {
            dt: 1.0,
            ..Default::default()
        }
    }

    fn ship_vy(world: &World) -> f32 {
        let ship = world.with_tag(Tag::Player).next().unwrap();
        ship.physics.as_ref().unwrap().velocity.y
    }

    fn put_ship_on_ground(world: &mut World, x: f32, vx: f32, vy: f32) {
        for ship in world.with_tag_mut(Tag::Player) {
            ship.transform.x = x;
            ship.transform.y = GROUND_Y - 1.0;
            let physics = ship.physics.as_mut().unwrap();
            physics.velocity.x = vx;
            physics.velocity.y = vy;
        }
    }

    // -----------------------------
    // Flight Tests
    // -----------------------------

    #[test]
    fn gravity_pulls_the_ship_down() {
        let mut world = lander_world();
        let mut state = GameState::new();

        physics_system(&mut world, &mut state, &one_second());

        assert_eq!(ship_vy(&world), GRAVITY);
    }

    #[test]
    fn thrust_fights_gravity_and_burns_fuel() {
        let mut world = lander_world();
        let mut state = GameState::new();

        let input = Input {
            up: true,
            dt: 1.0,
            ..Default::default()
        };
        control_system(&mut world, &mut state, &input);

        assert_eq!(ship_vy(&world), -MAIN_THRUST);
        let lander = world.get_resource::<LanderState>().unwrap();
        assert_eq!(lander.fuel, START_FUEL - MAIN_BURN);
        assert!(lander.thrusting);
    }

    #[test]
    fn no_thrust_without_fuel() {
        let mut world = lander_world();
        let mut state = GameState::new();

        world.get_resource_mut::<LanderState>().unwrap().fuel = 0.0;
        let input = Input {
            up: true,
            dt: 1.0,
            ..Default::default()
        };
        control_system(&mut world, &mut state, &input);

        assert_eq!(ship_vy(&world), 0.0);
        assert!(!world.get_resource::<LanderState>().unwrap().thrusting);
    }

    // -----------------------------
    // Landing Tests
    // -----------------------------

    #[test]
    fn soft_landing_on_pad_wins() {
        let mut world = lander_world();
        let mut state = GameState::new();

        put_ship_on_ground(&mut world, PAD_X + 20.0, 0.0, MAX_LAND_VY - 1.0);
        landing_system(&mut world, &mut state, &one_second());

        assert!(state.game_over);
        assert!(world.get_resource::<LanderState>().unwrap().won);
        assert!(state.score > 0.0); // remaining fuel becomes the score
    }

    #[test]
    fn hard_landing_crashes_even_on_pad() {
        let mut world = lander_world();
        let mut state = GameState::new();

        put_ship_on_ground(&mut world, PAD_X + 20.0, 0.0, MAX_LAND_VY + 50.0);
        landing_system(&mut world, &mut state, &one_second());

        assert!(state.game_over);
        assert!(!world.get_resource::<LanderState>().unwrap().won);
    }

    #[test]
    fn soft_landing_off_pad_crashes() {
        let mut world = lander_world();
        let mut state = GameState::new();

        put_ship_on_ground(&mut world, PAD_X + PAD_W + 50.0, 0.0, 10.0);
        landing_system(&mut world, &mut state, &one_second());

        assert!(state.game_over);
        assert!(!world.get_resource::<LanderState>().unwrap().won);
    }

    #[test]
    fn drifting_sideways_too_fast_crashes() {
        let mut world = lander_world();
        let mut state = GameState::new();

        put_ship_on_ground(&mut world, PAD_X + 20.0, MAX_LAND_VX + 20.0, 10.0);
        landing_system(&mut world, &mut state, &one_second());

        assert!(!world.get_resource::<LanderState>().unwrap().won);
    }

    #[test]
    fn no_landing_while_airborne() {
        let mut world = lander_world();
        let mut state = GameState::new();

        landing_system(&mut world, &mut state, &one_second());

        assert!(!state.game_over);
    }
}
