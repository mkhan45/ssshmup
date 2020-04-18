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
        WriteStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, Bullet>,
        WriteStorage<'a, Sprite>,
        WriteStorage<'a, Hitbox>,
        Entities<'a>,
        Read<'a, PlayerEntity>,
        Read<'a, SpriteSheets>,
        Read<'a, Sounds>,
        Write<'a, QueuedSounds>,
    );

    fn run(
        &mut self,
        (
            mut players,
            mut positions,
            mut vels,
            mut bullets,
            mut sprite_res,
            mut hitboxes,
            entities,
            player_entity,
            spritesheets,
            sounds,
            mut queued_sounds,
        ): Self::SystemData,
    ) {
        let player_data = &mut players.get_mut(player_entity.0).unwrap();
        let player_vel = vels.get(player_entity.0).unwrap().0;

        if player_data.reload_timer == 0 {
            let sound = sounds.0.get("shoot").unwrap();
            queued_sounds.0.push(sound.clone());

            player_data.reload_timer = player_data.reload_speed;
            let player_pos = positions.get(player_entity.0).unwrap().0;
            let bullet_pos: Point = player_pos + Vector::new(12.5, 5.0);
            let bullet = new_bullet(
                player_data.bullet_type,
                bullet_pos,
                [0.0, -5.0 + player_vel.y.min(0.0)].into(),
                DamagesWho::Enemy,
            );

            let spritesheet = spritesheets.0.get("bullets").unwrap().clone();

            entities
                .build_entity()
                .with(bullet.0, &mut positions)
                .with(bullet.1, &mut hitboxes)
                .with(bullet.2, &mut vels)
                .with(bullet.3, &mut bullets)
                .with(
                    Sprite::SpriteSheetInstance(spritesheet, bullet.4),
                    &mut sprite_res,
                )
                .build();
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

        let player_pos = positions.get(player_entity.0).unwrap().0;
        let player_hitbox = hitboxes.get(player_entity.0).unwrap();
        let player_vel = velocities.get_mut(player_entity.0).unwrap();
        let mut player_hp = *hp_storage.get(player_entity.0).unwrap();

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

        *hp_storage.get_mut(player_entity.0).unwrap() = player_hp;
    }
}
