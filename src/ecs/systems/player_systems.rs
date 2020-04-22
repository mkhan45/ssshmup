#![allow(clippy::type_complexity)]
use crate::ecs::components::*;
use crate::ecs::resources::*;
use specs::prelude::*;

use ggez::graphics::Rect;

#[derive(Default)]
pub struct SpawnBulletSys;
impl<'a> System<'a> for SpawnBulletSys {
    type SystemData = (
        WriteStorage<'a, Player>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Velocity>,
        Entities<'a>,
        Read<'a, PlayerEntity>,
        Read<'a, SpriteSheets>,
        Read<'a, Sounds>,
        Write<'a, QueuedSounds>,
        Read<'a, LazyUpdate>,
    );

    fn run(
        &mut self,
        (
            mut players,
            positions,
            vels,
            entities,
            player_entity,
            spritesheets,
            sounds,
            mut queued_sounds,
            lazy_update,
        ): Self::SystemData,
    ) {
        let player_data = &mut players
            .get_mut(player_entity.0)
            .expect("error getting player data");
        let player_vel = vels
            .get(player_entity.0)
            .expect("error getting player vel")
            .0;

        if player_data.reload_timer == 0 {
            if let Some(sound) = sounds.0.get("shoot") {
                queued_sounds.0.push(sound.clone());
            } else {
                log::warn!("error getting shot sound");
            }

            player_data.reload_timer = player_data.reload_speed;
            let player_pos = positions
                .get(player_entity.0)
                .expect("error getting player position")
                .0;
            let bullet_pos: Point = player_pos + Vector::new(12.0, 5.0);
            let bullet = new_bullet(
                player_data.bullet_type,
                bullet_pos,
                [0.0, -5.0 + player_vel.y.min(0.0)].into(),
                DamagesWho::Enemy,
            );

            let spritesheet = spritesheets
                .0
                .get("bullets")
                .expect("error getting bullet spritesheet")
                .clone();

            let entity = entities.create();
            lazy_update.insert(entity, bullet.0);
            lazy_update.insert(entity, bullet.1);
            lazy_update.insert(entity, bullet.2);
            lazy_update.insert(entity, bullet.3);
            lazy_update.insert(entity, Sprite::SpriteSheetInstance(spritesheet, bullet.4));
        }
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
        Read<'a, Dead>,
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
            dead,
        ): Self::SystemData,
    ) {
        if dead.0 {
            return;
        }

        let player_pos = positions
            .get(player_entity.0)
            .expect("error getting player pos")
            .0;
        let player_hitbox = hitboxes
            .get(player_entity.0)
            .expect("error getting player hitbox");
        let player_vel = velocities
            .get_mut(player_entity.0)
            .expect("error getting player vel");
        let mut player_hp = *hp_storage
            .get(player_entity.0)
            .expect("error getting player hp");

        let player_rect = Rect::new(
            player_pos.x + player_hitbox.0.x,
            player_pos.y + player_hitbox.0.y,
            player_hitbox.1,
            player_hitbox.2,
        );
        (&mut hp_storage, &positions, &hitboxes, &entities, !&bullets)
            .join()
            .for_each(|(mut other_hp, pos, hbox, entity, _)| {
                if entity == player_entity.0 || other_hp.iframes > 0 || player_hp.iframes > 0 {
                    return;
                }

                let other_rect = Rect::new(pos.0.x + hbox.0.x, pos.0.y + hbox.0.y, hbox.1, hbox.2);
                if other_rect.overlaps(&player_rect) {
                    if let Some(enemy) = enemies.get(entity) {
                        let (damage_to_player, iframes) = match enemy.ty {
                            _ => (1, 30),
                        };
                        player_hp.remaining =
                            (player_hp.remaining as i16 - damage_to_player).max(0) as u32;
                        player_hp.iframes = iframes;

                        player_vel.0 += (player_pos - pos.0).normalize() * 20.0;
                    }
                    other_hp.remaining = (other_hp.remaining as i16 - 3).max(0) as u32;
                }
            });

        *hp_storage
            .get_mut(player_entity.0)
            .expect("error getting player hp") = player_hp;
    }
}

pub struct DeflectorSys;
impl<'a> System<'a> for DeflectorSys {
    type SystemData = (
        WriteStorage<'a, Player>,
        WriteStorage<'a, Sprite>,
        Read<'a, Sprites>,
    );

    fn run(&mut self, (mut players, mut sprite_storage, sprites): Self::SystemData) {
        (&mut players, &mut sprite_storage)
            .join()
            .for_each(|(mut player, sprite)| {
                if player.deflector_timer > 0 {
                    player.deflector_timer -= 1;
                }
                if player.deflector_timer == 1 {
                    let player_cooldown_sprite = sprites
                        .0
                        .get("player_cooldown")
                        .expect("error getting player cooldown sprite");
                    *sprite = Sprite::Img(player_cooldown_sprite.clone());
                }
                if player.deflector_timer == player.deflector_frames - 1 {
                    let player_deflector_sprite = sprites
                        .0
                        .get("player_deflector")
                        .expect("error getting player deflector sprite");
                    *sprite = Sprite::Img(player_deflector_sprite.clone());
                }

                if player.deflector_cooldown > 0 {
                    player.deflector_cooldown -= 1;
                }
                if player.deflector_cooldown == 1 {
                    let player_default_sprite = sprites
                        .0
                        .get("player")
                        .expect("error getting player default sprite");
                    *sprite = Sprite::Img(player_default_sprite.clone());
                }
            });
    }
}
