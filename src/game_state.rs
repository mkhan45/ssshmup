use ggez::{
    event::EventHandler,
    graphics::{self, Color, DrawMode, DrawParam, MeshBuilder, Rect},
    input::{self, keyboard::KeyCode},
    Context, GameResult,
};
use specs::prelude::*;

use crate::components::*;
use crate::systems;

use rand::prelude::*;

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
        if ggez::timer::ticks(&ctx) < 5 {
            return Ok(());
        }

        if input::keyboard::is_key_pressed(ctx, KeyCode::Space) {
            let mut spawn_sys = systems::SpawnBulletSys::default();
            spawn_sys.run_now(&self.world);
        }

        {
            let hp_text = &mut self.world.fetch_mut::<HPText>();
            if hp_text.needs_redraw {
                hp_text.needs_redraw = false;

                let hp = {
                    let player = self.world.fetch::<PlayerEntity>().0;
                    self.world
                        .read_storage::<HP>()
                        .get(player)
                        .unwrap()
                        .remaining
                };

                let wave = self.world.fetch::<CurrentWave>().0;

                *hp_text.text.lock().unwrap() = {
                    use ggez::graphics::Scale;
                    let font = self.world.fetch::<GameFont>().0;
                    let mut text = graphics::Text::new(format!("HP: {}\nWave: {}", hp, wave));
                    text.set_font(font, Scale::uniform(48.0));
                    text
                };
            }
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
                    self.world.fetch_mut::<HPText>().needs_redraw = true;
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
                    let spritesheets = &self.world.fetch::<SpriteSheets>().0;
                    let spritesheet = spritesheets.get("enemies").unwrap().clone();

                    {
                        let player_hp = hp_storage
                            .get_mut(self.world.fetch::<PlayerEntity>().0)
                            .unwrap();
                        player_hp.remaining += 1;
                    }

                    let mut rng = rand::thread_rng();
                    queued_enemies.iter().for_each(|(pos, et)| {
                        let (mt1, mt2) = {
                            let mt = rng.gen_range(0, 2);
                            let mt2_x = crate::SCREEN_WIDTH - 90.0 - pos.x;
                            match mt {
                                0 => (
                                    MovementType::horizontal(pos.x, 75.0, 1.0),
                                    MovementType::horizontal(mt2_x, 75.0, 1.0),
                                ),
                                1 => (
                                    MovementType::vertical(pos.y, 90.0, 1.0),
                                    MovementType::vertical(pos.y, 90.0, 1.0),
                                ),
                                // 2 => (MovementType::circle(*pos, 60.0, 1.0), MovementType::circle(Point::new(mt2_x, pos.y), 60.0, 1.0)),
                                _ => unreachable!(),
                            }
                        };
                        let mut enemy = new_enemy(*et, *pos, mt1);

                        self.world
                            .entities()
                            .build_entity()
                            .with(enemy.0, &mut positions)
                            .with(enemy.1, &mut vels)
                            .with(enemy.2.clone(), &mut enemies)
                            .with(enemy.3, &mut hp_storage)
                            .with(enemy.4, &mut hitboxes)
                            .with(
                                Sprite::SpriteSheetInstance(spritesheet.clone(), enemy.5),
                                &mut sprites,
                            )
                            .build();

                        let pos_2 =
                            Point::new(crate::SCREEN_WIDTH - 90.0 - (enemy.0).0.x, (enemy.0).0.y);
                        enemy.2.movement = mt2;

                        self.world
                            .entities()
                            .build_entity()
                            .with(Position(pos_2), &mut positions)
                            .with(enemy.1, &mut vels)
                            .with(enemy.2, &mut enemies)
                            .with(enemy.3, &mut hp_storage)
                            .with(enemy.4, &mut hitboxes)
                            .with(
                                Sprite::SpriteSheetInstance(spritesheet.clone(), enemy.5),
                                &mut sprites,
                            )
                            .build();
                    });
                }
            } else if self.world.fetch::<FramesToNextWave>().0 == 0 {
                self.world.insert(FramesToNextWave::default());
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
                    if pos.0.y > 0.0 {
                        draw_colorect(&mut builder, (*pos).into(), &colorect);
                    }
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

        {
            let text_mutex = &self.world.fetch::<HPText>().text;
            let text = text_mutex.lock().unwrap();
            graphics::draw(
                ctx,
                &*text,
                graphics::DrawParam::new().dest([50.0, crate::SCREEN_HEIGHT - 100.0]),
            )
            .unwrap();
        }

        graphics::present(ctx).unwrap();
        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
        let aspect_ratio = height / width;
        let initial_ratio = SCREEN_HEIGHT / SCREEN_WIDTH;

        if initial_ratio > aspect_ratio {
            // width is greater than usual
            let new_width = SCREEN_WIDTH / aspect_ratio;
            let excess_width = new_width - SCREEN_WIDTH;
            ggez::graphics::set_screen_coordinates(
                ctx,
                graphics::Rect::new(-excess_width / 2.0, 0.0, new_width, SCREEN_HEIGHT),
            )
            .unwrap();
        } else {
            // height is greater than usual
            let new_height = SCREEN_HEIGHT * aspect_ratio;
            let excess_height = new_height - SCREEN_HEIGHT;
            ggez::graphics::set_screen_coordinates(
                ctx,
                graphics::Rect::new(0.0, -excess_height / 2.0, SCREEN_WIDTH, new_height),
            )
            .unwrap();
        }
    }
}

fn draw_colorect(builder: &mut MeshBuilder, pos: Point, colorect: &ColorRect) {
    let rect = Rect::new(pos.x, pos.y, colorect.w, colorect.h);
    builder.rectangle(DrawMode::fill(), rect, colorect.color);
}
