use ggez::graphics::{spritebatch::SpriteBatch, Color, Image};
use ggez::nalgebra::{Point2, Vector2};

use specs::prelude::*;
use specs::Component;

use std::collections::HashMap;

use std::sync::{Arc, Mutex};

pub type Point = Point2<f32>;
pub type Vector = Vector2<f32>;

#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[storage(VecStorage)]
pub struct Position(pub Point);

impl Into<Point> for Position {
    fn into(self) -> Point {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[storage(VecStorage)]
pub struct Velocity(pub Vector);
impl Default for Velocity {
    fn default() -> Self {
        Velocity(Vector::new(0.0, 0.0))
    }
}

#[derive(Clone, Debug, Component)]
#[storage(VecStorage)]
pub enum Sprite {
    Img(Image),
    SpriteSheetInstance(Arc<Mutex<SpriteSheet>>, u8),
}

#[derive(Clone, Debug, PartialEq, Component)]
#[storage(DenseVecStorage)]
pub struct AnimatedSprite {
    pub spritesheet: Image,
    pub num_frames: u8,
    pub spritesheet_width: u8,
    pub current_frame: u8,
    pub temporary: bool,
}

impl AnimatedSprite {
    pub fn new(spritesheet: Image, num_frames: u8, spritesheet_width: u8, temporary: bool) -> Self {
        AnimatedSprite {
            spritesheet,
            num_frames,
            spritesheet_width,
            current_frame: 0,
            temporary,
        }
    }

    #[allow(dead_code)]
    pub fn set_temporary(mut self, temporary: bool) -> Self {
        self.temporary = temporary;
        self
    }
}

#[derive(Clone, Copy, Default, Component)]
#[storage(NullStorage)]
pub struct Explosion;

#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[storage(VecStorage)]
pub struct ColorRect {
    pub color: Color,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[storage(VecStorage)]
pub struct HP {
    pub remaining: u32,
    pub iframes: u8,
}

impl HP {
    pub fn new(hp: u32) -> Self {
        HP {
            remaining: hp,
            iframes: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BulletType {
    BasicBullet,
    AimedBullet,
    PredictBullet,
    TrackingBullet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamagesWho {
    Player,
    Enemy,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Component)]
#[storage(VecStorage)]
pub struct Bullet {
    pub damage: u32,
    pub damages_who: DamagesWho,
    pub ty: BulletType,
}

impl Bullet {
    pub fn damages_player(self) -> bool {
        self.damages_who == DamagesWho::Both || self.damages_who == DamagesWho::Player
    }

    pub fn damages_enemy(self) -> bool {
        self.damages_who == DamagesWho::Both || self.damages_who == DamagesWho::Enemy
    }
}

pub type BulletTuple = (Position, Hitbox, Velocity, Bullet, u8);
pub fn new_bullet(ty: BulletType, pos: Point, vel: Vector, damages_who: DamagesWho) -> BulletTuple {
    let damage = match ty {
        BulletType::BasicBullet => 1,
        BulletType::AimedBullet => 1,
        BulletType::PredictBullet => 1,
        BulletType::TrackingBullet => 1,
    };

    let sprite_index = match ty {
        BulletType::BasicBullet => 0,
        BulletType::AimedBullet => 1,
        BulletType::PredictBullet => 2,
        BulletType::TrackingBullet => 3,
    };

    let (offset, width, height) = match ty {
        BulletType::BasicBullet
        | BulletType::AimedBullet
        | BulletType::PredictBullet
        | BulletType::TrackingBullet => (Point::new(5.0, 5.0), 15.0, 15.0),
    };

    let bullet = Bullet {
        damage,
        ty,
        damages_who,
    };

    let pos: Point = [pos.x, pos.y - 16.0].into();
    (
        Position(pos),
        Hitbox(offset, width, height),
        Velocity(vel),
        bullet,
        sprite_index,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnemyType {
    BasicEnemy,
    AimEnemy,
    PredictEnemy,
    TrackingEnemy,
    AimEnemy2,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MovementType {
    HLine(std::ops::Range<f32>, f32),
    VLine(std::ops::Range<f32>, f32),
    Circle(Point, f32, f32, f32),
}

impl MovementType {
    pub fn horizontal(center: f32, width: f32, speed: f32) -> Self {
        MovementType::HLine((center - width / 2.0)..(center + width / 2.0), speed)
    }

    pub fn vertical(center: f32, height: f32, speed: f32) -> Self {
        MovementType::VLine((center - height / 2.0)..(center + height / 2.0), speed)
    }

    pub fn circle(center: Point, radius: f32, speed: f32) -> Self {
        MovementType::Circle(center, radius, speed, 0.0)
    }
}

#[derive(Clone, Debug, PartialEq, Component)]
#[storage(VecStorage)]
pub struct Enemy {
    pub ty: EnemyType,
    pub movement: MovementType,
    pub bullet_type: BulletType,
    pub reload_timer: u32,
    pub reload_speed: u32,
}

pub type EnemyTuple = (Position, Velocity, Enemy, HP, Hitbox, u8);
pub fn new_enemy(ty: EnemyType, pos: Point, movement: MovementType) -> EnemyTuple {
    let pos = Position(pos);
    let (hp, size, bullet_type, reload_speed) = match ty {
        EnemyType::BasicEnemy => (3, (55.0, 43.0), BulletType::BasicBullet, 180),
        EnemyType::AimEnemy => (3, (55.0, 43.0), BulletType::AimedBullet, 180),
        EnemyType::PredictEnemy => (3, (55.0, 43.0), BulletType::PredictBullet, 90),
        EnemyType::TrackingEnemy => (3, (55.0, 43.0), BulletType::TrackingBullet, 180),
        EnemyType::AimEnemy2 => (5, (55.0, 43.0), BulletType::AimedBullet, 90),
    };

    let vel = match movement {
        MovementType::HLine(_, speed) => [speed, 0.0].into(),
        MovementType::VLine(_, speed) => [0.0, speed].into(),
        MovementType::Circle(_, _, speed, _) => [0.0, speed].into(),
    };

    let sprite_index = match ty {
        EnemyType::BasicEnemy => 0,
        EnemyType::AimEnemy | EnemyType::AimEnemy2 => 1,
        EnemyType::PredictEnemy => 2,
        EnemyType::TrackingEnemy => 3,
    };

    (
        pos,
        Velocity(vel),
        Enemy {
            ty,
            movement,
            bullet_type,
            reload_timer: reload_speed,
            reload_speed,
        },
        HP::new(hp),
        Hitbox([21.0, 32.0].into(), size.0, size.1),
        sprite_index,
    )
}

pub fn create_enemy(world: &mut World, enemy: EnemyTuple) -> Entity {
    let spritesheet = {
        let spritesheets = world.fetch::<SpriteSheets>();
        spritesheets.0.get("enemies").unwrap().clone()
    };

    world
        .create_entity()
        .with(enemy.0)
        .with(enemy.1)
        .with(enemy.2)
        .with(enemy.3)
        .with(enemy.4)
        .with(Sprite::SpriteSheetInstance(spritesheet, enemy.5))
        .build()
}

#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[storage(HashMapStorage)]
pub struct Player {
    pub bullet_type: BulletType,
    pub reload_speed: u32,
    pub reload_timer: u32,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            bullet_type: BulletType::BasicBullet,
            reload_speed: 6,
            reload_timer: 6,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerEntity(pub Entity);

impl Default for PlayerEntity {
    fn default() -> Self {
        panic!("something has gone terribly wrong")
    }
}

pub type PlayerTuple = (Position, Velocity, HP, Sprite, Player, Hitbox);
pub fn new_player(sprite: Image, hp: u32) -> PlayerTuple {
    let pos = Position(
        [
            crate::SCREEN_WIDTH / 2.0 - 25.0,
            crate::SCREEN_HEIGHT * 0.75,
        ]
        .into(),
    );
    let vel = Velocity::default();
    let hp = HP::new(hp);

    (
        pos,
        vel,
        hp,
        Sprite::Img(sprite),
        Player::default(),
        Hitbox([0.0, 0.0].into(), 45.0, 45.0),
    )
}

pub fn create_player(world: &mut World, player: PlayerTuple) -> Entity {
    world
        .create_entity()
        .with(player.0)
        .with(player.1)
        .with(player.2)
        .with(player.3)
        .with(player.4)
        .with(player.5)
        .build()
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
            color: ggez::graphics::WHITE,
            w: size,
            h: size,
        };

        (Position(pos), Velocity(vel), color_rect)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Component, Default)]
#[storage(NullStorage)]
pub struct Star;

#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[storage(VecStorage)]
pub struct Hitbox(pub Point, pub f32, pub f32);

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
pub struct AnimatedSprites(pub HashMap<String, AnimatedSprite>);

#[derive(Clone, Default)]
pub struct CurrentWave(pub u8);

#[derive(Copy, Clone)]
pub struct FramesToNextWave(pub u16);
impl Default for FramesToNextWave {
    fn default() -> Self {
        FramesToNextWave(120)
    }
}

#[derive(Clone, Default)]
pub struct QueuedEnemies(pub Vec<(Point, EnemyType)>);

#[derive(Default)]
pub struct HPText {
    pub needs_redraw: bool,
    pub text: Mutex<ggez::graphics::Text>,
}

#[derive(Clone, Default)]
pub struct GameFont(pub ggez::graphics::Font);
