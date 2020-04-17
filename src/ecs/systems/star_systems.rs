#![allow(clippy::type_complexity)]
use crate::ecs::components::*;
use crate::ecs::resources::*;
use specs::prelude::*;

#[derive(Default)]
pub struct StarInitSys;
impl<'a> System<'a> for StarInitSys {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Star>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, ColorRect>,
        Read<'a, StarInfo>,
    );

    fn run(
        &mut self,
        (entities, mut stars, mut positions, mut vels, mut colorects, star_info): Self::SystemData,
    ) {
        (0..star_info.num_stars).for_each(|_| {
            let mut star = star_info.new_star();
            (star.0).0.y += crate::SCREEN_HEIGHT * 0.9;

            entities
                .build_entity()
                .with(Star::default(), &mut stars)
                .with(star.0, &mut positions)
                .with(star.1, &mut vels)
                .with(star.2, &mut colorects)
                .build();
        });
    }
}

pub struct StarMoveSys;
impl<'a> System<'a> for StarMoveSys {
    type SystemData = (
        ReadStorage<'a, Star>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, ColorRect>,
        Read<'a, StarInfo>,
    );

    fn run(
        &mut self,
        (stars, mut positions, mut vels, mut colorects, star_info): Self::SystemData,
    ) {
        (&stars, &mut positions, &mut vels, &mut colorects)
            .par_join()
            .for_each(|(_, pos, vel, colorect)| {
                if pos.0.y > crate::SCREEN_HEIGHT {
                    let (npos, nvel, ncolorect) = star_info.new_star();
                    *pos = npos;
                    *vel = nvel;
                    *colorect = ncolorect;
                }
            });
    }
}
