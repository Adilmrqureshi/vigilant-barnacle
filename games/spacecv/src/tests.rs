#[cfg(test)]
mod tests {
    use shared_v2::{GameState, Input, Tag, World};

    use crate::*;

    // -----------------------------
    // Helpers
    // -----------------------------

    fn cv_world() -> World {
        let mut world = World::new();
        spawn_initial(&mut world);
        world
    }

    fn distance(world: &World) -> f32 {
        world.get_resource::<Journey>().unwrap().distance
    }

    fn one_second() -> Input {
        Input {
            dt: 1.0,
            ..Default::default()
        }
    }

    // -----------------------------
    // Travel Tests
    // -----------------------------

    #[test]
    fn ship_cruises_forward_by_default() {
        let mut world = cv_world();
        let mut state = GameState::new();

        travel_system(&mut world, &mut state, &one_second());

        assert_eq!(distance(&world), CRUISE_SPEED);
    }

    #[test]
    fn boost_is_faster_than_cruise() {
        let mut world = cv_world();
        let mut state = GameState::new();

        let input = Input {
            dt: 1.0,
            right: true,
            ..Default::default()
        };
        travel_system(&mut world, &mut state, &input);

        assert_eq!(distance(&world), BOOST_SPEED);
        assert!(BOOST_SPEED > CRUISE_SPEED);
    }

    #[test]
    fn can_fly_back_but_not_before_the_start() {
        let mut world = cv_world();
        let mut state = GameState::new();

        let input = Input {
            dt: 1.0,
            left: true,
            ..Default::default()
        };
        travel_system(&mut world, &mut state, &input);

        assert_eq!(distance(&world), 0.0);
    }

    #[test]
    fn journey_stops_at_the_end() {
        let mut world = cv_world();
        let mut state = GameState::new();

        world.get_resource_mut::<Journey>().unwrap().distance = JOURNEY_END - 1.0;
        let input = Input {
            dt: 1.0,
            right: true,
            ..Default::default()
        };
        travel_system(&mut world, &mut state, &input);

        assert_eq!(distance(&world), JOURNEY_END);
    }

    #[test]
    fn steering_clamps_to_board() {
        let mut world = cv_world();
        let mut state = GameState::new();

        let input = Input {
            dt: 100.0,
            up: true,
            ..Default::default()
        };
        travel_system(&mut world, &mut state, &input);

        let ship = world.with_tag(Tag::Player).next().unwrap();
        assert_eq!(ship.transform.y, SHIP_MIN_Y);
    }

    // -----------------------------
    // Station Tests
    // -----------------------------

    #[test]
    fn station_activates_in_range() {
        assert_eq!(active_station(STATIONS[0].at), Some(0));
        assert_eq!(active_station(STATIONS[3].at + ACTIVATION / 2.0), Some(3));
    }

    #[test]
    fn no_station_between_stops() {
        let midway = (STATIONS[0].at + STATIONS[1].at) / 2.0;
        assert_eq!(active_station(midway), None);
        assert_eq!(active_station(0.0), None);
    }

    #[test]
    fn stations_are_in_travel_order() {
        for pair in STATIONS.windows(2) {
            assert!(pair[0].at < pair[1].at);
        }
        assert!(STATIONS.last().unwrap().at < JOURNEY_END);
    }
}
