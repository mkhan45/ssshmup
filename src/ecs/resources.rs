use ggez::audio::SoundData;
use ggez::graphics::{spritebatch::SpriteBatch, Image};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use std::time::Duration;

use crate::ecs::components::*;

#[derive(Default)]
pub struct DeadText(pub Mutex<[ggez::graphics::Text; 2]>);

#[derive(Clone, Default)]
pub struct GameFont(pub ggez::graphics::Font);

#[derive(Clone, Default)]
pub struct Dead(pub bool);

#[derive(Default)]
pub struct HPText {
    pub needs_redraw: bool,
    pub text: Mutex<ggez::graphics::Text>,
}

#[derive(Clone, Default)]
pub struct QueuedEnemies(pub Vec<(Point, EnemyType)>);

#[derive(Clone, Default)]
pub struct Sprites(pub HashMap<String, Image>);

#[derive(Clone, Debug)]
pub struct SpriteSheet {
    pub width: u8,
    pub batch: SpriteBatch,
}

#[derive(Clone, Default)]
pub struct SpriteSheets(pub HashMap<String, Arc<Mutex<SpriteSheet>>>);

#[derive(Clone)]
pub struct BulletSpriteBatch(pub SpriteBatch);

#[derive(Clone, Default)]
pub struct AnimatedSprites(pub HashMap<String, crate::ecs::components::AnimatedSprite>);

#[derive(Clone, Default)]
pub struct CurrentWave(pub u8);

#[derive(Copy, Clone)]
pub struct FramesToNextWave(pub u16);
impl Default for FramesToNextWave {
    fn default() -> Self {
        FramesToNextWave(120)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct StarInfo {
    pub num_stars: usize,
    pub size: f32,
    pub size_variance: f32,
    pub vel: f32,
    pub vel_variance: f32,
}

impl StarInfo {
    pub fn new_star(&self) -> (Position, Velocity, ColorRect) {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0.0, crate::SCREEN_WIDTH);
        let y = rng.gen_range(-crate::SCREEN_WIDTH, 0.0);
        let y_vel = rng.gen_range(self.vel - self.vel_variance, self.vel + self.vel_variance);
        let size = rng.gen_range(
            self.size - self.size_variance,
            self.size + self.size_variance,
        );

        let pos = [x, y].into();
        let vel = [0.0, y_vel].into();
        let color_rect = ColorRect {
            color: ggez::graphics::Color::new(1.0, 1.0, 1.0, 0.35),
            w: size,
            h: size,
        };

        (Position(pos), Velocity(vel), color_rect)
    }
}

#[derive(Clone, Default)]
pub struct Sounds(pub HashMap<String, SoundData>);

#[derive(Clone, Default)]
pub struct QueuedSounds(pub Vec<SoundData>);

#[derive(Clone, Default)]
pub struct LastUpdate(pub Duration);
