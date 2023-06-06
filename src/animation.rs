use std::collections::HashMap;

use bevy::{
    asset::{Asset, AssetLoader, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    sprite::Anchor,
    utils::BoxedFuture,
};
use image::DynamicImage;
use serde::{Deserialize, Serialize};

use crate::{
    character::{Character, CharacterState, FacingDirection},
    fixedpoint::FixedPoint,
    Frameticker, Position,
};

pub fn init(app: &mut App) {
    app.add_asset::<Animation>();
    app.add_asset_loader(AnimationLoader);
}

struct AnimationLoader;

impl AssetLoader for AnimationLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async {
            let anim: AnimationFileData = serde_json::from_slice(bytes)?;

            let sprite_sheet = image::load_from_memory(&anim.spritesheet).unwrap();
            let image = Image::from_dynamic(sprite_sheet, true);
            let handle =
                load_context.set_labeled_asset("spritesheet_image", LoadedAsset::new(image));
            let handle = load_context.set_labeled_asset(
                "spritesheet",
                LoadedAsset::new(TextureAtlas::from_grid(
                    handle,
                    Vec2::new(anim.info.cell_width as _, anim.info.cell_height as _),
                    anim.info.columns,
                    anim.info.frame_count.div_ceil(anim.info.columns),
                    None,
                    None,
                )),
            );

            let anim = Animation {
                spritesheet: Spritesheet {
                    image: handle,
                    cell_width: anim.info.cell_width,
                    cell_height: anim.info.cell_height,
                    colums: anim.info.columns,
                    frame_count: anim.info.frame_count,
                },
                frames: anim
                    .info
                    .frame_data
                    .into_iter()
                    .map(|fd| Frame {
                        duration: fd.delay,
                        offset: fd.origin,
                        root_motion: fd.root_motion,
                        hitboxes: fd.hitboxes,
                    })
                    .collect(),
                hitboxes: anim.info.hitboxes,
            };

            load_context.set_default_asset(LoadedAsset::new(anim));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["anim"]
    }
}

#[derive(Serialize, Deserialize)]
struct AnimationFileData {
    #[serde(with = "seethe")]
    spritesheet: Vec<u8>,
    info: Info,
}

mod seethe {
    use base64::Engine;
    use serde::{de::Visitor, Deserializer, Serializer};

    pub(super) fn serialize<S: Serializer>(bytes: &[u8], mut s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            s.serialize_str(&base64::engine::general_purpose::STANDARD_NO_PAD.encode(bytes))
        } else {
            s.serialize_bytes(bytes)
        }
    }

    pub(super) fn deserialize<'d, D: Deserializer<'d>>(mut d: D) -> Result<Vec<u8>, D::Error> {
        struct V;

        impl Visitor<'_> for V {
            type Value = Vec<u8>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("data")
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v)
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v.into())
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(base64::engine::general_purpose::STANDARD_NO_PAD
                    .decode(v)
                    .unwrap())
            }
        }

        if d.is_human_readable() {
            d.deserialize_str(V)
        } else {
            d.deserialize_byte_buf(V)
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Info {
    cell_width: usize,
    cell_height: usize,
    columns: usize,
    frame_count: usize,
    frame_data: Vec<FrameData>,
    hitboxes: HashMap<usize, Hitbox>,
}

#[derive(Serialize, Deserialize)]
struct FrameData {
    delay: usize,
    origin: Vec2,
    root_motion: Vec2,
    hitboxes: HashMap<usize, HitboxPos>,
}

#[derive(Clone, TypeUuid)]
#[uuid = "5fe2f03e-3d6f-4ac5-95ec-132d62b816fd"]
pub struct Animation {
    pub spritesheet: Spritesheet,
    frames: Vec<Frame>,
    hitboxes: HashMap<usize, Hitbox>,
}

#[derive(Clone)]
pub struct Spritesheet {
    pub image: Handle<TextureAtlas>,
    cell_width: usize,
    cell_height: usize,
    colums: usize,
    frame_count: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Hitbox {
    id: usize,
    #[serde(alias = "desc")]
    tag: String,
    is_hurtbox: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HitboxPos {
    id: usize,
    pos: Vec2,
    size: Vec2,
    enabled: bool,
}

#[derive(Clone)]
pub struct Frame {
    duration: usize,
    offset: Vec2,
    root_motion: Vec2,
    hitboxes: HashMap<usize, HitboxPos>,
}

#[derive(Component)]
pub struct Animator {
    pub animation: Handle<Animation>,
    pub just_changed_animation: bool,
    pub last_frame_change: usize,
    pub idle_after_animation: bool,
}

#[derive(Component, Debug)]
pub struct Hitboxes {
    pub hitboxes: Vec<Hitbox2>,
}

#[derive(Debug)]
pub struct Hitbox2 {
    pub offset: crate::Vec2,
    pub size: crate::Vec2,
    pub tag: String,
    pub hitbox_type: HitboxType,
    entity: Option<Entity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitboxType {
    Hurtbox,
    Hitbox
}

pub(crate) fn animator(
    frame_ticker: Res<Frameticker>,
    mut query: Query<(
        &mut Character,
        &mut Position,
        &mut Animator,
        &mut TextureAtlasSprite,
        &mut Handle<TextureAtlas>,
        &mut Hitboxes,
    )>,
    animations: Res<Assets<Animation>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (mut character, mut pos, mut anim, mut sprite, mut atlas, mut hitboxes) in query.iter_mut()
    {
        let flip = character.facing ==  FacingDirection::Left;
        let modifier = if flip { -1.0 } else { 1.0 };
        let modifier_vec = Vec2::new(modifier, 1.0);
        sprite.flip_x = flip;

        let mut diff = frame_ticker.current_frame - anim.last_frame_change;
        let cur_frame = sprite.index;

        let mut just_changed = false;
        let last_frame = cur_frame;

        let animation = animations.get(&anim.animation).unwrap();

        *atlas = animation.spritesheet.image.clone();

        while diff >= animation.frames[sprite.index].duration {
            sprite.index += 1;
            if sprite.index >= animation.frames.len() {
                if anim.idle_after_animation {
                    sprite.index = animation.frames.len() - 1;
                    character.state = CharacterState::Idle;
                    character.just_transitioned = true;
                    character.current_move_on_hit = None;
                    println!("{}: Player returned to normal from move", frame_ticker.current_frame + 1);
                    return;
                }
                sprite.index = 0;
            }
            anim.last_frame_change = frame_ticker.current_frame;
            just_changed = true;

            diff = frame_ticker.current_frame - anim.last_frame_change;
        }

        if just_changed || anim.just_changed_animation {
            anim.just_changed_animation = false;

            let cur_frame = sprite.index;

            let cell_size = Vec2::new(
                animation.spritesheet.cell_width as _,
                animation.spritesheet.cell_height as _,
            );

            sprite.anchor = Anchor::Custom(
                (animation.frames[cur_frame].offset / cell_size - Vec2::new(0.5, 0.5))
                    * Vec2::new(modifier, -1.0),
            );

            // for hb in &hitboxes.hitboxes {
            //     // println!("Despawning");
            //     commands.get_entity(hb.entity.unwrap()).unwrap().despawn();
            // }

            hitboxes.hitboxes.clear();

            for hp in animation.frames[cur_frame].hitboxes.values() {
                let hb = &animation.hitboxes[&hp.id];

                if hp.enabled {
                    // let e = commands
                    //     .spawn(SpriteBundle {
                    //         texture: asset_server.load("pixel.png"),
                    //         transform: Transform {
                    //             translation: Vec3 {
                    //                 x: f32::from(pos.0.x) + if flip { -hp.pos.x - hp.size.x / 2.0 } else { hp.pos.x + hp.size.x / 2.0 },
                    //                 y: f32::from(pos.0.y) + hp.pos.y - hp.size.y / 2.0,
                    //                 z: 0.0,
                    //             },
                    //             scale: Vec3 {
                    //                 x: hp.size.x,
                    //                 y: hp.size.y,
                    //                 z: 1.0,
                    //             },
                    //             ..default()
                    //         },
                    //         ..default()
                    //     })
                    //     .id();

                    hitboxes.hitboxes.push(Hitbox2 {
                        offset: crate::Vec2 {
                            x: if flip { -hp.pos.x - hp.size.x } else { hp.pos.x }.into(),
                            y: hp.pos.y.into(),
                        },
                        size: crate::Vec2 {
                            x: hp.size.x.into(),
                            y: hp.size.y.into(),
                        },
                        tag: hb.tag.clone(),
                        hitbox_type: if hb.is_hurtbox {
                            HitboxType::Hurtbox
                        } else {
                            HitboxType::Hitbox
                        },
                        entity: None,
                    })
                }
            }

            // println!("Updating hitboxes:");
            // println!("{hitboxes:?}");

            // println!("{:?}", sprite.anchor);
            // if just_changed {
            //     // let root_motion = animation.frames[cur_frame].root_motion;

            //     // let root_motion = crate::Vec2 {
            //     //     x: root_motion.x.into(),
            //     //     y: root_motion.y.into(),
            //     // };

            //     // let last_root_motion = animation.frames[last_frame].root_motion;

            //     // let last_root_motion = crate::Vec2 {
            //     //     x: last_root_motion.x.into(),
            //     //     y: last_root_motion.y.into(),
            //     // };

            //     // pos.0 = pos.0 + root_motion;
            //     // if last_frame < cur_frame {
            //     //     pos.0 = pos.0 - last_root_motion;
            //     // }
            //     if pos.0.x > FixedPoint::from(200.0) {
            //         pos.0.x = FixedPoint::from(-200.0);
            //     }
            //     if pos.0.x < FixedPoint::from(-200.0) {
            //         pos.0.x = FixedPoint::from(200.0);
            //     }
            // }
        }
    }
}
