use crate::{
    character::{Character, Health, Team},
    fixedpoint::FixedPoint,
    Frameticker,
};
use bevy::prelude::*;

#[derive(Component)]
pub struct KoTextMarker;

pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(100.0)),
                padding: UiRect {
                    left: Val::Px(25.0),
                    right: Val::Px(25.0),
                    top: Val::Px(25.0),
                    bottom: Val::Px(25.0),
                },
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size {
                            width: Val::Px(550.0),
                            height: Val::Px(40.0),
                        },
                        justify_content: JustifyContent::Start,
                        ..default()
                    },
                    background_color: Color::GRAY.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        NodeBundle {
                            style: Style {
                                size: Size::width(Val::Percent(100.0)),
                                ..default()
                            },
                            background_color: Color::RED.into(),
                            ..default()
                        },
                        Team::Team1,
                    ));
                });
            parent.spawn({
                let mut text = TextBundle::from_section(
                    "KO",
                    TextStyle {
                        font: asset_server.load("VT323-Regular.ttf"),
                        font_size: 320.0,
                        ..default()
                    },
                );
                text.visibility = Visibility::Hidden;
                (text, KoTextMarker)
            });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size {
                            width: Val::Px(550.0),
                            height: Val::Px(40.0),
                        },
                        justify_content: JustifyContent::End,
                        ..default()
                    },
                    background_color: Color::GRAY.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        NodeBundle {
                            style: Style {
                                size: Size::width(Val::Percent(100.0)),
                                ..default()
                            },
                            background_color: Color::RED.into(),
                            ..default()
                        },
                        Team::Team2,
                    ));
                });
        });
}

pub(crate) fn ui_system(
    mut frame_ticker: ResMut<Frameticker>,
    player_query: Query<(&Health, &Team), With<Character>>,
    mut healths: Query<(&mut Style, &Team)>,
    mut ko_visibility: Query<&mut Visibility, With<KoTextMarker>>,
    mut text: Query<&mut Text, With<KoTextMarker>>,
) {
    for (health, team) in player_query.iter() {
        let (mut hp_style, _) = healths.iter_mut().find(|(_, tm)| *tm == team).unwrap();
        hp_style.size.width = Val::Percent(health.value.into());

        if health.value <= FixedPoint::ZERO {
            *ko_visibility.single_mut() = Visibility::Inherited;
            frame_ticker.pause = true;
        }
    }

    // text.single_mut().sections[0].value = format!("{}", frame_ticker.current_frame);
}
