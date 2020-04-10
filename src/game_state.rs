use specs::prelude::*;
use ggez::{
    event::EventHandler,
    graphics::{self, Color, DrawMode, DrawParam, Font, MeshBuilder, Scale, Text, TextFragment, Rect},
    input::{self, keyboard::KeyCode},
    timer, Context, GameResult,
};

use crate::components::*;

pub struct GameState {
    world: World,
}

impl GameState {
    pub fn new(world: World) -> Self {
        GameState {
            world,
        } 
    }
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        {
            let mut positions = self.world.write_storage::<Position>();
            let mut player_pos = &mut positions.get_mut(self.world.fetch::<PlayerEntity>().0).unwrap();
            if input::keyboard::is_key_pressed(ctx, KeyCode::W) {
                player_pos.0.y -= 10.0;
            }
            if input::keyboard::is_key_pressed(ctx, KeyCode::S) {
                player_pos.0.y += 10.0;
            }
            if input::keyboard::is_key_pressed(ctx, KeyCode::A) {
                player_pos.0.x -= 10.0;
            }
            if input::keyboard::is_key_pressed(ctx, KeyCode::D) {
                player_pos.0.x += 10.0;
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));

        let positions = self.world.read_storage::<Position>();
        let colorects = self.world.read_storage::<ColorRect>();

        let mut builder = MeshBuilder::new();
        (&positions, &colorects).join().for_each(|(pos, colorect)|{
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
