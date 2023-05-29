#![feature(default_free_fn)]

use std::{
    default::default,
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};

use bevy::{
    asset::LoadState,
    prelude::{
        App, AssetEvent, AssetServer, Assets, Camera2dBundle, ClearColor, Color, Commands,
        Component, CoreSchedule, CoreSet, EventReader, FixedTime, GamepadButtonType, Handle, Image,
        ImagePlugin, IntoSystemConfig, KeyCode, Msaa, PluginGroup, Query, Res, ResMut, Resource,
        Transform,
    },
    render::{render_resource::FilterMode, texture::ImageSampler},
    sprite::{Anchor, Sprite, SpriteBundle, SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
    window::{Window, WindowPlugin},
    DefaultPlugins,
};
use leafwing_input_manager::{
    orientation::Rotation,
    prelude::{ActionState, InputManagerPlugin, InputMap, VirtualDPad},
    Actionlike, InputManagerBundle,
};

fn main() {
    let mut app = App::new();
    app.insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgb(0.7, 0.7, 0.9)))
        .insert_resource(Frameticker::default())
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Fighting Game".into(),
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugin(InputManagerPlugin::<Input>::default())
        .add_startup_system(startup)
        .add_system(input);
    app.get_schedule_mut(CoreSchedule::FixedUpdate)
        .unwrap()
        .add_system(velocity_system.after(input))
        .add_system(animation.after(velocity_system))
        .add_system(render_system.after(animation))
        .add_system(tick_frame.after(render_system));
    app.run();
}

#[derive(Component)]
struct Animation {
    first_frame: usize,
    last_frame: usize,
    frame_delay: usize,
    last_switch_frame: usize,
}

impl Animation {
    fn new(
        first_frame: usize,
        last_frame: usize,
        frame_delay: usize,
        last_switch_frame: usize,
    ) -> Self {
        Self {
            first_frame,
            last_frame,
            frame_delay,
            last_switch_frame,
        }
    }
}

#[derive(Component, PartialEq, Eq)]
struct Player(usize);

#[derive(Resource, Default)]
struct Frameticker {
    current_frame: usize,
}

fn tick_frame(mut frame_ticker: ResMut<Frameticker>) {
    frame_ticker.current_frame += 1;
}

fn animation(
    mut query: Query<(&Position, &mut TextureAtlasSprite, &mut Animation, &Player)>,
    mut q2: Query<(&Position, &Player)>,
    frame_ticker: Res<Frameticker>,
) {
    let cur_frame = frame_ticker.current_frame;

    query.for_each_mut(|(position, mut sprite, mut anim, player)| {
        let frames_passed = cur_frame - anim.last_switch_frame;

        let mut reverse = false;

        q2.for_each(|(pos, p)| {
            if p != player && pos.0.x.0 < position.0.x.0 {
                reverse = true;
            }
        });

        sprite.flip_x = reverse;

        if frames_passed >= anim.frame_delay {
            let mut frame = sprite.index + 1;
            anim.last_switch_frame = cur_frame;
            if frame > anim.last_frame {
                frame = anim.first_frame;
            }

            sprite.index = frame;
        }
    })
}

fn startup(
    mut commands: Commands,
    mut asset_server: ResMut<AssetServer>,
    mut assets: ResMut<Assets<Image>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let mut b = Camera2dBundle::default();
    b.projection.scale = 0.2;
    b.transform.translation.y += 45.0;

    commands.spawn(b);

    let mut input_map_p1 = InputMap::default();

    input_map_p1.insert(VirtualDPad::wasd(), Input::Movement);
    input_map_p1.insert(KeyCode::J, Input::Punch);
    input_map_p1.insert(KeyCode::K, Input::Kick);

    let mut input_map_p2 = InputMap::default();

    input_map_p2.insert(VirtualDPad::dpad(), Input::Movement);
    input_map_p2.insert(GamepadButtonType::West, Input::Punch);
    input_map_p2.insert(GamepadButtonType::South, Input::Kick);

    let player_idle: Handle<Image> = asset_server.load("player_idle.png");
    let texture_atlas = TextureAtlas::from_grid(
        player_idle,
        bevy::prelude::Vec2::new(64.0, 64.0),
        2,
        1,
        None,
        None,
    );
    let player_idle = texture_atlases.add(texture_atlas);

    spawn_player(
        &mut commands,
        1,
        -50.0,
        input_map_p1,
        player_idle.clone(),
        &asset_server,
    );
    spawn_player(
        &mut commands,
        2,
        50.0,
        input_map_p2,
        player_idle,
        &asset_server,
    );
}

fn spawn_player(
    commands: &mut Commands,
    player: usize,
    x_pos: f32,
    input_map: InputMap<Input>,
    player_idle: Handle<TextureAtlas>,
    asset_server: &AssetServer,
) {
    // let bundle = SpriteBundle {
    //     texture: asset_server.load("player.png"),
    //     sprite: Sprite {
    //         anchor: Anchor::BottomCenter,
    //         ..default()
    //     },
    //     ..default()
    // };
    let mut bundle = SpriteSheetBundle {
        sprite: TextureAtlasSprite::new(0),
        texture_atlas: player_idle,
        ..default()
    };
    bundle.sprite.anchor = Anchor::BottomCenter;

    commands
        .spawn(InputManagerBundle::<Input> {
            action_state: ActionState::default(),
            input_map: input_map,
        })
        .insert(InputHistory::default())
        .insert(bundle)
        .insert(Position(Vec2 {
            x: (x_pos).into(),
            y: FixedPoint(0),
        }))
        .insert(Velocity(Vec2 {
            x: FixedPoint(0),
            y: FixedPoint(0),
        }))
        .insert(Character {
            state: CharacterState::Grounded,
        })
        .insert(Animation::new(0, 1, 30, 0))
        .insert(Player(player));
}

fn input(
    mut query: Query<(
        &ActionState<Input>,
        &mut InputHistory,
        &mut Velocity,
        &mut Character,
    )>,
) {
    query.for_each_mut(
        |(action_state, mut input_history, mut velocity, mut character)| {
            let dir = action_state
                .clamped_axis_pair(Input::Movement)
                .unwrap()
                .rotation()
                .map(|rot| rot.into())
                .unwrap_or(AbsoluteDirection::Neutral);

            if dir != input_history.last_dir {
                input_history.last_dir = dir;
                // println!("{dir:?}");
            }
            if action_state.just_pressed(Input::Punch) {
                // println!("Punch");
            }
            if action_state.just_pressed(Input::Kick) {
                // println!("Kick");
            }

            if character.state == CharacterState::Grounded {
                if dir == AbsoluteDirection::Left {
                    velocity.0.x = (-1f32).into();
                } else if dir == AbsoluteDirection::Right {
                    velocity.0.x = 1f32.into();
                } else if dir == AbsoluteDirection::Neutral {
                    velocity.0.x = 0f32.into();
                }

                if dir.is_up() {
                    velocity.0.y = 4f32.into();
                    character.state = CharacterState::Jumping;
                }
            }
        },
    );
}

fn velocity_system(mut query: Query<(&mut Position, &mut Velocity, &mut Character)>) {
    query.for_each_mut(|(mut pos, mut vel, mut character)| {
        pos.0 = pos.0 + vel.0;
        vel.0.y -= FixedPoint::from(0.2);

        if pos.0.y.0 < 0 {
            pos.0.y.0 = 0;
            vel.0.y.0 = 0;
            character.state = CharacterState::Grounded;
        }
    });
}

fn render_system(mut query: Query<(&Position, &mut Transform)>) {
    query.for_each_mut(|(pos, mut transform)| {
        transform.translation.x = pos.0.x.into();
        transform.translation.y = pos.0.y.into();
    })
}

#[derive(Actionlike, Clone, Copy)]
enum Input {
    Movement,
    Punch,
    Kick,
}

#[derive(Component, Default)]
struct InputHistory {
    last_dir: AbsoluteDirection,
}

#[derive(Component)]
struct Character {
    state: CharacterState,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum CharacterState {
    Grounded,
    Jumping,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct FixedPoint(i64);

impl FixedPoint {
    const DECIMALS: usize = 16;
}

impl Add for FixedPoint {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for FixedPoint {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for FixedPoint {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for FixedPoint {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for FixedPoint {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self((self.0 * rhs.0) >> Self::DECIMALS)
    }
}

impl MulAssign for FixedPoint {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl From<FixedPoint> for f32 {
    fn from(value: FixedPoint) -> Self {
        value.0 as f32 / 2usize.pow(FixedPoint::DECIMALS as _) as f32
    }
}

impl From<f32> for FixedPoint {
    fn from(value: f32) -> Self {
        let v = value * 2usize.pow(FixedPoint::DECIMALS as _) as f32;
        if v > i64::MAX as f32 {
            Self(i64::MAX)
        } else if v < i64::MIN as f32 {
            Self(i64::MIN)
        } else {
            Self(v as i64)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Vec2 {
    x: FixedPoint,
    y: FixedPoint,
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<FixedPoint> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: FixedPoint) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

#[derive(Component)]
struct Position(Vec2);

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum AbsoluteDirection {
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
    Up,
    UpRight,
    #[default]
    Neutral,
}

impl AbsoluteDirection {
    fn is_up(&self) -> bool {
        matches!(self, Self::Up | Self::UpRight | Self::UpLeft)
    }
}

impl From<Rotation> for AbsoluteDirection {
    fn from(value: Rotation) -> Self {
        match (value.deci_degrees() + 225) % 3600 / 450 {
            0 => Self::Right,
            1 => Self::UpRight,
            2 => Self::Up,
            3 => Self::UpLeft,
            4 => Self::Left,
            5 => Self::DownLeft,
            6 => Self::Down,
            7 => Self::DownRight,
            _ => unreachable!(),
        }
    }
}
