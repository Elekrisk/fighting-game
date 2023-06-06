#![feature(default_free_fn)]
#![feature(trivial_bounds)]
#![feature(let_chains)]
#![feature(int_roundings)]

mod animation;
mod character;
mod effects;
mod fixedpoint;
mod movelist;
mod physics;
mod ui;
mod vec2;

use std::{
    collections::HashMap,
    default::default,
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};

use animation::{Animation, Animator, Hitboxes};
use bevy::{
    asset::{AssetPath, LoadState},
    prelude::{
        App, AssetEvent, AssetServer, Assets, Camera2dBundle, ClearColor, Color, Commands,
        Component, CoreSchedule, CoreSet, EventReader, FixedTime, GamepadButtonType, Handle, Image,
        ImagePlugin, IntoSystemConfig, IntoSystemConfigs, KeyCode, Msaa, PluginGroup, Query, Res,
        ResMut, Resource, Transform,
    },
    render::{render_resource::FilterMode, texture::ImageSampler},
    sprite::{Anchor, Sprite, SpriteBundle, SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
    window::{Window, WindowPlugin},
    DefaultPlugins,
};
use character::{Health, InputAction, InputActionKind, InputHistory, Team, FacingDirection};
use effects::{Effect, Effects};
use fixedpoint::FixedPoint;
use leafwing_input_manager::{
    orientation::Rotation,
    prelude::{ActionState, InputManagerPlugin, InputMap, VirtualDPad},
    Actionlike, InputManagerBundle,
};
use movelist::{InputMatcher, Move, Movelist, StateMatcher};
use physics::Collisions;
use ui::setup_ui;
use vec2::Vec2;

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
        .add_startup_systems((startup, ui::setup_ui))
        .add_system(input);
    animation::init(&mut app);
    app.get_schedule_mut(CoreSchedule::FixedUpdate)
        .unwrap()
        .add_systems(
            (
                character::facing_corrector,
                character::input_manager,
                character::state_manager,
                velocity_system,
                animation::animator,
                physics::collisions,
                physics::collision_resolver,
                effects::apply_effects,
                ui::ui_system,
                render_system,
                tick_frame,
            )
                .chain(),
        );
    app.run();
}

// #[derive(Component)]
// struct Animation {
//     first_frame: usize,
//     last_frame: usize,
//     frame_delay: usize,
//     last_switch_frame: usize,
// }

// impl Animation {
//     fn new(
//         first_frame: usize,
//         last_frame: usize,
//         frame_delay: usize,
//         last_switch_frame: usize,
//     ) -> Self {
//         Self {
//             first_frame,
//             last_frame,
//             frame_delay,
//             last_switch_frame,
//         }
//     }
// }

#[derive(Component, PartialEq, Eq)]
struct Player(usize);

#[derive(Resource, Default)]
struct Frameticker {
    current_frame: usize,
    pause: bool,
}

fn tick_frame(mut frame_ticker: ResMut<Frameticker>) {
    if !frame_ticker.pause {
        frame_ticker.current_frame += 1;
    }
}

// fn animation(
//     mut query: Query<(&Position, &mut TextureAtlasSprite, &mut Animation, &Player)>,
//     mut q2: Query<(&Position, &Player)>,
//     frame_ticker: Res<Frameticker>,
// ) {
//     let cur_frame = frame_ticker.current_frame;

//     query.for_each_mut(|(position, mut sprite, mut anim, player)| {
//         let frames_passed = cur_frame - anim.last_switch_frame;

//         let mut reverse = false;

//         q2.for_each(|(pos, p)| {
//             if p != player && pos.0.x.0 < position.0.x.0 {
//                 reverse = true;
//             }
//         });

//         sprite.flip_x = reverse;

//         if frames_passed >= anim.frame_delay {
//             let mut frame = sprite.index + 1;
//             anim.last_switch_frame = cur_frame;
//             if frame > anim.last_frame {
//                 frame = anim.first_frame;
//             }

//             sprite.index = frame;
//         }
//     })
// }

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

    let anim = asset_server.load("c1_walking_v2.anim");

    spawn_player(
        &mut commands,
        1,
        -50.0,
        input_map_p1,
        anim.clone(),
        Team::Team1,
        &asset_server,
    );
    spawn_player(
        &mut commands,
        2,
        50.0,
        input_map_p2,
        anim,
        Team::Team2,
        &asset_server,
    );
}

fn spawn_player(
    commands: &mut Commands,
    player: usize,
    x_pos: f32,
    input_map: InputMap<Input>,
    player_idle: Handle<Animation>,
    team: Team,
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

    let path = asset_server.get_handle_path(&player_idle).unwrap();
    let atlas_path = AssetPath::new(path.path().to_path_buf(), Some("spritesheet".into()));
    let atlas = asset_server.get_handle(atlas_path);

    let mut bundle = SpriteSheetBundle {
        sprite: TextureAtlasSprite::new(0),
        texture_atlas: atlas,
        ..default()
    };
    bundle.sprite.anchor = Anchor::BottomCenter;

    let animations: HashMap<&str, Handle<Animation>> = [
        ("idle", asset_server.load("c1_idle.anim")),
        ("walking_forward", asset_server.load("c1_walking.anim")),
        ("walking_forward_2", asset_server.load("c1_walking_v2.anim")),
        ("walking_backward", asset_server.load("c1_walking_v2.anim")),
        ("punching", asset_server.load("c1_punch.anim")),
    ]
    .into_iter()
    .collect();

    commands
        .spawn(InputManagerBundle::<Input> {
            action_state: ActionState::default(),
            input_map: input_map,
        })
        .insert(InputHistory::default())
        .insert(bundle)
        .insert(Position(Vec2 {
            x: (x_pos).into(),
            y: FixedPoint::ZERO,
        }))
        .insert(Velocity(Vec2 {
            x: FixedPoint::ZERO,
            y: FixedPoint::ZERO,
        }))
        .insert(Character {
            state: CharacterState::Grounded,
        })
        .insert(character::Character {
            just_transitioned: true,
            animations: animations.clone(),
            new_anim: true,
            ..default()
        })
        .insert(Animator {
            animation: player_idle,
            last_frame_change: 0,
            just_changed_animation: true,
            idle_after_animation: false,
        })
        .insert(Movelist {
            moves: vec![Move {
                name: "Jab".into(),
                input_matcher: InputMatcher::Button(movelist::Button::Punch),
                valid_in_states: StateMatcher::all(),
                to_state: character::CharacterState::Normal,
                animation: animations["punching"].clone(),
                effects: vec![Effect::Damage(10.0.into()), Effect::Hitstun(21), Effect::Blockstun(15), Effect::Pushback(0.8.into())],
            }],
        })
        .insert(Hitboxes { hitboxes: vec![] })
        .insert(Collisions { collisions: vec![] })
        .insert(Effects { effects: vec![] })
        .insert(team)
        .insert(Health {
            value: 100.0.into(),
        })
        // .insert(Animation::new(0, 1, 30, 0))
        .insert(Player(player));
}

fn input(
    frame_ticker: Res<Frameticker>,
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
                if let Some(time) = input_history.find_last_pressed_dir(input_history.last_dir) {
                    let direction = input_history.last_dir;
                    input_history.move_buffer.push(InputAction {
                        time: frame_ticker.current_frame,
                        kind: InputActionKind::ReleaseDirection {
                            direction,
                            duration: frame_ticker.current_frame - time,
                        },
                    });
                }
                input_history.move_buffer.push(InputAction {
                    time: frame_ticker.current_frame,
                    kind: InputActionKind::PressDirection(dir),
                });
            }

            if action_state.just_pressed(Input::Punch) {
                input_history.move_buffer.push(InputAction {
                    time: frame_ticker.current_frame,
                    kind: InputActionKind::PressButton(movelist::Button::Punch),
                });
            }
            if action_state.just_released(Input::Punch) {
                if let Some(time) = input_history.find_last_pressed_button(movelist::Button::Punch)
                {
                    input_history.move_buffer.push(InputAction {
                        time: frame_ticker.current_frame,
                        kind: InputActionKind::ReleaseButton {
                            button: movelist::Button::Punch,
                            duration: frame_ticker.current_frame - time,
                        },
                    });
                }
            }
            if action_state.just_pressed(Input::Kick) {
                input_history.move_buffer.push(InputAction {
                    time: frame_ticker.current_frame,
                    kind: InputActionKind::PressButton(movelist::Button::Kick),
                });
            }
            if action_state.just_released(Input::Kick) {
                if let Some(time) = input_history.find_last_pressed_button(movelist::Button::Kick) {
                    input_history.move_buffer.push(InputAction {
                        time: frame_ticker.current_frame,
                        kind: InputActionKind::ReleaseButton {
                            button: movelist::Button::Kick,
                            duration: frame_ticker.current_frame - time,
                        },
                    });
                }
            }
        },
    );
}

fn velocity_system(mut query: Query<(&mut Position, &mut Velocity, &mut Character)>) {
    query.for_each_mut(|(mut pos, mut vel, mut character)| {
        pos.0 = pos.0 + vel.0;
        vel.0.y -= FixedPoint::from(0.2);

        if pos.0.y < FixedPoint::ZERO {
            pos.0.y = FixedPoint::ZERO;
            vel.0.y = FixedPoint::ZERO;
            character.state = CharacterState::Grounded;
        }

        if pos.0.x < FixedPoint::from(-100.0) {
            pos.0.x = FixedPoint::from(-100.0);
        }
        if pos.0.x > FixedPoint::from(100.0) {
            pos.0.x = FixedPoint::from(100.0);
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
#[derive(Component)]
struct Character {
    state: CharacterState,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum CharacterState {
    Grounded,
    Jumping,
}

#[derive(Component, Clone, Copy)]
struct Position(Vec2);

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AbsoluteDirection {
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

    fn flipped(&self, facing: FacingDirection) -> Self {
        if facing == FacingDirection::Left {
            match self {
                AbsoluteDirection::Right => AbsoluteDirection::Left,
                AbsoluteDirection::DownRight => AbsoluteDirection::DownLeft,
                AbsoluteDirection::Down => AbsoluteDirection::Down,
                AbsoluteDirection::DownLeft => AbsoluteDirection::DownRight,
                AbsoluteDirection::Left => AbsoluteDirection::Right,
                AbsoluteDirection::UpLeft => AbsoluteDirection::UpRight,
                AbsoluteDirection::Up => AbsoluteDirection::Up,
                AbsoluteDirection::UpRight => AbsoluteDirection::UpLeft,
                AbsoluteDirection::Neutral => AbsoluteDirection::Neutral,
            }
        } else {
            *self
        }
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
