use ggez::{
    event::EventHandler,
    graphics::{
        self, Color, DrawMode, DrawParam, Font, MeshBuilder, Rect, Scale, Text, TextFragment,
    },
    input::{self, keyboard::KeyCode},
    timer, Context, GameResult,
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
        init_star_sys.run_now(&mut world);
        GameState { world, dispatcher }
    }
}

impl EventHandler for GameState<'_, '_> {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if ggez::timer::ticks(&ctx) % 120 == 0 {
            dbg!(ggez::timer::fps(&ctx));
        }

        {
            let mut velocities = self.world.write_storage::<Velocity>();
            let player_vel = &mut velocities
                .get_mut(self.world.fetch::<PlayerEntity>().0)
                .unwrap();
            player_vel.0 /= 1.35;
            if input::keyboard::is_key_pressed(ctx, KeyCode::W) {
                player_vel.0.y -= 3.0;
            }
            if input::keyboard::is_key_pressed(ctx, KeyCode::S) {
                player_vel.0.y += 3.0;
            }
            if input::keyboard::is_key_pressed(ctx, KeyCode::A) {
                player_vel.0.x -= 3.0;
            }
            if input::keyboard::is_key_pressed(ctx, KeyCode::D) {
                player_vel.0.x += 3.0;
            }
        }

        self.dispatcher.dispatch(&mut self.world);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));

        let positions = self.world.read_storage::<Position>();
        let colorects = self.world.read_storage::<ColorRect>();

        let mut builder = MeshBuilder::new();
        (&positions, &colorects).join().for_each(|(pos, colorect)| {
            draw_colorect(&mut builder, (*pos).into(), &colorect);
        });

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
