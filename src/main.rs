use components::PlayerEntity;
use ggez::{event, graphics::spritebatch::SpriteBatch, GameResult};
use specs::prelude::*;

use std::collections::HashMap;

mod components;
mod game_state;
mod systems;

const SCREEN_WIDTH: f32 = 1024.0 * 0.75;
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
    world.register::<components::Sprite>();
    world.register::<components::AnimatedSprite>();
    world.register::<components::Hitbox>();

    world.insert(components::StarInfo {
        num_stars: 200,
        size: 2.5,
        size_variance: 1.5,
        vel: 5.0,
        vel_variance: 2.0,
    });

    let mut player_sprite = ggez::graphics::Image::new(ctx, "/player.png").unwrap();
    player_sprite.set_filter(ggez::graphics::FilterMode::Nearest);
    let player = components::new_player(player_sprite, 3);
    let player = components::create_player(&mut world, player);
    world.insert(PlayerEntity(player));

    let mut sprites = HashMap::new();
    let mut animated_sprites = HashMap::new();
    {
        use ggez::graphics::{FilterMode, Image};
        let enemy1_image = Image::new(ctx, "/ufo1.png");
        let bullet1_image = Image::new(ctx, "/bullet1.png");

        sprites.insert("enemy1".to_string(), enemy1_image.unwrap());
        sprites.insert("bullet1".to_string(), bullet1_image.unwrap());
        sprites
            .iter_mut()
            .for_each(|(_, image)| image.set_filter(FilterMode::Nearest));

        let explosion: Vec<Image> = (1..12)
            .map(|i| {
                let mut img = Image::new(ctx, format!("/explosion/explosion{:02}.png", i)).unwrap();
                img.set_filter(FilterMode::Nearest);
                img
            })
            .collect();

        animated_sprites.insert("explosion".to_string(), explosion);
    }
    world.insert(components::BulletSpriteBatch(SpriteBatch::new(
        sprites.get("bullet1").unwrap().clone(),
    )));
    world.insert(components::Sprites(sprites));
    world.insert(components::AnimatedSprites(animated_sprites));

    (0..10).for_each(|i| {
        let min_x = i as f32 * 60.0;
        let (et1, et2) = if i % 2 == 0 {
            (
                components::EnemyType::AimEnemy,
                components::EnemyType::BasicEnemy,
            )
        } else {
            (
                components::EnemyType::BasicEnemy,
                components::EnemyType::AimEnemy,
            )
        };
        let enemy = components::new_enemy(
            et1,
            [min_x, 100.0].into(),
            components::MovementType::HLine(min_x..min_x + 175.0, 1.25),
        );
        components::create_enemy(&mut world, enemy);

        let enemy = components::new_enemy(
            et2,
            [min_x, 0.0].into(),
            components::MovementType::HLine(min_x..min_x + 175.0, 1.25),
        );
        components::create_enemy(&mut world, enemy);
    });

    let mut dispatcher = DispatcherBuilder::new()
        .with(systems::EnemyMoveSys, "enemy_move_sys", &[])
        .with(systems::IntegrateSys, "integrate_system", &[])
        .with(systems::StarMoveSys, "star_system", &[])
        .with(systems::ReloadTimerSys, "reload_timer_sys", &[])
        .with(systems::EnemyShootSys, "enemy_shoot_sys", &[])
        .with(systems::AnimationSys, "animation_sys", &[])
        .with(
            systems::BulletCollSys,
            "bullet_coll_sys",
            &["integrate_system"],
        )
        .with(
            systems::PlayerCollSys,
            "player_coll_sys",
            &["integrate_system"],
        )
        .with(
            systems::HPKillSys,
            "hp_kill_sys",
            &["bullet_coll_sys", "player_coll_sys"],
        )
        .with(systems::IFrameSys, "iframe_sys", &["hp_kill_sys"])
        .build();

    dispatcher.setup(&mut world);

    let mut game_state = game_state::GameState::new(world, dispatcher);

    event::run(ctx, event_loop, &mut game_state)
}
