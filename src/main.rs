#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use ggez::{event, graphics::spritebatch::SpriteBatch, GameResult};
use specs::prelude::*;

use log;
use simple_logger;

use std::collections::HashMap;

use std::sync::{Arc, Mutex};

mod game_state;

mod ecs;

use ecs::{components, resources, systems};

const SCREEN_WIDTH: f32 = 1024.0 * 0.75;
const SCREEN_HEIGHT: f32 = 1024.0 * 0.75;

const VOLUME_MULTIPLIER: f32 = 0.2;

fn main() -> GameResult {
    simple_logger::init_with_level(log::Level::Warn).expect("error initializing logger");
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
    .expect("error setting default screen coordinates");

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
        num_stars: 60,
        size: 2.25,
        size_variance: 1.5,
        vel: 5.0,
        vel_variance: 2.0,
    });

    let mut sprites = HashMap::new();
    let player_sprite =
        ggez::graphics::Image::new(ctx, "/player.png").expect("error loading player sprite");
    let player_deflector_sprite = ggez::graphics::Image::new(ctx, "/player_deflector.png")
        .expect("error loading player deflector sprite");
    let player_cooldown_sprite = ggez::graphics::Image::new(ctx, "/player_cooldown.png")
        .expect("error loading player cooldown sprite");
    sprites.insert("player".to_string(), player_sprite.clone());
    sprites.insert("player_deflector".to_string(), player_deflector_sprite);
    sprites.insert("player_cooldown".to_string(), player_cooldown_sprite);
    let player = components::new_player(player_sprite, 6);
    let player = components::create_player(&mut world, player);
    world.insert(components::PlayerEntity(player));

    let mut animated_sprites = HashMap::new();
    let mut spritesheets = HashMap::new();
    {
        use ggez::graphics::Image;
        let bullet_spritesheet =
            Image::new(ctx, "/bullet_sheet.png").expect("error loading bullet spritesheet");
        let bullet_spritebatch = SpriteBatch::new(bullet_spritesheet);
        spritesheets.insert(
            "bullets".to_string(),
            Arc::new(Mutex::new(resources::SpriteSheet {
                width: 8,
                batch: bullet_spritebatch,
            })),
        );

        let enemy_spritesheet =
            Image::new(ctx, "/enemy_sheet.png").expect("error loading enemy spritesheet");
        let enemy_spritebatch = SpriteBatch::new(enemy_spritesheet);
        spritesheets.insert(
            "enemies".to_string(),
            Arc::new(Mutex::new(resources::SpriteSheet {
                width: 8,
                batch: enemy_spritebatch,
            })),
        );

        let explosion_img = Image::new(ctx, "/boom.png").expect("error loading explosion sprite");
        animated_sprites.insert(
            "explosion".to_string(),
            components::AnimatedSprite::new(explosion_img, 12, 16, true),
        );
    }
    world.insert(resources::Sprites(sprites));
    world.insert(resources::AnimatedSprites(animated_sprites));
    world.insert(resources::SpriteSheets(spritesheets));
    world.insert(resources::CurrentWave(0));
    world.insert(resources::QueuedEnemies(Vec::new()));
    world.insert(resources::FramesToNextWave(0));
    world.insert(resources::Dead(false));
    {
        use ggez::graphics::{Font, Scale, Text};
        let font = Font::new(ctx, "/fonts/Xolonium-Regular.ttf").expect("error loading font");
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
        {
            use ggez::audio::{SoundSource, Source};
            let bg_music_source = Source::new(ctx, "/bgmusic.ogg");
            if let Ok(mut bg_music_source) = bg_music_source {
                bg_music_source.set_repeat(true);
                bg_music_source.set_volume(0.2 * VOLUME_MULTIPLIER);
                if bg_music_source.play_detached().is_err() {
                    log::warn!("error playing background music");
                }
            } else {
                log::warn!("error loading background music");
            }
        }
        sounds.insert(
            "shoot".to_string(),
            SoundData::new(ctx, "/shoot2.ogg").expect("error loading shoot2.ogg"),
        );
        sounds.insert(
            "boom".to_string(),
            SoundData::new(ctx, "/boom.ogg").expect("error loading boom.ogg"),
        );
        sounds.insert(
            "dead".to_string(),
            SoundData::new(ctx, "/dead.ogg").expect("error loading dead.ogg"),
        );
        sounds.insert(
            "deflect".to_string(),
            SoundData::new(ctx, "/deflect.ogg").expect("error loading deflect.ogg"),
        );
        world.insert(resources::Sounds(sounds));
        world.insert(resources::QueuedSounds(Vec::new()));
    }

    let mut dispatcher = DispatcherBuilder::new()
        .with(systems::EnemyMoveSys, "enemy_move_sys", &[])
        .with(systems::BulletTrackingSys, "tracking_bullet_sys", &[])
        .with(systems::BounceBulletSys, "bouncing_bullet_sys", &[])
        .with(systems::IntegrateSys, "integrate_system", &[])
        .with(systems::StarMoveSys, "star_system", &[])
        .with(systems::ReloadTimerSys, "reload_timer_sys", &[])
        .with(systems::DeflectorSys, "deflector_timer_sys", &[])
        .with(systems::EnemyShootSys, "enemy_shoot_sys", &[])
        .with(systems::AnimationSys, "animation_sys", &[])
        .with(
            systems::BulletCollSys,
            "bullet_coll_sys",
            &["integrate_system", "bouncing_bullet_sys"],
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
