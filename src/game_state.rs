use ggez::{
    event::EventHandler,
    graphics::{self, Color, DrawMode, DrawParam, MeshBuilder, Rect},
    input::{self, keyboard::KeyCode},
    Context, GameResult,
};
use specs::prelude::*;

use crate::components::*;
use crate::systems;

pub struct GameState<'a, 'b> {
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> GameState<'a, 'b> {
    pub fn new(mut world: World, dispatcher: Dispatcher<'a, 'b>) -> Self {
        let mut init_star_sys = systems::StarInitSys::default();
        specs::RunNow::setup(&mut init_star_sys, &mut world);
        init_star_sys.run_now(&world);
        GameState { world, dispatcher }
    }
}

impl EventHandler for GameState<'_, '_> {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // std::thread::sleep(std::time::Duration::from_millis(50));
        if ggez::timer::ticks(&ctx) % 120 == 0 {
            dbg!(ggez::timer::fps(&ctx));
        }

        // for stuff to load in
        if ggez::timer::ticks(&ctx) < 30 {
            return Ok(());
        }

        if input::keyboard::is_key_pressed(ctx, KeyCode::Space) {
            let mut spawn_sys = systems::SpawnBulletSys::default();
            spawn_sys.run_now(&self.world);
        }

        {
            let num_enemies = {
                let enemies = self.world.read_storage::<Enemy>();
                enemies.join().count()
            };
            if num_enemies == 0 {
                let frames_to_next_wave = &mut self.world.fetch_mut::<FramesToNextWave>().0;
                if *frames_to_next_wave != 0 {
                    *frames_to_next_wave -= 1;
                } else {
                    {
                        let current_wave = &mut self.world.fetch_mut::<CurrentWave>().0;
                        *current_wave += 1;
                    }

                    {
                        let mut wave_calc_sys = systems::WaveCalcSys::default();
                        wave_calc_sys.run_now(&self.world);
                    }

                    let queued_enemies = &self.world.fetch::<QueuedEnemies>().0;
                    let mut positions = self.world.write_storage::<Position>();
                    let mut vels = self.world.write_storage::<Velocity>();
                    let mut enemies = self.world.write_storage::<Enemy>();
                    let mut hp_storage = self.world.write_storage::<HP>();
                    let mut hitboxes = self.world.write_storage::<Hitbox>();
                    let mut sprites = self.world.write_storage::<Sprite>();

                    queued_enemies.iter().for_each(|(pos, et)| {
                        let enemy = new_enemy(*et, *pos, MovementType::HLine(0.0..0.0, 0.0));
                        let sprite = self
                            .world
                            .fetch::<Sprites>()
                            .0
                            .get(&enemy.5)
                            .unwrap()
                            .clone();

                        self.world
                            .entities()
                            .build_entity()
                            .with(enemy.0, &mut positions)
                            .with(enemy.1, &mut vels)
                            .with(enemy.2.clone(), &mut enemies)
                            .with(enemy.3, &mut hp_storage)
                            .with(enemy.4, &mut hitboxes)
                            .with(Sprite::Img(sprite.clone()), &mut sprites)
                            .build();

                        let pos_2 =
                            Point::new(crate::SCREEN_WIDTH - 60.0 - (enemy.0).0.x, (enemy.0).0.y);

                        self.world
                            .entities()
                            .build_entity()
                            .with(Position(pos_2), &mut positions)
                            .with(enemy.1, &mut vels)
                            .with(enemy.2, &mut enemies)
                            .with(enemy.3, &mut hp_storage)
                            .with(enemy.4, &mut hitboxes)
                            .with(Sprite::Img(sprite), &mut sprites)
                            .build();
                    });
                }
            } else {
                if self.world.fetch::<FramesToNextWave>().0 == 0 {
                    self.world.insert(FramesToNextWave::default());
                }
            }
        }

        {
            let player_entity = self.world.fetch::<PlayerEntity>().0;
            let velocities = &mut self.world.write_storage::<Velocity>();
            let positions = &mut self.world.write_storage::<Position>();
            let player_vel = &mut velocities.get_mut(player_entity).unwrap().0;
            let player_pos = &mut positions.get_mut(player_entity).unwrap().0;

            *player_vel /= 1.45;
            if input::keyboard::is_key_pressed(ctx, KeyCode::W) && player_pos.y > 0.0 {
                player_vel.y -= 1.5;
            }
            if input::keyboard::is_key_pressed(ctx, KeyCode::S)
                && player_pos.y < crate::SCREEN_HEIGHT - 45.0
            {
                player_vel.y += 1.5;
            }
            if input::keyboard::is_key_pressed(ctx, KeyCode::A) && player_pos.x > 0.0 {
                player_vel.x -= 1.5;
            }
            if input::keyboard::is_key_pressed(ctx, KeyCode::D)
                && player_pos.x < crate::SCREEN_WIDTH - 45.0
            {
                player_vel.x += 1.5;
            }

            player_pos.y = player_pos.y.min(crate::SCREEN_HEIGHT - 45.0).max(0.0);
            player_pos.x = player_pos.x.min(crate::SCREEN_WIDTH - 45.0).max(0.0);
        }

        self.dispatcher.dispatch_par(&self.world);
        self.world.maintain();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));

        let mut builder = MeshBuilder::new();
        {
            let positions = self.world.read_storage::<Position>();
            let colorects = self.world.read_storage::<ColorRect>();
            let sprites = self.world.read_storage::<Sprite>();
            let stars = self.world.read_storage::<Star>();
            let hp_storage = self.world.read_storage::<HP>();
            let entities = self.world.entities();
            let animated_sprite_storage = self.world.read_storage::<AnimatedSprite>();
            let mut bullet_spritebatch = self.world.fetch_mut::<BulletSpriteBatch>();

            (&positions, &colorects, &stars)
                .join()
                .for_each(|(pos, colorect, _)| {
                    draw_colorect(&mut builder, (*pos).into(), &colorect);
                });

            (&positions, &colorects, !&stars)
                .join()
                .for_each(|(pos, colorect, _)| {
                    draw_colorect(&mut builder, (*pos).into(), &colorect);
                });

            (&positions, &sprites, &entities)
                .join()
                .for_each(|(pos, sprite, entity)| {
                    let draw_color = if let Some(hp) = hp_storage.get(entity) {
                        let opacity = (60 - hp.iframes as u32) as f32 / 60.0;
                        Color::new(1.0, 1.0, 1.0, opacity.powi(5))
                    } else {
                        graphics::WHITE
                    };
                    match sprite {
                        Sprite::Img(img) => {
                            graphics::draw(
                                ctx,
                                img,
                                graphics::DrawParam::new()
                                    .scale([3.0, 3.0])
                                    .dest(pos.0)
                                    .color(draw_color),
                            )
                            .unwrap();
                        }
                        Sprite::SpriteSheetInstance(spritesheet, index) => {
                            let frame_width = 1.0 / spritesheet.lock().unwrap().width as f32;
                            let src_rect =
                                Rect::new(frame_width * *index as f32, 0.0, frame_width, 1.0);
                            spritesheet.lock().unwrap().batch.add(
                                DrawParam::new()
                                    .src(src_rect)
                                    .scale([3.0, 3.0])
                                    .dest(pos.0)
                                    .color(draw_color),
                            );
                        }
                    }
                });

            (&positions, &animated_sprite_storage)
                .join()
                .for_each(|(pos, animated_sprite)| {
                    let frame_width = 1.0 / animated_sprite.spritesheet_width as f32;
                    let src_rect = Rect::new(
                        animated_sprite.current_frame as f32 * frame_width,
                        0.0,
                        frame_width,
                        1.0,
                    );
                    graphics::draw(
                        ctx,
                        &animated_sprite.spritesheet,
                        graphics::DrawParam::new()
                            .src(src_rect)
                            .scale([3.5, 3.5])
                            .dest(pos.0),
                    )
                    .unwrap();
                });

            if cfg!(feature = "draw_hitboxes") {
                let hitboxes = self.world.read_storage::<Hitbox>();
                (&positions, &hitboxes).join().for_each(|(pos, hitbox)| {
                    let rect = Rect::new(
                        pos.0.x + hitbox.0.x,
                        pos.0.y + hitbox.0.y,
                        hitbox.1,
                        hitbox.2,
                    );
                    builder.rectangle(DrawMode::stroke(2.5), rect, Color::new(1.0, 0.0, 0.0, 1.0));
                });
            }

            graphics::draw(ctx, &bullet_spritebatch.0, graphics::DrawParam::new())?;
            bullet_spritebatch.0.clear();
        }

        {
            let spritesheets = self.world.get_mut::<SpriteSheets>().unwrap();
            spritesheets.0.values_mut().for_each(|spritesheet| {
                let mut spritesheet = spritesheet.lock().unwrap();
                graphics::draw(ctx, &spritesheet.batch, graphics::DrawParam::new()).unwrap();
                spritesheet.batch.clear();
            });
        }

        let mesh = builder.build(ctx)?;
        graphics::draw(ctx, &mesh, DrawParam::new())?;

        graphics::present(ctx).unwrap();
        Ok(())
    }
}

fn draw_colorect(builder: &mut MeshBuilder, pos: Point, colorect: &ColorRect) {
    let rect = Rect::new(pos.x, pos.y, colorect.w, colorect.h);
    builder.rectangle(DrawMode::fill(), rect, colorect.color);
}
