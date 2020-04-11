#![allow(clippy::type_complexity)]
use crate::components::*;
use specs::prelude::*;

pub struct IntegrateSys;
impl<'a> System<'a> for IntegrateSys {
    type SystemData = (WriteStorage<'a, Position>, ReadStorage<'a, Velocity>);

    fn run(&mut self, (mut positions, vels): Self::SystemData) {
        (&mut positions, &vels).par_join().for_each(|(pos, vel)| {
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

#[derive(Default)]
pub struct SpawnBulletSys;

impl<'a> System<'a> for SpawnBulletSys {
    type SystemData = (
        WriteStorage<'a, Player>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, Bullet>,
        WriteStorage<'a, Sprite>,
        Entities<'a>,
        Read<'a, PlayerEntity>,
        Read<'a, Sprites>,
    );

    fn run(
        &mut self,
        (
            mut players,
            mut positions,
            mut vels,
            mut bullets,
            mut sprite_res,
            entities,
            player_entity,
            sprites,
        ): Self::SystemData,
    ) {
        let player_data = &mut players.get_mut(player_entity.0).unwrap();
        let player_vel = vels.get(player_entity.0).unwrap().0;

        if player_data.reload_timer == 0 {
            player_data.reload_timer = player_data.reload_speed;
            let player_pos = positions.get(player_entity.0).unwrap().0;
            let bullet = new_bullet(player_data.bullet_type, player_pos, player_vel);
            let sprite = {
                sprites
                    .0
                    .get(match player_data.bullet_type {
                        BulletType::BasicBullet => "bullet1",
                    })
                    .unwrap()
                    .clone()
            };

            entities
                .build_entity()
                .with(bullet.0, &mut positions)
                .with(bullet.1, &mut vels)
                .with(bullet.2, &mut bullets)
                .with(Sprite(sprite), &mut sprite_res)
                .build();
        }
    }
}

pub struct HPKillSys;
impl<'a> System<'a> for HPKillSys {
    type SystemData = (ReadStorage<'a, HP>, Entities<'a>);

    fn run(&mut self, (hp_storage, entities): Self::SystemData) {
        (&hp_storage, &entities)
            .par_join()
            .for_each(|(hp, entity)| {
                if hp.0 == 0 {
                    entities.delete(entity).unwrap();
                }
            });
    }
}

pub struct BulletCollSys;
impl<'a> System<'a> for BulletCollSys {
    type SystemData = (
        ReadStorage<'a, Enemy>,
        ReadStorage<'a, Bullet>,
        WriteStorage<'a, HP>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, AnimatedSprite>,
        Entities<'a>,
        Read<'a, AnimatedSprites>,
    );

    fn run(
        &mut self,
        (
            enemies,
            bullets,
            mut hp_storage,
            mut positions,
            mut animated_sprite_storage,
            entities,
            animated_sprites,
        ): Self::SystemData,
    ) {
        let mut explosion_positions: Vec<Point> = Vec::new();

        (&bullets, &positions, &entities)
            .join()
            .for_each(|(bullet, pos, bullet_entity)| {
                if pos.0.y <= -20.0 {
                    entities.delete(bullet_entity).unwrap();
                } else {
                    (&enemies, &mut hp_storage, &positions).join().for_each(
                        |(_enemy, enemy_hp, enemy_pos)| {
                            //  _____
                            // |     |
                            // |_____| <-- enemy
                            //  ---* <-- bullet
                            //   ^
                            //   |
                            // x_diff
                            let x_diff = pos.0.x - enemy_pos.0.x;
                            let y_diff = enemy_pos.0.y - pos.0.y;
                            if enemy_hp.0 > 0
                                && (-5.0..50.0).contains(&x_diff)
                                && (-5.0..5.0).contains(&y_diff)
                            {
                                enemy_hp.0 -= bullet.damage;
                                explosion_positions.push(pos.0);
                                entities.delete(bullet_entity).unwrap();
                            }
                        },
                    );
                }
            });

        explosion_positions.iter().for_each(|pos| {
            entities
                .build_entity()
                .with(Position(*pos), &mut positions)
                .with(
                    AnimatedSprite {
                        frames: animated_sprites.0.get("explosion").unwrap().to_vec(),
                        current_frame: 0,
                        temporary: true,
                    },
                    &mut animated_sprite_storage,
                )
                .build();
        });
    }
}

pub struct ReloadTimerSys;
impl<'a> System<'a> for ReloadTimerSys {
    type SystemData = (WriteStorage<'a, Player>, Read<'a, PlayerEntity>);

    fn run(&mut self, (mut players, player_entity): Self::SystemData) {
        let player_data = &mut players.get_mut(player_entity.0).unwrap();

        if player_data.reload_timer != 0 {
            player_data.reload_timer -= 1;
        }
    }
}

pub struct AnimationSys;
impl<'a> System<'a> for AnimationSys {
    type SystemData = (WriteStorage<'a, AnimatedSprite>, Entities<'a>);

    fn run(&mut self, (mut animated_sprite_storage, entities): Self::SystemData) {
        use std::convert::TryInto;

        (&mut animated_sprite_storage, &entities)
            .join()
            .for_each(|(animated_sprite, entity)| {
                animated_sprite.current_frame += 1;
                assert!(animated_sprite.frames.len() < 256);
                if animated_sprite.current_frame == animated_sprite.frames.len().try_into().unwrap()
                {
                    if animated_sprite.temporary {
                        entities.delete(entity).unwrap();
                    } else {
                        animated_sprite.current_frame = 0;
                    }
                }
            })
    }
}
