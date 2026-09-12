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

    // -----------------------------
    // Scene / Explore Tests
    // -----------------------------

    fn press_space() -> Input {
        Input {
            spacebar: true,
            ..Default::default()
        }
    }

    // E is mapped onto Input::a in main's input gathering.
    fn press_e() -> Input {
        Input {
            a: true,
            ..Default::default()
        }
    }

    #[test]
    fn space_lands_on_the_station_in_range() {
        let mut world = cv_world();
        let mut state = GameState::new();
        world.get_resource_mut::<Journey>().unwrap().distance = STATIONS[2].at;

        scene_system(&mut world, &mut state, &press_space());

        assert_eq!(scene(&world), Scene::Planet(2));
    }

    #[test]
    fn cannot_land_between_stations() {
        let mut world = cv_world();
        let mut state = GameState::new();
        world.get_resource_mut::<Journey>().unwrap().distance =
            (STATIONS[0].at + STATIONS[1].at) / 2.0;

        scene_system(&mut world, &mut state, &press_space());

        assert_eq!(scene(&world), Scene::Space);
    }

    #[test]
    fn e_returns_to_space() {
        let mut world = cv_world();
        let mut state = GameState::new();
        *world.get_resource_mut::<Scene>().unwrap() = Scene::Planet(1);

        scene_system(&mut world, &mut state, &press_e());

        assert_eq!(scene(&world), Scene::Space);
        let ship = world.with_tag(Tag::Player).next().unwrap();
        assert_eq!(ship.transform.x, SHIP_X);
    }

    #[test]
    fn travel_is_frozen_while_exploring() {
        let mut world = cv_world();
        let mut state = GameState::new();
        *world.get_resource_mut::<Scene>().unwrap() = Scene::Planet(0);

        travel_system(&mut world, &mut state, &one_second());

        assert_eq!(distance(&world), 0.0);
    }

    #[test]
    fn exploring_moves_the_ship_and_clamps_to_the_board() {
        let mut world = cv_world();
        let mut state = GameState::new();
        *world.get_resource_mut::<Scene>().unwrap() = Scene::Planet(0);

        let input = Input {
            dt: 100.0,
            right: true,
            down: true,
            ..Default::default()
        };
        explore_system(&mut world, &mut state, &input);

        let ship = world.with_tag(Tag::Player).next().unwrap();
        assert_eq!(ship.transform.x, BOARD_W - SHIP_W);
        assert_eq!(ship.transform.y, SHIP_MAX_Y);
    }

    #[test]
    fn explore_does_nothing_in_space() {
        let mut world = cv_world();
        let mut state = GameState::new();

        let input = Input {
            dt: 1.0,
            right: true,
            ..Default::default()
        };
        explore_system(&mut world, &mut state, &input);

        let ship = world.with_tag(Tag::Player).next().unwrap();
        assert_eq!(ship.transform.x, SHIP_X);
    }

    #[test]
    fn beacon_activates_only_when_near() {
        let p = &STATIONS[0].pois[0];
        assert_eq!(active_poi(0, p.x, p.y), Some(0));
        assert_eq!(active_poi(0, p.x + POI_RANGE + 1.0, p.y + POI_RANGE + 1.0), None);
    }

    #[test]
    fn every_station_has_beacons_on_the_board() {
        for s in &STATIONS {
            assert!(s.pois.len() >= 2);
            for p in s.pois {
                assert!(p.x > 0.0 && p.x < BOARD_W);
                assert!(p.y > 0.0 && p.y < 320.0, "keep beacons clear of the text panel");
                assert!(!p.lines.is_empty());
            }
        }
    }
}
