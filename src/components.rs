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
    let hp = HP(0);
    let rect = ColorRect {
        color: Color::new(1.0, 1.0, 1.0, 1.0),
        w: 60.0,
        h: 100.0,
    };

    (pos, vel, hp, rect, Player::default())
}

pub fn create_player(world: &mut World, player: &PlayerTuple) -> Entity {
    world.create_entity()
        .with(player.0)
        .with(player.1)
        .with(player.2)
        .with(player.3)
        .with(player.4)
        .build()
}
