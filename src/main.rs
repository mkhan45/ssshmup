use components::PlayerEntity;
use ggez::{event, GameResult};
use specs::prelude::*;

mod components;
mod game_state;
mod systems;

const SCREEN_WIDTH: f32 = 576.0 * 0.75;
const SCREEN_HEIGHT: f32 = 1024.0 * 0.75;

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
    world.register::<components::Bullet>();

    world.insert(components::StarInfo {
        num_stars: 150,
        size: 2.5,
        size_variance: 1.5,
        vel: 5.0,
        vel_variance: 2.0,
    });

    let player = components::new_player(3);
    let player = components::create_player(&mut world, &player);
    world.insert(PlayerEntity(player));

    let enemy = components::new_enemy(
        components::Enemy::BasicEnemy,
        [SCREEN_WIDTH / 2.0, 100.0].into(),
    );
    components::create_enemy(&mut world, &enemy);

    let mut dispatcher = DispatcherBuilder::new()
        .with(systems::IntegrateSys, "integrate_system", &[])
        .with(systems::StarMoveSys, "star_system", &[])
        .with(systems::ReloadTimerSys, "reload_timer_sys", &[])
        .build();

    dispatcher.setup(&mut world);

    let mut game_state = game_state::GameState::new(world, dispatcher);

    event::run(ctx, event_loop, &mut game_state)
}
