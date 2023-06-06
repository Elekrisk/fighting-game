use std::collections::HashMap;

use bevy::{ecs::schedule::SystemConfigs, prelude::*};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    animation::{Animation, Animator},
    effects::Effect,
    fixedpoint::FixedPoint,
    movelist::{Button, Movelist},
    AbsoluteDirection, Frameticker, Velocity, Position,
};

#[derive(Component, Default)]
pub struct Character {
    pub facing: FacingDirection,
    pub state: CharacterState,
    pub just_transitioned: bool,
    pub animations: HashMap<&'static str, Handle<Animation>>,
    pub new_anim: bool,
    pub input_dir: crate::AbsoluteDirection,
    pub current_move_on_hit: Option<Vec<Effect>>,
}

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Team {
    Team1,
    Team2,
    Neutral,
}

#[derive(Bundle)]
pub struct CharacterBundle {
    pub character: Character,
    pub input_history: InputHistory,
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum FacingDirection {
    Left,
    #[default]
    Right,
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum CharacterState {
    #[default]
    Idle,
    MovingForward,
    MovingBackward,
    Normal,

    Hitstun(usize),
    Blockstun(usize),
}

#[derive(Component, Default)]
pub struct InputHistory {
    pub last_dir: AbsoluteDirection,
    pub move_buffer: Vec<InputAction>,
}

impl InputHistory {
    pub const BUFFER: usize = 5;

    pub fn find_last(&self, predicate: impl FnMut(&&InputAction) -> bool) -> Option<&InputAction> {
        self.move_buffer.iter().rev().find(predicate)
    }

    pub fn find_last_mapped<T>(
        &self,
        predicate: impl FnMut(&InputAction) -> Option<T>,
    ) -> Option<T> {
        self.move_buffer.iter().rev().find_map(predicate)
    }

    pub fn find_last_pressed_dir(&self, dir: AbsoluteDirection) -> Option<usize> {
        self.find_last_mapped(|ia| if let InputActionKind::PressDirection(d) = &ia.kind && *d == dir { Some(ia.time) } else { None })
    }

    pub fn find_last_pressed_button(&self, btn: Button) -> Option<usize> {
        self.find_last_mapped(|ia| if let InputActionKind::PressButton(b) = &ia.kind && *b == btn { Some(ia.time) } else { None })
    }
}

#[derive(Component)]
pub struct Health {
    pub value: FixedPoint,
}
pub struct InputAction {
    pub time: usize,
    pub kind: InputActionKind,
}

pub enum InputActionKind {
    PressDirection(AbsoluteDirection),
    ReleaseDirection {
        direction: AbsoluteDirection,
        duration: usize,
    },
    PressButton(Button),
    ReleaseButton {
        button: Button,
        duration: usize,
    },
}

pub(crate) fn input_manager(mut query: Query<(&mut Character, &ActionState<crate::Input>)>) {
    for (mut character, input) in query.iter_mut() {
        let dir = input
            .clamped_axis_pair(crate::Input::Movement)
            .unwrap()
            .rotation()
            .map_or(AbsoluteDirection::Neutral, AbsoluteDirection::from);
        character.input_dir = dir;
    }
}

pub(crate) fn facing_corrector(mut query: Query<(Entity, &Position, &mut Character, &Team)>) {
    let (p1, p1pos, _, _) = query.iter().find(|(_, _, _, t)| **t == Team::Team1).unwrap();
    let (p2, p2pos, _, _) = query.iter().find(|(_, _, _, t)| **t == Team::Team2).unwrap();

    let p1pos = *p1pos;
    let p2pos = *p2pos;

    if p1pos.0.x < p2pos.0.x {
        query.get_component_mut::<Character>(p1).unwrap().facing = FacingDirection::Right;
        query.get_component_mut::<Character>(p2).unwrap().facing = FacingDirection::Left;
    } else if p1pos.0.x > p2pos.0.x {
        query.get_component_mut::<Character>(p1).unwrap().facing = FacingDirection::Left;
        query.get_component_mut::<Character>(p2).unwrap().facing = FacingDirection::Right;
    }
}

pub(crate) fn state_manager(
    frameticker: Res<Frameticker>,
    mut query: Query<(
        &mut Character,
        &mut Velocity,
        &Movelist,
        &InputHistory,
        &ActionState<crate::Input>,
        &mut Animator,
        &mut TextureAtlasSprite,
    )>,
) {
    for (mut character, mut velocity, movelist, input_history, input, mut animator, mut sprite) in
        query.iter_mut()
    {
        if let CharacterState::Hitstun(frames) = character.state {
            if frames == 0 {
                character.state = CharacterState::Idle;
                println!("{}: Player returned to normal after hitstun", frameticker.current_frame)
            } else {
                character.state = CharacterState::Hitstun(frames - 1);
            }
        }
        if let CharacterState::Blockstun(frames) = character.state {
            if frames == 0 {
                character.state = CharacterState::Idle;
                println!("{}: Player returned to normal after blockstun", frameticker.current_frame)
            } else {
                character.state = CharacterState::Blockstun(frames - 1);
            }
        }

        match character.state {
            CharacterState::Idle => {
                if character.input_dir.flipped(character.facing) == AbsoluteDirection::Right {
                    character.state = CharacterState::MovingForward;
                    character.just_transitioned = true;
                } else if character.input_dir.flipped(character.facing) == AbsoluteDirection::Left {
                    character.state = CharacterState::MovingBackward;
                    character.just_transitioned = true;
                }
            }
            CharacterState::MovingForward => {
                if character.input_dir.flipped(character.facing) == AbsoluteDirection::Left {
                    character.state = CharacterState::MovingBackward;
                    character.just_transitioned = true;
                } else if character.input_dir.flipped(character.facing)
                    == AbsoluteDirection::Neutral
                {
                    character.state = CharacterState::Idle;
                    character.just_transitioned = true;
                }
            }
            CharacterState::MovingBackward => {
                if character.input_dir.flipped(character.facing) == AbsoluteDirection::Right {
                    character.state = CharacterState::MovingForward;
                    character.just_transitioned = true;
                } else if character.input_dir.flipped(character.facing)
                    == AbsoluteDirection::Neutral
                {
                    character.state = CharacterState::Idle;
                    character.just_transitioned = true;
                }
            }
            CharacterState::Normal => {
                // if character.input_dir.flipped(character.facing) == AbsoluteDirection::Down {
                //     character.state = CharacterState::Idle;
                //     character.just_transitioned = true;
                // }
            }
            _ => {}
        }

        for mov in &movelist.moves {
            if mov.valid_in_states.matches(character.state)
                && mov.input_matcher.matches(
                    input_history,
                    character.facing,
                    frameticker.current_frame,
                )
            {
                character.state = mov.to_state;
                character.just_transitioned = false;
                character.current_move_on_hit = Some(mov.effects.clone());
                animator.animation = mov.animation.clone();
                sprite.index = 0;
                animator.last_frame_change = frameticker.current_frame;
                animator.just_changed_animation = true;
                animator.idle_after_animation = true;

                println!("{}: Player used {}", frameticker.current_frame, mov.name);
            }
        }

        if character.just_transitioned {
            // println!("JUST TRANSITIONED");
            character.just_transitioned = false;
            let (id, stop_after) = match character.state {
                CharacterState::Idle => ("idle", false),
                CharacterState::MovingForward => {
                    if character.new_anim {
                        ("walking_forward_2", false)
                    } else {
                        ("walking_forward", false)
                    }
                }
                CharacterState::MovingBackward => ("walking_backward", false),
                CharacterState::Hitstun(_) => ("idle", false),
                CharacterState::Blockstun(_) => ("idle", false),
                _ => unreachable!(),
            };

            let anim = &character.animations[id];
            animator.animation = anim.clone();
            animator.last_frame_change = frameticker.current_frame;
            sprite.index = 0;
            animator.just_changed_animation = true;
            animator.idle_after_animation = stop_after;
        }

        let modifier = if character.facing == FacingDirection::Right {
            1.0
        } else {
            -1.0
        };

        match character.state {
            CharacterState::MovingForward => velocity.0.x = FixedPoint::from(1.0 * modifier),
            CharacterState::MovingBackward => velocity.0.x = FixedPoint::from(-1.0 * modifier),
            CharacterState::Blockstun(_) | CharacterState::Hitstun(_) => {
                velocity.0.x *= FixedPoint::from(0.9);
                // println!("{:?}", velocity.0.x);
            }
            _ => velocity.0.x = FixedPoint::from(0.0),
        }
    }
}
