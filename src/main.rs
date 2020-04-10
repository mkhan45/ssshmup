use components::PlayerEntity;
use ggez::{event, graphics::Font, GameResult};
use specs::prelude::*;

use rand;

mod components;
mod game_state;
mod systems;

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
    world.register::<components::Enemy>();

    world.insert(components::StarInfo {
        num_stars: 75,
        size: 3.0,
        size_variance: 1.5,
        vel: 5.0,
        vel_variance: 2.0,
    });

    let player = components::new_player(3);
    let player = components::create_player(&mut world, &player);
    world.insert(PlayerEntity(player));

    let enemy = components::new_enemy(components::Enemy::BasicEnemy, [288.0, 100.0].into());
    let enemy = components::create_enemy(&mut world, &enemy);

    let mut dispatcher = DispatcherBuilder::new()
        .with(systems::IntegrateSys, "integrate_system", &[])
        .with(systems::StarMoveSys, "star_system", &[])
        .build();

    dispatcher.setup(&mut world);

    let mut game_state = game_state::GameState::new(world, dispatcher);

    event::run(ctx, event_loop, &mut game_state)
}
