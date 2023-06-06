use crate::{
    animation::Animation,
    character::{CharacterState, InputHistory, FacingDirection, InputActionKind}, effects::Effect,
};
use bevy::prelude::*;

#[derive(Component)]
pub struct Movelist {
    pub moves: Vec<Move>,
}

pub struct Move {
    pub name: String,
    pub input_matcher: InputMatcher,
    pub valid_in_states: StateMatcher,
    pub to_state: CharacterState,
    pub animation: Handle<Animation>,
    pub effects: Vec<Effect>,
}

pub struct StateMatcher {
    matcher: Box<dyn Fn(CharacterState) -> bool + Send + Sync>,
}

impl StateMatcher {
    pub fn idle() -> Self {
        Self {
            matcher: Box::new(|character_state| character_state == CharacterState::Idle),
        }
    }

    pub fn forward() -> Self {
        Self {
            matcher: Box::new(|character_state| character_state == CharacterState::MovingForward),
        }
    }

    pub fn backward() -> Self {
        Self {
            matcher: Box::new(|character_state| character_state == CharacterState::MovingBackward),
        }
    }

    pub fn all() -> Self {
        Self {
            matcher: Box::new(|character_state| matches!(character_state, CharacterState::Idle | CharacterState::MovingBackward | CharacterState::MovingForward))
        }
    }

    pub fn matches(&self, state: CharacterState) -> bool {
        (self.matcher)(state)
    }
}

pub enum InputMatcher {
    Button(Button),
}

impl InputMatcher {
    pub fn matches(&self, input_history: &InputHistory, facing_direction: FacingDirection, current_frame: usize) -> bool {
        match self {
            InputMatcher::Button(button) => {
                for action in input_history.move_buffer.iter().rev() {
                    if current_frame - action.time > InputHistory::BUFFER {
                        return false;
                    }

                    if let InputActionKind::PressButton(b) = &action.kind && b == button {
                        return true;
                    }
                }
                false
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Punch,
    Kick,
}
