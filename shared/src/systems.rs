use crate::*;

pub fn movement_system(entities: &mut [Entity], input: &components::Input) {
    for entity in entities {
        let Some(ref movement) = entity.movement else {
            continue;
        };
        let should_not_move = matches!(entity.collide, Some(ref c) if c.is_collided);

        if !should_not_move {
            entity.transform.x += movement.velocity.x * input.dt;
            entity.transform.y += movement.velocity.y * input.dt;
        };
    }
}

pub fn collide_system(entities: &mut [Entity]) {
    let len = entities.len();

    for i in 0..len {
        for j in i + 1..len {
            let (a, b) = {
                let (left, right) = entities.split_at_mut(j);
                (&mut left[i], &mut right[0])
            };

            let Some(ref mut col_1) = a.collide else {
                continue;
            };
            let Some(ref mut col_2) = b.collide else {
                continue;
            };
            col_1.is_collided = false;
            col_2.is_collided = false;

            if a.transform.overlaps(&b.transform) {
                col_1.is_collided = true;
                col_2.is_collided = true;
            }
        }
    }
}

pub fn jump_system(entities: &mut [Entity], input: &components::Input) {
    for entity in entities {
        let Some(jump) = &mut entity.jump else {
            continue;
        };

        if entity.transform.y >= jump.ground_level {
            if input.is_jump {
                jump.velocity.y = jump.force;
            } else {
                jump.velocity.y = 0.0;
            }
        } else if entity.transform.y < jump.ground_level {
            jump.velocity.y -= jump.gravity * input.dt;
        }
        entity.transform.x += jump.velocity.x * input.dt;
        entity.transform.y -= jump.velocity.y * input.dt;
    }
}
