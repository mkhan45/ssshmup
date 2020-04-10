use ggez::graphics::{Color, Rect};
use ggez::nalgebra::{Point2, Vector2};

use specs::prelude::*;
use specs::Component;

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

#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[storage(VecStorage)]
pub struct Sprite;

#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[storage(VecStorage)]
pub struct ColorRect {
    pub color: Color,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[storage(VecStorage)]
pub struct HP(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[storage(VecStorage)]
pub struct Bullet(pub u32);
pub type BulletTuple = (Position, Velocity, ColorRect, Bullet);

#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[storage(VecStorage)]
pub enum Enemy {
    BasicEnemy,
}

pub type EnemyTuple = (Position, Velocity, ColorRect, Enemy, HP);
pub fn new_enemy(enemy_type: Enemy, pos: Point) -> EnemyTuple {
    let pos = Position(pos);
    let vel = Velocity::default();
    let (color_rect, hp) = match enemy_type {
        Enemy::BasicEnemy => (
            ColorRect {
                color: Color::new(1.0, 0.0, 0.0, 1.0),
                w: 30.0,
                h: 30.0,
            },
            1,
        ),
    };

    (pos, vel, color_rect, enemy_type, HP(hp))
}

pub fn create_enemy(world: &mut World, enemy: &EnemyTuple) -> Entity {
    world
        .create_entity()
        .with(enemy.0)
        .with(enemy.1)
        .with(enemy.2)
        .with(enemy.3)
        .with(enemy.4)
        .build()
}

#[derive(Clone, Copy, Debug, PartialEq, Component, Default)]
#[storage(NullStorage)]
pub struct Player;

#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[storage(HashMapStorage)]
pub struct PlayerEntity(pub Entity);

pub type PlayerTuple = (Position, Velocity, HP, ColorRect, Player);
pub fn new_player(hp: u32) -> PlayerTuple {
    let pos = Position([288.0 - 30.0, 768.0].into());
    let vel = Velocity::default();
    let hp = HP(hp);
    let rect = ColorRect {
        color: Color::new(1.0, 1.0, 1.0, 1.0),
        w: 60.0,
        h: 100.0,
    };

    (pos, vel, hp, rect, Player::default())
}

pub fn create_player(world: &mut World, player: &PlayerTuple) -> Entity {
    world
        .create_entity()
        .with(player.0)
        .with(player.1)
        .with(player.2)
        .with(player.3)
        .with(player.4)
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
        let x = rng.gen_range(0.0, 576.0);
        let y = rng.gen_range(-576.0, 0.0);
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
