#![allow(clippy::type_complexity)]
use crate::ecs::components::*;
use crate::ecs::resources::*;
use specs::prelude::*;

pub struct ReloadTimerSys;
impl<'a> System<'a> for ReloadTimerSys {
    type SystemData = (WriteStorage<'a, Player>, Read<'a, PlayerEntity>);

    fn run(&mut self, (mut players, player_entity): Self::SystemData) {
        if let Some(player_data) = &mut players.get_mut(player_entity.0) {
            if player_data.reload_timer != 0 {
                player_data.reload_timer -= 1;
            }
        }
    }
}

pub struct EnemyMoveSys;
impl<'a> System<'a> for EnemyMoveSys {
    type SystemData = (
        ReadStorage<'a, Position>,
        WriteStorage<'a, Enemy>,
        WriteStorage<'a, Velocity>,
    );

    fn run(&mut self, (positions, mut enemies, mut velocities): Self::SystemData) {
        (&mut enemies, &positions, &mut velocities)
            .join()
            .for_each(|(enemy, pos, vel)| match &mut enemy.movement {
                MovementType::HLine(range, _) => {
                    if !range.contains(&pos.0.x) {
                        vel.0.x *= -1.0;
                    }
                }
                MovementType::VLine(range, _) => {
                    if !range.contains(&pos.0.y) {
                        vel.0.y *= -1.0;
                    }
                }
                MovementType::Circle(_, rad, speed, angle) => {
                    // TODO some math here
                    vel.0.y = angle.sin() * *rad;
                    vel.0.x = angle.cos() * *rad;
                    *angle += *speed / 20.0 * std::f32::consts::PI;
                }
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
        WriteStorage<'a, Hitbox>,
        Entities<'a>,
        Read<'a, SpriteSheets>,
        Read<'a, PlayerEntity>,
        Read<'a, Dead>,
    );

    fn run(
        &mut self,
        (
            mut positions,
            mut enemies,
            mut vels,
            mut bullets,
            mut sprite_storage,
            mut hitboxes,
            entities,
            spritesheets,
            player_entity,
            dead,
        ): Self::SystemData,
    ) {
        if dead.0 {
            return;
        }

        let new_bullets: Vec<(Point, BulletType)> = (&positions, &mut enemies)
            .par_join()
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
            let vel = match bullet_type {
                BulletType::PlayerBullet => unreachable!(),
                BulletType::BasicBullet => [0.0, 8.0].into(),
                BulletType::AimedBullet | BulletType::TrackingBullet => {
                    let speed = match bullet_type {
                        BulletType::AimedBullet => 9.0,
                        BulletType::TrackingBullet => 5.0,
                        _ => unreachable!(),
                    };
                    (player_pos - pos).normalize() * speed
                }
                BulletType::PredictBullet => {
                    let bullet_speed = 13.0f32;

                    let mut player_projected_pos = player_pos;

                    (0..2).for_each(|_| {
                        let player_vec = player_projected_pos - pos;
                        let dist_to_player = player_vec.norm();
                        let time_to_hit = dist_to_player / bullet_speed;

                        player_projected_pos = player_pos + player_vel * time_to_hit;
                    });

                    let direction = (player_projected_pos - pos).normalize();

                    direction * bullet_speed
                }
            };
            let bullet_tuple = new_bullet(
                *bullet_type,
                *pos + Vector::new(36.0, 72.0),
                vel,
                DamagesWho::Player,
            );
            let spritesheet = spritesheets.0.get("bullets").unwrap().clone();
            entities
                .build_entity()
                .with(bullet_tuple.0, &mut positions)
                .with(bullet_tuple.1, &mut hitboxes)
                .with(bullet_tuple.3, &mut bullets)
                .with(bullet_tuple.2, &mut vels)
                .with(
                    Sprite::SpriteSheetInstance(spritesheet, bullet_tuple.4),
                    &mut sprite_storage,
                )
                .build();
        });
    }
}

#[derive(Default)]
pub struct WaveCalcSys;
impl<'a> System<'a> for WaveCalcSys {
    type SystemData = (Write<'a, QueuedEnemies>, Read<'a, CurrentWave>);

    fn run(&mut self, (mut queued_enemies, current_wave): Self::SystemData) {
        use std::collections::HashMap;

        let enemies = &mut queued_enemies.0;
        enemies.clear();

        let mut new_enemies = Vec::new();
        let mut counter: HashMap<EnemyType, u8> = HashMap::new();
        let target_difficulty = match current_wave.0 {
            1 => 12,
            2 => 14,
            3 => 20,
            4 => 24,
            _ => current_wave.0 as u16 * 5 + 5,
        };
        let mut difficulty = 0u16;

        fn calc_diff(ty: EnemyType) -> u16 {
            match ty {
                EnemyType::BasicEnemy => 1,
                EnemyType::BasicEnemy2 => 2,
                EnemyType::AimEnemy => 2,
                EnemyType::AimEnemy2 => 4,
                EnemyType::PredictEnemy => 5,
                EnemyType::TrackingEnemy => 5,
            }
        }

        while difficulty < target_difficulty {
            let new_enemy = [
                EnemyType::BasicEnemy,
                EnemyType::BasicEnemy2,
                EnemyType::AimEnemy,
                EnemyType::AimEnemy2,
                EnemyType::PredictEnemy,
                EnemyType::TrackingEnemy,
            ]
            .iter()
            .filter_map(|enemy_ty| {
                let diff = calc_diff(*enemy_ty);
                if diff < (target_difficulty - difficulty)
                    && (diff as f32) < target_difficulty as f32 / 4.0
                {
                    Some((enemy_ty, diff))
                } else {
                    None
                }
            })
            .max_by_key(|(ty, diff)| {
                *diff - (((*counter.get(ty).unwrap_or(&0) as u16).pow(2)) * 3).min(*diff)
            })
            .unwrap_or((&EnemyType::BasicEnemy, 1));
            difficulty += new_enemy.1 * 2;
            if let Some(count) = counter.get_mut(new_enemy.0) {
                *count += 1;
            } else {
                counter.insert(*new_enemy.0, 1);
            }
            new_enemies.push(new_enemy);
        }

        let mut new_enemies = new_enemies
            .iter()
            .enumerate()
            .map(|(i, (ty, _))| {
                (
                    [(i % 4) as f32 * 90.0, 20.0 + 100.0 * (i / 4) as f32].into(),
                    **ty,
                )
            })
            .collect();

        enemies.append(&mut new_enemies);
    }
}
