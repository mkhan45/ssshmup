use ggez::{event, graphics::Font, GameResult};
use specs::prelude::*;
use components::PlayerEntity;

mod game_state;
mod components;

const SCREEN_WIDTH: f32 = 576.0;
const SCREEN_HEIGHT: f32 = 1024.0;

fn main() -> GameResult {
    let (ctx, event_loop) = &mut ggez::ContextBuilder::new("Tetrs", "Fish")
        .window_setup(ggez::conf::WindowSetup::default().title("Tetrs"))
        .window_mode(
            ggez::conf::WindowMode::default()
            .dimensions(SCREEN_WIDTH, SCREEN_HEIGHT)
            .resizable(false),
        )
        .build()
        .expect("error building context");

    let mut world = World::new();

    world.register::<components::Position>();
    world.register::<components::Player>();
    world.register::<components::Velocity>();
    world.register::<components::ColorRect>();
    world.register::<components::HP>();

    let player = components::new_player(3);
    let player = components::create_player(&mut world, &player);
    world.insert(PlayerEntity(player));

    let mut game_state = game_state::GameState::new(world);

    event::run(ctx, event_loop, &mut game_state)
}
