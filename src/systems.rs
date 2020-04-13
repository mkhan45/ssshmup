#![allow(clippy::type_complexity)]
use crate::components::*;
use ggez::graphics::Rect;
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
            let bullet_pos: Point = player_pos + Vector::new(18.0, 5.0);
            let bullet = new_bullet(
                player_data.bullet_type,
                bullet_pos,
                [0.0, -5.0 + player_vel.y.min(0.0)].into(),
                true,
            );
            let sprite = {
                sprites
                    .0
                    .get(match player_data.bullet_type {
                        BulletType::BasicBullet => "bullet1",
                        BulletType::AimedBullet => "bullet1",
                        BulletType::PredictBullet => "bullet1",
                        BulletType::TrackingBullet => "bullet1",
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
    type SystemData = (ReadStorage<'a, HP>, Entities<'a>, Read<'a, PlayerEntity>);

    fn run(&mut self, (hp_storage, entities, player_entity): Self::SystemData) {
        (&hp_storage, &entities)
            .par_join()
            .for_each(|(hp, entity)| {
                if hp.remaining == 0 {
                    entities.delete(entity).unwrap();
                    if entity == player_entity.0 {
                        println!("Player died");
                        std::process::exit(0);
                    }
                }
            });
    }
}

pub struct IFrameSys;
impl<'a> System<'a> for IFrameSys {
    type SystemData = WriteStorage<'a, HP>;

    fn run(&mut self, mut hp_storage: Self::SystemData) {
        (&mut hp_storage).par_join().for_each(|hp| {
            if hp.iframes > 0 {
                hp.iframes -= 1;
            }
        });
    }
}

pub struct BulletCollSys;
impl<'a> System<'a> for BulletCollSys {
    type SystemData = (
        ReadStorage<'a, Bullet>,
        ReadStorage<'a, Hitbox>,
        WriteStorage<'a, HP>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, AnimatedSprite>,
        Entities<'a>,
        Read<'a, AnimatedSprites>,
        Read<'a, PlayerEntity>,
    );

    fn run(
        &mut self,
        (
            bullets,
            hitboxes,
            mut hp_storage,
            mut positions,
            mut animated_sprite_storage,
            entities,
            animated_sprites,
            player_entity,
        ): Self::SystemData,
    ) {
        let mut explosion_positions: Vec<Point> = Vec::new();

        (&bullets, &positions, &entities)
            .join()
            .for_each(|(bullet, pos, bullet_entity)| {
                let bullet_rect = Rect::new(pos.0.x, pos.0.y, 5.0, 5.0);
                if !(-10.0..crate::SCREEN_WIDTH).contains(&pos.0.x) || 
                    !(-10.0..crate::SCREEN_HEIGHT).contains(&pos.0.y) {
                    entities.delete(bullet_entity).unwrap();
                } else {
                    (&mut hp_storage, &positions, &hitboxes, &entities)
                        .join()
                        .for_each(|(hp, collided_pos, hitbox, entity)| {
                            if (bullet.friendly && entity != player_entity.0)
                                || (!bullet.friendly && entity == player_entity.0)
                                    && hp.remaining > 0
                            {
                                let collidee_rect = Rect::new(
                                    collided_pos.0.x,
                                    collided_pos.0.y,
                                    hitbox.0,
                                    hitbox.1,
                                );
                                if bullet_rect.overlaps(&collidee_rect) {
                                    if hp.remaining >= bullet.damage {
                                        hp.remaining -= bullet.damage;
                                    } else {
                                        hp.remaining = 0;
                                    }
                                    explosion_positions.push(pos.0 + Vector::new(-20.0, -20.0));
                                    entities.delete(bullet_entity).unwrap();
                                }
                            }
                        });
                }
            });

        explosion_positions.iter().for_each(|pos| {
            entities
                .build_entity()
                .with(Position(*pos), &mut positions)
                .with(animated_sprites.0.get("explosion").unwrap().clone(), &mut animated_sprite_storage,
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
        (&mut animated_sprite_storage, &entities)
            .join()
            .for_each(|(animated_sprite, entity)| {
                animated_sprite.current_frame += 1;
                if animated_sprite.current_frame == animated_sprite.num_frames
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

pub struct PlayerCollSys;
impl<'a> System<'a> for PlayerCollSys {
    type SystemData = (
        WriteStorage<'a, HP>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, Hitbox>,
        ReadStorage<'a, Bullet>,
        ReadStorage<'a, Enemy>,
        Entities<'a>,
        Read<'a, PlayerEntity>,
    );

    fn run(
        &mut self,
        (
            mut hp_storage,
            positions,
            mut velocities,
            hitboxes,
            bullets,
            enemies,
            entities,
            player_entity,
        ): Self::SystemData,
    ) {
        let player_pos = positions.get(player_entity.0).unwrap().0;
        let player_hitbox = hitboxes.get(player_entity.0).unwrap();
        let player_vel = velocities.get_mut(player_entity.0).unwrap();
        let mut player_hp = hp_storage.get(player_entity.0).unwrap().clone();

        let player_rect = Rect::new(player_pos.x, player_pos.y, player_hitbox.0, player_hitbox.1);
        (&mut hp_storage, &positions, &hitboxes, &entities, !&bullets)
            .join()
            .for_each(|(mut other_hp, pos, hbox, entity, _)| {
                if entity == player_entity.0 || other_hp.iframes > 0 || player_hp.iframes > 0 {
                    return;
                }

                let other_rect = Rect::new(pos.0.x, pos.0.y, hbox.0, hbox.1);
                if other_rect.overlaps(&player_rect) {
                    if let Some(enemy) = enemies.get(entity) {
                        let (damage_to_player, iframes) = match enemy.ty {
                            EnemyType::BasicEnemy => (1, 30),
                            EnemyType::AimEnemy => (1, 30),
                            EnemyType::PredictEnemy => (1, 30),
                            EnemyType::TrackingEnemy => (1, 30),
                        };
                        player_hp.remaining =
                            (player_hp.remaining as i16 - damage_to_player).max(0) as u32;
                        player_hp.iframes = iframes;

                        player_vel.0 += (player_pos - pos.0).normalize() * 20.0;
                    }
                    other_hp.remaining = (other_hp.remaining as i16 - 3).max(0) as u32;
                }
            });

        *hp_storage.get_mut(player_entity.0).unwrap() = player_hp;
    }
}

pub struct EnemyMoveSys;
impl<'a> System<'a> for EnemyMoveSys {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Enemy>,
        WriteStorage<'a, Velocity>,
    );

    fn run(&mut self, (positions, enemies, mut velocities): Self::SystemData) {
        (&enemies, &positions, &mut velocities)
            .join()
            .for_each(|(enemy, pos, vel)| match &enemy.movement {
                MovementType::HLine(range, _) => {
                    if !range.contains(&pos.0.x) {
                        vel.0.x *= -1.0;
                    }
                }
                _ => todo!(),
            });
    }
}

pub struct EnemyShootSys;
impl<'a> System<'a> for EnemyShootSys {
    type SystemData = (
        WriteStorage<'a, Position>,
        WriteStorage<'a, Enemy>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, Bullet>,
        WriteStorage<'a, Sprite>,
        Entities<'a>,
        Read<'a, Sprites>,
        Read<'a, PlayerEntity>,
    );

    fn run(
        &mut self,
        (
            mut positions,
            mut enemies,
            mut vels,
            mut bullets,
            mut sprite_storage,
            entities,
            sprites,
            player_entity,
        ): Self::SystemData,
    ) {
        let new_bullets: Vec<(Point, BulletType)> = (&positions, &mut enemies)
            .join()
            .filter_map(|(pos, mut enemy)| {
                if enemy.reload_timer != 0 {
                    enemy.reload_timer -= 1;
                    None
                } else {
                    enemy.reload_timer = enemy.reload_speed;
                    Some((pos.0, enemy.bullet_type))
                }
            })
        .collect();

        let player_pos = positions.get(player_entity.0).unwrap().0;
        let player_vel = vels.get(player_entity.0).unwrap().0;

        new_bullets.iter().for_each(|(pos, bullet_type)| {
            let (vel, sprite) = match bullet_type {
                BulletType::BasicBullet => {
                    ([0.0, 8.0].into(), sprites.0.get("bullet1").unwrap().clone())
                }
                BulletType::AimedBullet | BulletType::TrackingBullet => {
                    let speed = match bullet_type {
                        BulletType::AimedBullet => 9.0,
                        BulletType::TrackingBullet => 5.0,
                        _ => panic!("something has gone terribly wrong"),
                    };
                    let vel = (player_pos - pos).normalize() * speed;
                    (vel, sprites.0.get("bullet1").unwrap().clone())
                }
                BulletType::PredictBullet => {
                    let bullet_speed = 10.0f32;

                    let player_vec = player_pos - pos;
                    let dist_to_player = player_vec.norm();
                    let time_to_hit = dist_to_player / bullet_speed;

                    let player_projected_pos = player_pos + player_vel * time_to_hit; 
                    let direction = (player_projected_pos - pos).normalize();

                    let vel = direction * bullet_speed;
                    (vel, sprites.0.get("bullet1").unwrap().clone())
                }
            };
            let bullet_tuple = new_bullet(*bullet_type, *pos, vel, false);
            entities
                .build_entity()
                .with(bullet_tuple.0, &mut positions)
                .with(bullet_tuple.2, &mut bullets)
                .with(bullet_tuple.1, &mut vels)
                .with(Sprite(sprite), &mut sprite_storage)
                .build();
            });
    }
}

pub struct BulletTrackingSys;
impl<'a> System<'a> for BulletTrackingSys {
    type SystemData = (WriteStorage<'a, Velocity>, ReadStorage<'a, Position>, ReadStorage<'a, Bullet>, Read<'a, PlayerEntity>);

    fn run(&mut self, (mut vels, positions, bullets, player_entity): Self::SystemData) {
        let player_pos = positions.get(player_entity.0).unwrap().0;
        (&mut vels, &positions, &bullets).join().filter(|(_, _, bullet)| bullet.ty == BulletType::TrackingBullet).for_each(|(vel, pos, _)|{
            let direction = (player_pos - pos.0).normalize();
            let target_vel = direction * 8.0;
            vel.0 += (target_vel - vel.0) * 0.02;
        });
    }
}
