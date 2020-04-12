use ggez::graphics::{spritebatch::SpriteBatch, Color, Image};
use ggez::nalgebra::{Point2, Vector2};

use specs::prelude::*;
use specs::Component;

use std::collections::HashMap;

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

#[derive(Clone, Debug, PartialEq, Component)]
#[storage(VecStorage)]
pub struct Sprite(pub Image);

#[derive(Clone, Debug, PartialEq, Component)]
#[storage(DenseVecStorage)]
pub struct AnimatedSprite {
    pub frames: Vec<Image>,
    pub current_frame: u8,
    pub temporary: bool,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Component)]
#[storage(VecStorage)]
pub struct Bullet {
    pub damage: u32,
    pub friendly: bool,
    pub ty: BulletType,
}

pub type BulletTuple = (Position, Velocity, Bullet);
pub fn new_bullet(ty: BulletType, pos: Point, start_vel: Vector, friendly: bool) -> BulletTuple {
    let (damage, speed) = match ty {
        BulletType::BasicBullet => (1, 8.0),
    };

    let bullet = Bullet {
        damage,
        ty,
        friendly,
    };

    let pos: Point = [pos.x, pos.y - 16.0].into();
    (
        Position(pos),
        Velocity([0.0, -speed + start_vel.y.min(0.0)].into()),
        bullet,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[storage(VecStorage)]
pub enum Enemy {
    BasicEnemy,
}

pub type EnemyTuple = (Position, Velocity, Enemy, HP, Hitbox);
pub fn new_enemy(enemy_type: Enemy, pos: Point) -> EnemyTuple {
    let pos = Position(pos);
    let vel = Velocity::default();
    let (hp, size) = match enemy_type {
        Enemy::BasicEnemy => (1, (55.0, 43.0)),
    };

    (pos, vel, enemy_type, HP::new(hp), Hitbox(size.0, size.1))
}

pub fn create_enemy(world: &mut World, enemy: &EnemyTuple) -> Entity {
    let sprite = {
        let sprites = &world.fetch::<Sprites>().0;
        sprites
            .get(match enemy.2 {
                Enemy::BasicEnemy => "enemy1",
            })
            .unwrap()
            .clone()
    };

    world
        .create_entity()
        .with(enemy.0)
        .with(enemy.1)
        .with(enemy.2)
        .with(enemy.3)
        .with(enemy.4)
        .with(Sprite(sprite))
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
        Sprite(sprite),
        Player::default(),
        Hitbox(45.0, 45.0),
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

#[derive(Clone, Copy, Debug, PartialEq, Component, Default)]
#[storage(VecStorage)]
pub struct Hitbox(pub f32, pub f32);

#[derive(Clone, Default)]
pub struct Sprites(pub HashMap<String, Image>);

#[derive(Clone)]
pub struct BulletSpriteBatch(pub SpriteBatch);

#[derive(Clone, Default)]
pub struct AnimatedSprites(pub HashMap<String, Vec<Image>>);
