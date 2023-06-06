use crate::{
    character::{Character, CharacterState, FacingDirection, Health},
    fixedpoint::FixedPoint,
    Velocity, Frameticker,
};
use bevy::prelude::*;

#[derive(Component)]
pub struct Effects {
    pub effects: Vec<Effect>,
}

#[derive(Clone)]
pub enum Effect {
    Damage(FixedPoint),
    Hitstun(usize),
    Blockstun(usize),
    Pushback(FixedPoint),
}

pub(crate) fn apply_effects(
    frameticker: Res<Frameticker>,
    mut query: Query<(&mut Health, &mut Velocity, &mut Character, &mut Effects)>,
) {
    for (mut health, mut velocity, mut character, mut effects) in query.iter_mut() {
        let blocking = matches!(
            character.state,
            CharacterState::MovingBackward | CharacterState::Blockstun(_)
        );
        for effect in std::mem::take(&mut effects.effects) {
            println!("Applying effect");
            match effect {
                Effect::Damage(dmg) if blocking => health.value -= dmg * FixedPoint::from(0.2),
                Effect::Damage(dmg) if !blocking => health.value -= dmg,
                Effect::Hitstun(frames) if !blocking => {
                    println!("{}: Player entered hitstun", frameticker.current_frame);
                    character.state = CharacterState::Hitstun(frames);
                    character.just_transitioned = true;
                }
                Effect::Blockstun(frames) if blocking => {
                    println!("{}: Player entered blockstun", frameticker.current_frame);
                    character.state = CharacterState::Blockstun(frames);
                    character.just_transitioned = true;
                }
                Effect::Pushback(vel) => {
                    velocity.0.x = vel
                        * if character.facing == FacingDirection::Right {
                            FixedPoint::from(-1.0)
                        } else {
                            FixedPoint::from(1.0)
                        };

                    // println!("{:?}", velocity.0.x);
                }
                _ => {}
            }
        }
    }
}
