use ggez::{
    event::EventHandler,
    graphics::{self, Color, DrawMode, DrawParam, MeshBuilder, Rect},
    input::{self, keyboard::KeyCode, keyboard::KeyMods},
    Context, GameResult,
};
use specs::prelude::*;

use crate::ecs::components::*;
use crate::ecs::resources::*;
use crate::ecs::systems;

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
        {
            let last_update = self.world.fetch::<LastUpdate>().0;
            let time_since_start = ggez::timer::time_since_start(ctx);

            let time_diff = time_since_start - last_update;
            if time_diff < std::time::Duration::from_millis(32) {
                return Ok(());
            }

            self.world.insert(LastUpdate(time_since_start));
        }

        if cfg!(feature = "print_fps") && ggez::timer::ticks(&ctx) % 120 == 0 {
            dbg!(ggez::timer::fps(&ctx));
        }

        // for stuff to load in
        if ggez::timer::ticks(&ctx) < 5 {
            return Ok(());
        }

        let dead = self.world.fetch::<Dead>().0;

        if !dead && input::keyboard::is_key_pressed(ctx, KeyCode::Space) {
            let mut spawn_sys = systems::SpawnBulletSys::default();
            spawn_sys.run_now(&self.world);
        }

        {
            use ggez::audio::Source;

            self.world
                .fetch_mut::<QueuedSounds>()
                .0
                .drain(..)
                .for_each(|sound_data| {
                    use ggez::audio::SoundSource;

                    if let Ok(mut source) = Source::from_data(ctx, sound_data) {
                        source.set_volume(0.2 * crate::VOLUME_MULTIPLIER);
                        if source.play_detached().is_err() {
                            log::warn!("Error playing sound");
                        }
                    } else {
                        log::warn!("Error initializing sound source")
                    }
                });
        }

        {
            let hp_text = &mut self.world.fetch_mut::<HPText>();
            if hp_text.needs_redraw {
                hp_text.needs_redraw = false;

                let hp = if dead {
                    0
                } else {
                    let player = self.world.fetch::<PlayerEntity>().0;
                    self.world
                        .read_storage::<HP>()
                        .get(player)
                        .expect("Error fetching player hp")
                        .remaining
                };

                let wave = self.world.fetch::<CurrentWave>().0;

                *hp_text.text.lock().expect("error locking hp_text") = {
                    use ggez::graphics::Scale;
                    let font = self.world.fetch::<GameFont>().0;
                    let mut text = graphics::Text::new(format!("     x {}\nWave: {}", hp, wave));
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
                    let spritesheet = spritesheets
                        .get("enemies")
                        .expect("error getting enemy spritesheet")
                        .clone();

                    if self.world.fetch::<CurrentWave>().0 != 1 {
                        if let Some(mut player_hp) =
                            hp_storage.get_mut(self.world.fetch::<PlayerEntity>().0)
                        {
                            player_hp.remaining += 1;
                        }
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

        if !dead {
            let player_entity = self.world.fetch::<PlayerEntity>().0;
            let velocities = &mut self.world.write_storage::<Velocity>();
            let positions = &mut self.world.write_storage::<Position>();
            let player_vel = &mut velocities
                .get_mut(player_entity)
                .expect("error getting player_vel")
                .0;
            let player_pos = &mut positions
                .get_mut(player_entity)
                .expect("error getting player pos")
                .0;

            *player_vel /= 1.45;

            let speed = if input::keyboard::is_key_pressed(ctx, KeyCode::Space) {
                1.3
            } else {
                1.7
            };

            if input::keyboard::is_key_pressed(ctx, KeyCode::W) && player_pos.y > 0.0 {
                player_vel.y -= speed;
            }
            if input::keyboard::is_key_pressed(ctx, KeyCode::S)
                && player_pos.y < crate::SCREEN_HEIGHT - 45.0
            {
                player_vel.y += speed;
            }
            if input::keyboard::is_key_pressed(ctx, KeyCode::A) && player_pos.x > 0.0 {
                player_vel.x -= speed;
            }
            if input::keyboard::is_key_pressed(ctx, KeyCode::D)
                && player_pos.x < crate::SCREEN_WIDTH - 45.0
            {
                player_vel.x += speed;
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
                            .expect("error drawing sprite");
                        }
                        Sprite::SpriteSheetInstance(spritesheet, index) => {
                            let mut spritesheet =
                                spritesheet.lock().expect("error locking spritesheet");
                            let frame_width = 1.0 / spritesheet.width as f32;
                            let src_rect =
                                Rect::new(frame_width * *index as f32, 0.0, frame_width, 1.0);
                            spritesheet.batch.add(
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
                    .expect("error drawing animated sprite");
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
        }

        {
            let spritesheets = self
                .world
                .get_mut::<SpriteSheets>()
                .expect("error getting spritesheet");
            spritesheets.0.values_mut().for_each(|spritesheet| {
                let mut spritesheet = spritesheet.lock().expect("error locking spritesheet");
                graphics::draw(ctx, &spritesheet.batch, graphics::DrawParam::new())
                    .expect("error drawing spritebatch");
                spritesheet.batch.clear();
            });
        }

        let mesh = builder.build(ctx)?;
        graphics::draw(ctx, &mesh, DrawParam::new())?;

        let heart_sprite = self
            .world
            .fetch::<Sprites>()
            .0
            .get("heart")
            .expect("error getting heart sprite")
            .clone();
        graphics::draw(
            ctx,
            &heart_sprite,
            graphics::DrawParam::new()
                .dest([13.5, crate::SCREEN_HEIGHT - 130.0])
                .scale(Vector::new(0.45, 0.45)),
        )
        .expect("error drawing heart sprite");
        let text_mutex = &self.world.fetch::<HPText>().text;
        let text = text_mutex.lock().expect("error locking hp text");
        graphics::draw(
            ctx,
            &*text,
            graphics::DrawParam::new().dest([50.0, crate::SCREEN_HEIGHT - 100.0]),
        )
        .expect("error drawing hp text");

        if self.world.fetch::<Dead>().0 {
            let dead_text = &self.world.fetch::<DeadText>().0;
            let text = dead_text.lock().expect("error locking dead text");
            graphics::draw(
                ctx,
                &text[0],
                graphics::DrawParam::new()
                    .dest([crate::SCREEN_WIDTH / 4.0, crate::SCREEN_HEIGHT / 4.0]),
            )
            .expect("error drawing dead text");

            graphics::draw(
                ctx,
                &text[1],
                graphics::DrawParam::new()
                    .dest([crate::SCREEN_WIDTH / 5.0, crate::SCREEN_HEIGHT / 2.5]),
            )
            .expect("error drawing dead text");
        }

        graphics::present(ctx)?;
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
            .expect("error setting screen coordinates on resize");
        } else {
            // height is greater than usual
            let new_height = SCREEN_HEIGHT * aspect_ratio;
            let excess_height = new_height - SCREEN_HEIGHT;
            ggez::graphics::set_screen_coordinates(
                ctx,
                graphics::Rect::new(0.0, -excess_height / 2.0, SCREEN_WIDTH, new_height),
            )
            .expect("error setting screen coordinates on resize");
        }
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        if keycode == KeyCode::Space && self.world.fetch::<Dead>().0 {
            let player_sprite = self
                .world
                .fetch::<Sprites>()
                .0
                .get("player")
                .expect("error gettng player sprite")
                .clone();
            let player = new_player(player_sprite, 5);
            let player = create_player(&mut self.world, player);
            self.world.insert(PlayerEntity(player));

            self.world.insert(Dead(false));
            self.world.insert(CurrentWave(0));
            self.world.fetch_mut::<HPText>().needs_redraw = true;

            {
                let entities = self.world.entities();
                let enemies = self.world.read_storage::<Enemy>();
                let bullets = self.world.read_storage::<Bullet>();
                entities.join().for_each(|entity| {
                    if enemies.get(entity).is_some() || bullets.get(entity).is_some() {
                        entities
                            .delete(entity)
                            .expect("error deleting enemy or bullet");
                    }
                });
            }
            self.world.maintain();
        }

        if keycode == KeyCode::LControl && !self.world.fetch::<Dead>().0 {
            let mut players = self.world.write_storage::<Player>();
            let player_entity = self.world.fetch::<PlayerEntity>().0;
            let player = players
                .get_mut(player_entity)
                .expect("error getting player entity");

            if player.deflector_cooldown == 0 {
                player.deflector_timer = player.deflector_frames;
                player.deflector_cooldown = player.deflector_reload_frames;
            }
        }
    }
}

fn draw_colorect(builder: &mut MeshBuilder, pos: Point, colorect: &ColorRect) {
    let rect = Rect::new(pos.x, pos.y, colorect.w, colorect.h);
    builder.rectangle(DrawMode::fill(), rect, colorect.color);
}
