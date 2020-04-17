#![allow(clippy::type_complexity)]
use crate::ecs::components::*;
use crate::ecs::resources::*;
use specs::prelude::*;

use ggez::graphics::Rect;

pub struct IntegrateSys;
impl<'a> System<'a> for IntegrateSys {
    type SystemData = (WriteStorage<'a, Position>, ReadStorage<'a, Velocity>);

    fn run(&mut self, (mut positions, vels): Self::SystemData) {
        (&mut positions, &vels).par_join().for_each(|(pos, vel)| {
            pos.0 += vel.0;
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
        Write<'a, HPText>,
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
            mut hp_text,
        ): Self::SystemData,
    ) {
        let mut explosion_positions: Vec<Point> = Vec::new();

        (&bullets, &positions, &hitboxes, &entities)
            .join()
            .for_each(|(bullet, pos, bullet_hitbox, bullet_entity)| {
                let bullet_rect = Rect::new(
                    pos.0.x + bullet_hitbox.0.x,
                    pos.0.y + bullet_hitbox.0.y,
                    bullet_hitbox.1,
                    bullet_hitbox.2,
                );
                if !(-10.0..crate::SCREEN_WIDTH).contains(&pos.0.x)
                    || !(-10.0..crate::SCREEN_HEIGHT).contains(&pos.0.y)
                {
                    entities.delete(bullet_entity).unwrap();
                } else {
                    (&mut hp_storage, &positions, &hitboxes, &entities)
                        .join()
                        .for_each(|(hp, collided_pos, hitbox, entity)| {
                            if (bullet.damages_player() && entity == player_entity.0)
                                || (bullet.damages_enemy() && entity != player_entity.0)
                                    && hp.remaining > 0
                            {
                                let collidee_rect = Rect::new(
                                    collided_pos.0.x + hitbox.0.x,
                                    collided_pos.0.y + hitbox.0.y,
                                    hitbox.1,
                                    hitbox.2,
                                );
                                if bullet_rect.overlaps(&collidee_rect) {
                                    if hp.remaining >= bullet.damage {
                                        hp.remaining -= bullet.damage;
                                    } else {
                                        hp.remaining = 0;
                                    }
                                    if entity == player_entity.0 {
                                        hp_text.needs_redraw = true;
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
                .with(
                    animated_sprites.0.get("explosion").unwrap().clone(),
                    &mut animated_sprite_storage,
                )
                .build();
        });
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
                if animated_sprite.current_frame == animated_sprite.num_frames {
                    if animated_sprite.temporary {
                        entities.delete(entity).unwrap();
                    } else {
                        animated_sprite.current_frame = 0;
                    }
                }
            })
    }
}

pub struct BulletTrackingSys;
impl<'a> System<'a> for BulletTrackingSys {
    type SystemData = (
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Bullet>,
        Read<'a, PlayerEntity>,
    );

    fn run(&mut self, (mut vels, positions, bullets, player_entity): Self::SystemData) {
        if let Some(player_pos) = positions.get(player_entity.0) {
            let player_pos = player_pos.0;
            (&mut vels, &positions, &bullets)
                .par_join()
                .filter(|(_, _, bullet)| bullet.ty == BulletType::TrackingBullet)
                .for_each(|(vel, pos, _)| {
                    let direction = (player_pos - pos.0).normalize();
                    let target_vel = direction * 8.0;
                    vel.0 += (target_vel - vel.0) * 0.02;
                });
        }
    }
}

pub struct HPKillSys;
impl<'a> System<'a> for HPKillSys {
    type SystemData = (
        ReadStorage<'a, HP>,
        Entities<'a>,
        Read<'a, PlayerEntity>,
        Write<'a, Dead>,
    );

    fn run(&mut self, (hp_storage, entities, player_entity, mut dead): Self::SystemData) {
        (&hp_storage, &entities).join().for_each(|(hp, entity)| {
            if hp.remaining == 0 {
                entities.delete(entity).unwrap();
                if entity == player_entity.0 {
                    dead.0 = true;
                }
            }
        });
    }
}
