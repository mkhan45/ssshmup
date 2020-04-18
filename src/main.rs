#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use ggez::{event, graphics::spritebatch::SpriteBatch, GameResult};
use specs::prelude::*;

use std::collections::HashMap;

use std::sync::{Arc, Mutex};

mod game_state;

mod ecs;

use ecs::{components, resources, systems};

const SCREEN_WIDTH: f32 = 1024.0 * 0.75;
const SCREEN_HEIGHT: f32 = 1024.0 * 0.75;

fn main() -> GameResult {
    let (ctx, event_loop) = &mut ggez::ContextBuilder::new("Game", "Fish")
        .window_setup(ggez::conf::WindowSetup::default().title("Game"))
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(SCREEN_WIDTH, SCREEN_HEIGHT)
                .resizable(true),
        )
        .build()
        .expect("error building context");

    ggez::graphics::set_default_filter(ctx, ggez::graphics::FilterMode::Nearest);
    ggez::graphics::set_screen_coordinates(
        ctx,
        ggez::graphics::Rect::new(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT),
    )
    .unwrap();

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

    world.insert(resources::StarInfo {
        num_stars: 150,
        size: 2.5,
        size_variance: 1.5,
        vel: 5.0,
        vel_variance: 2.0,
    });

    let player_sprite = ggez::graphics::Image::new(ctx, "/player.png").unwrap();
    let player = components::new_player(player_sprite, 5);
    let player = components::create_player(&mut world, player);
    world.insert(components::PlayerEntity(player));

    let mut sprites = HashMap::new();
    let mut animated_sprites = HashMap::new();
    let mut spritesheets = HashMap::new();
    {
        use ggez::graphics::Image;
        let enemy1_image = Image::new(ctx, "/ufo1.png");
        let bullet1_image = Image::new(ctx, "/bullet1.png");

        sprites.insert("enemy1".to_string(), enemy1_image.unwrap());
        sprites.insert("bullet1".to_string(), bullet1_image.unwrap());

        let bullet_spritesheet = Image::new(ctx, "/bullet_sheet.png");
        let bullet_spritebatch = SpriteBatch::new(bullet_spritesheet.unwrap());
        spritesheets.insert(
            "bullets".to_string(),
            Arc::new(Mutex::new(resources::SpriteSheet {
                width: 4,
                batch: bullet_spritebatch,
            })),
        );

        let enemy_spritesheet = Image::new(ctx, "/enemy_sheet.png");
        let enemy_spritebatch = SpriteBatch::new(enemy_spritesheet.unwrap());
        spritesheets.insert(
            "enemies".to_string(),
            Arc::new(Mutex::new(resources::SpriteSheet {
                width: 8,
                batch: enemy_spritebatch,
            })),
        );

        let explosion_img = Image::new(ctx, "/boom.png").unwrap();
        animated_sprites.insert(
            "explosion".to_string(),
            components::AnimatedSprite::new(explosion_img, 12, 16, true),
        );
    }
    world.insert(resources::BulletSpriteBatch(SpriteBatch::new(
        sprites.get("bullet1").unwrap().clone(),
    )));
    world.insert(resources::Sprites(sprites));
    world.insert(resources::AnimatedSprites(animated_sprites));
    world.insert(resources::SpriteSheets(spritesheets));
    world.insert(resources::CurrentWave(0));
    world.insert(resources::QueuedEnemies(Vec::new()));
    world.insert(resources::FramesToNextWave(0));
    world.insert(resources::Dead(false));
    {
        use ggez::graphics::{Font, Scale, Text};
        let font = Font::new(ctx, "/fonts/Xolonium-Regular.ttf").unwrap();
        let mut text = Text::new(format!("HP: {}\nWave: {}", 5, 0));
        text.set_font(font, Scale::uniform(48.0));
        world.insert(resources::HPText {
            needs_redraw: false,
            text: Mutex::new(text),
        });
        world.insert(resources::GameFont(font));

        let mut dead_text1 = Text::new("You Died!");
        dead_text1.set_font(font, Scale::uniform(96.0));

        let mut dead_text2 = Text::new("Press Space to respawn");
        dead_text2.set_font(font, Scale::uniform(48.0));
        world.insert(resources::DeadText(Mutex::new([dead_text1, dead_text2])));
    }

    {
        use ggez::audio::SoundData;

        let mut sounds = HashMap::new();
        sounds.insert(
            "shoot".to_string(),
            SoundData::new(ctx, "/shoot2.ogg").unwrap(),
        );
        sounds.insert(
            "boom".to_string(),
            SoundData::new(ctx, "/boom.ogg").unwrap(),
        );
        sounds.insert(
            "dead".to_string(),
            SoundData::new(ctx, "/dead.ogg").unwrap(),
        );
        world.insert(resources::Sounds(sounds));
        world.insert(resources::QueuedSounds(Vec::new()));
    }

    let mut dispatcher = DispatcherBuilder::new()
        .with(systems::EnemyMoveSys, "enemy_move_sys", &[])
        .with(systems::BulletTrackingSys, "tracking_bullet_sys", &[])
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
