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

        let positions = self.world.read_storage::<Position>();
        let colorects = self.world.read_storage::<ColorRect>();
        let hitboxes = self.world.read_storage::<Hitbox>();
        let sprites = self.world.read_storage::<Sprite>();
        let bullets = self.world.read_storage::<Bullet>();
        let stars = self.world.read_storage::<Star>();
        let hp_storage = self.world.read_storage::<HP>();
        let entities = self.world.entities();
        let animated_sprite_storage = self.world.read_storage::<AnimatedSprite>();
        let mut bullet_spritebatch = self.world.fetch_mut::<BulletSpriteBatch>();

        let mut builder = MeshBuilder::new();
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

        (&positions, &sprites, &entities, !&bullets)
            .join()
            .for_each(|(pos, sprite, entity, _)| {
                let draw_color = if let Some(hp) = hp_storage.get(entity) {
                    let opacity = (60 - hp.iframes as u32) as f32 / 60.0;
                    Color::new(1.0, 1.0, 1.0, opacity.powi(5))
                } else {
                    graphics::WHITE
                };
                graphics::draw(
                    ctx,
                    &sprite.0,
                    graphics::DrawParam::new()
                        .scale([3.0, 3.0])
                        .dest(pos.0)
                        .color(draw_color),
                )
                .unwrap()
            });

        (&positions, &bullets).join().for_each(|(pos, _)| {
            bullet_spritebatch
                .0
                .add(DrawParam::new().scale([3.0, 3.0]).dest(pos.0));
        });

        (&positions, &animated_sprite_storage)
            .join()
            .for_each(|(pos, animated_sprite)| {
                graphics::draw(
                    ctx,
                    &animated_sprite.frames[animated_sprite.current_frame as usize],
                    graphics::DrawParam::new().scale([3.5, 3.5]).dest(pos.0),
                )
                .unwrap();
            });

        (&positions, &hitboxes).join().for_each(|(pos, hitbox)| {
            let rect = Rect::new(pos.0.x, pos.0.y, hitbox.0, hitbox.1);
            builder.rectangle(DrawMode::stroke(2.5), rect, Color::new(1.0, 0.0, 0.0, 1.0));
        });

        (&positions, &bullets).join().for_each(|(pos, _)| {
            let rect = Rect::new(pos.0.x, pos.0.y, 5.0, 5.0);
            let mesh = graphics::Mesh::new_rectangle(
                ctx,
                DrawMode::stroke(2.5),
                rect,
                Color::new(1.0, 0.0, 0.0, 1.0),
            )
            .unwrap();
            graphics::draw(ctx, &mesh, graphics::DrawParam::new()).unwrap();
        });

        graphics::draw(ctx, &bullet_spritebatch.0, graphics::DrawParam::new())?;
        bullet_spritebatch.0.clear();

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
