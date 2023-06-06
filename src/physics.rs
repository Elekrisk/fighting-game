use crate::{
    animation::{Hitbox2, Hitboxes, HitboxType},
    Position, Frameticker, character::{Team, Character}, effects::Effects,
};
use bevy::prelude::*;

#[derive(Component)]
pub struct Collisions {
    pub collisions: Vec<Collision>,
}

pub struct Collision {
    pub other_entity: Entity,
    pub other_team: Team,
    pub did_the_hitting: bool,
}

pub(crate) fn collisions(frame_ticker: Res<Frameticker>, mut query: Query<(Entity, &Team, &Position, &Hitboxes, &mut Collisions)>) {

    let mut iterator = query.iter_combinations_mut();
    while let Some([(aentity, ateam, apos, ahitboxes, mut acollisions), (bentity, bteam, bpos, bhitboxes, mut bcollisions)]) = iterator.fetch_next()  {
        for ahitbox in &ahitboxes.hitboxes {
            for bhitbox in &bhitboxes.hitboxes {
                if ahitbox.hitbox_type == bhitbox.hitbox_type {
                    continue;
                }

                let apos = apos.0 + ahitbox.offset;
                let asize = ahitbox.size;
                let bpos = bpos.0 + bhitbox.offset;
                let bsize = bhitbox.size;

                if apos.x < bpos.x + bsize.x
                    && apos.x + asize.x > bpos.x
                    && apos.y > bpos.y - bsize.y
                    && apos.y - asize.y < bpos.y
                {
                    println!("{}: COLLISION! {} and {}", frame_ticker.current_frame, ahitbox.tag, bhitbox.tag);
                    acollisions.collisions.push(Collision {
                        other_entity: bentity,
                        other_team: *bteam,
                        did_the_hitting: ahitbox.hitbox_type == HitboxType::Hitbox,
                    });
                    bcollisions.collisions.push(Collision {
                        other_entity: aentity,
                        other_team: *ateam,
                        did_the_hitting: bhitbox.hitbox_type == HitboxType::Hitbox,
                    });
                }
            }
        }

        // println!("{}, {}", ahitboxes.hitboxes.len(), bhitboxes.hitboxes.len());
    }
}

pub fn collision_resolver(mut player_query: Query<(&Team, &mut Collisions, &mut Character, &mut Effects)>) {
    let mut effects_to_apply = vec![];

    for (team, mut collisions, mut character, mut effects) in player_query.iter_mut() {
        for collision in std::mem::take(&mut collisions.collisions) {
            if collision.other_team == *team {
                println!("Skipping same team collision");
                continue;
            }
            if collision.did_the_hitting {
                if let Some(effects) = &character.current_move_on_hit {
                    effects_to_apply.push((collision.other_entity, effects.clone()));
                }
            }
        }
    }

    for (entity, mut effects) in effects_to_apply {
        player_query.get_mut(entity).unwrap().3.effects.append(&mut effects);
    }
}

//  |---|         |OK  |---|    |OK    |---|  |        |---|  |   |---|   |  |-----|
//         |---|  |      |---|  |    |---|    | |---|         |  |-----|  |   |---|
