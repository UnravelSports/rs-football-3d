use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, WorldInspectorPlugin};

use cute::c;

pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1080.0;

// import 'module' from src/data.rs
mod data;
mod player;

// use any public functions from src/data.rs
pub use data::*;
pub use player::*;

#[derive(Resource)]
pub struct GameAssets {
    goal_scene: Handle<Scene>,
    home_player_scene: Handle<Scene>,
    away_player_scene: Handle<Scene>,
    stadium_scene: Handle<Scene>,
    pitch_scene: Handle<Scene>,
    play_button: Handle<Image>,
}

fn main() {
    // create window
    App::new()
        // plugins:
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: WIDTH,
                height: HEIGHT,
                title: "UnravelSports  |  Football 3D".to_string(),
                resizable: false,
                ..Default::default()
            },
            ..default()
        }))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0,
        })
        // plugin:
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(PlayerPlugin)
        // types:
        // .register_type::<Player>() <- We add this via a Plugin now
        // systems before start up:
        .add_startup_system_to_stage(StartupStage::PreStartup, asset_loading)
        .add_startup_system_to_stage(StartupStage::PreStartup, json_loading)
        // systems at start up:
        .add_startup_system(spawn_basic_scene)
        .add_startup_system(create_ui)
        .add_startup_system(spawn_camera)
        // systems to run each frame
        .add_system(camera_controls)
        .add_system(update)
        .add_system(button_clicked)
        .run();
}

fn asset_loading(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        goal_scene: assets.load("goal_frame.glb#Scene0"),
        home_player_scene: assets.load("player_blue.glb#Scene0"),
        away_player_scene: assets.load("player_red.glb#Scene0"),
        stadium_scene: assets.load("stadium.glb#Scene0"),
        pitch_scene: assets.load("pitch.glb#Scene0"),
        play_button: assets.load("play_pause.png"),
    });
}

fn json_loading(mut commands: Commands) {
    let input_path = "assets/goal_sequence.json";
    commands.insert_resource(MatchData {
        data: {
            let match_data = std::fs::read_to_string(&input_path).expect("JSON Loading Failed...");
            serde_json::from_str::<MatchFrames>(&match_data).unwrap()
        },
        t: 0.0,
    })
}

fn spawn_basic_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_assets: Res<GameAssets>,
    match_data: ResMut<MatchData>,
) {
    commands
        .spawn(SceneBundle {
            scene: game_assets.stadium_scene.clone(),
            transform: Transform::from_xyz(-8.0, -0.5, -7.7),
            ..Default::default()
        })
        .insert(Name::new("StadiumModel"));

    commands
        .spawn(SceneBundle {
            scene: game_assets.pitch_scene.clone(),
            transform: Transform::from_xyz(0.1, 0.09, 0.0).with_scale(Vec3::new(0.468, 1.0, 0.468)),
            ..Default::default()
        })
        .insert(Name::new("PitchModel"));

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Quad {
                size: Vec2 { x: 105.0, y: 68.0 },
                flip: false,
            })),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_xyz(0.0, 0.05, 0.0)
                .with_rotation(Quat::from_rotation_x(-PI / 2.0)),
            ..default()
        })
        .insert(Name::new("Outerlines"));

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Quad {
                size: Vec2 {
                    x: 1000.0,
                    y: 1000.0,
                },
                flip: false,
            })),
            material: materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
            transform: Transform::from_xyz(0.0, -0.5, 0.0)
                .with_rotation(Quat::from_rotation_x(-PI / 2.0)),
            ..default()
        })
        .insert(Name::new("Underground"));

    let signs = vec![-1.0, 1.0];
    for i in &signs {
        commands
            .spawn(SceneBundle {
                scene: game_assets.goal_scene.clone(),
                transform: Transform::from_xyz(i * 53.13, -0.10, 0.0 + (i * 3.66))
                    // scale to human size
                    .with_scale(Vec3::new(0.01464, 0.01464, 0.01464))
                    // rotate facing goal or away from goal
                    .with_rotation(Quat::from_rotation_y((-PI / 2.0) * -i)),
                ..Default::default()
            })
            .insert(Name::new("GoalModel"));
    }

    let idx: usize = 0;

    for p in &match_data.data.players[idx] {
        println!("{:?}", p);

        let scene = if p.team == "home" {
            game_assets.home_player_scene.clone()
        } else {
            game_assets.away_player_scene.clone()
        };

        let theta = p.vy / p.vx;
        commands
            .spawn(SceneBundle {
                scene: scene,
                transform: Transform::from_xyz(p.x, 0.0, -1.0 * p.y)
                    .with_rotation(Quat::from_rotation_y(theta.tan())),

                ..Default::default()
            })
            .insert(Player { pid: p.pid })
            .insert(Name::new(format!("Player-{t}", t = p.team)));
    }

    let ball = &match_data.data.ball[idx];

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.216,
                subdivisions: 24,
            })),
            material: materials.add(Color::rgb(0.95, 0.95, 0.95).into()),
            transform: Transform::from_xyz(ball.x, 0.216 + ball.z, -1.0 * ball.y),
            ..default()
        })
        .insert(Player { pid: 0 })
        .insert(Name::new("BallModel"));
}

fn update(
    mut players: Query<(&Player, &mut Transform)>,
    buttons: Query<&Button>,
    mut match_data: ResMut<MatchData>,
    time: Res<Time>,
) {
    let mut is_play: bool = true;
    for button in &buttons {
        if matches!(button.kind, ButtonType::Play) {
            is_play = button.is_enabled;
        }
    }

    if is_play {
        let (idx, alpha) = match_data.get_interpolation_values_and_increment(time.delta_seconds());

        for (player, mut transform) in &mut players {
            if player.pid == 0 {
                // pid == 0 is the ball
                let b: &BallFrame = &match_data.data.ball[idx];
                let b1: &BallFrame = &match_data.data.ball[idx + 1];

                transform.translation.x = interpolate(alpha, b.x, b1.x);
                transform.translation.y = interpolate(alpha, b.z, b1.z) + 0.216; // add 0.216 so the ball is not half under the pitch
                transform.translation.z = interpolate(alpha, b.y, b1.y) * -1.0;
            } else {
                // sub-optimal way to find the PlayerFrame related to the player.pid that we happen to loop over right now
                // this is done in a Pythonic way with a crate(cute) that supports this
                let p: &PlayerFrame =
                    c![p, for p in &match_data.data.players[idx], if p.pid == player.pid][0];
                let p1: &PlayerFrame =
                    c![p, for p in &match_data.data.players[idx+1], if p.pid == player.pid][0];

                // interpolate x and z by taking the value from the current frame and the next frame;
                transform.translation.x = interpolate(alpha, p.x, p1.x);
                transform.translation.z = interpolate(alpha, p.y, p1.y) * -1.0;

                let vx: f32 = interpolate(alpha, p.vx, p1.vx);
                let vy: f32 = interpolate(alpha, p.vy, p1.vy);
                let theta = vy / vx;

                // only update rotation if player speed is more than 5ms, because lower results in some buggy/spinny players
                if (vx.powi(2) + vy.powi(2)).sqrt() > 5.0 {
                    let rot = transform.rotation.y;
                    transform.rotate_y(theta.tan() - rot);
                }
            }
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-25.0, 25.0, 10.0)
            .looking_at(Vec3::new(52.5, -20.0, -10.0), Vec3::Y),
        ..default()
    });
}

fn camera_controls(
    keyboard: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    let mut camera = camera_query.single_mut();

    let mut forward = camera.forward();
    forward.y = 0.0;
    forward = forward.normalize();

    let mut left = camera.left();
    left.y = 0.0;
    left = left.normalize();

    let speed = 10.0;
    let rotate_speed = 0.25;

    if keyboard.pressed(KeyCode::W) {
        camera.translation += forward * time.delta_seconds() * speed;
    }
    if keyboard.pressed(KeyCode::S) {
        camera.translation -= forward * time.delta_seconds() * speed;
    }
    if keyboard.pressed(KeyCode::A) {
        camera.translation += left * time.delta_seconds() * speed;
    }
    if keyboard.pressed(KeyCode::D) {
        camera.translation -= left * time.delta_seconds() * speed;
    }
    if keyboard.pressed(KeyCode::Q) {
        camera.rotate_axis(Vec3::Y, rotate_speed * time.delta_seconds())
    }
    if keyboard.pressed(KeyCode::E) {
        camera.rotate_axis(Vec3::Y, -rotate_speed * time.delta_seconds())
    }
}

#[derive(Inspectable, Component, Clone, Copy, Debug)]
pub struct Button {
    kind: ButtonType,
    is_enabled: bool,
}

#[derive(Inspectable, Component, Clone, Copy, Debug)]
pub enum ButtonType {
    Play,
}

fn create_ui(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|commands| {
            commands
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Percent(10.0 * 9.0 / 16.0), Val::Percent(10.0)),
                        align_self: AlignSelf::FlexEnd,
                        margin: UiRect::all(Val::Percent(2.0)),
                        ..default()
                    },
                    image: game_assets.play_button.clone().into(),
                    ..default()
                })
                .insert(Button {
                    kind: ButtonType::Play,
                    is_enabled: true,
                });
        });
}

fn button_clicked(
    mut interaction: Query<(&Interaction, &mut Button), Changed<Interaction>>,
    game_assets: Res<GameAssets>,
) {
    for (interaction, mut button) in &mut interaction {
        if matches!(interaction, Interaction::Clicked) {
            if matches!(button.kind, ButtonType::Play) {
                button.is_enabled = !button.is_enabled;
                println!("Spawning: {:?}", button);
            }
        }
    }
}
