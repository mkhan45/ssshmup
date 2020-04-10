use crate::components::*;
use specs::prelude::*;

pub struct IntegrateSys;
impl<'a> System<'a> for IntegrateSys {
    type SystemData = (WriteStorage<'a, Position>, ReadStorage<'a, Velocity>);

    fn run(&mut self, (mut positions, vels): Self::SystemData) {
        (&mut positions, &vels).join().for_each(|(pos, vel)| {
            pos.0 += vel.0;
        });
    }
}

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
            let star = star_info.new_star();

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
            .join()
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
