use bevy::input::keyboard::Key;
use bevy::math::EulerRot::XYZ;
use bevy::prelude::*;
use bevy_mod_billboard::prelude::BillboardPlugin;
use bevy_mod_billboard::{BillboardLockAxis, BillboardTextBundle};
use bevy_openxr::add_xr_plugins;
use bevy_openxr::init::OxrInitPlugin;
use bevy_openxr::types::OxrExtensions;
use bevy_xr::hands::{HandBone, LeftHand, RightHand};
use bevy_xr_utils::xr_utils_actions::{
    ActiveSet, XRUtilsAction, XRUtilsActionSet, XRUtilsActionState, XRUtilsActionSystemSet,
    XRUtilsActionsPlugin, XRUtilsBinding,
};
use std::f32::consts::PI;
use std::ops::Add;

#[bevy_main]
pub fn main() {
    let mut app = App::new();
    app.add_plugins(add_xr_plugins(DefaultPlugins).set(OxrInitPlugin {
        app_info: default(),
        exts: {
            let mut exts = OxrExtensions::default();
            //exts.enable_fb_passthrough();
            exts.enable_hand_tracking();
            //exts.enable_custom_refresh_rates();
            exts
        },
        blend_modes: default(),
        backends: default(),
        formats: default(),
        resolutions: default(),
        synchronous_pipeline_compilation: default(),
    }))
    .add_plugins((
        bevy_embedded_assets::EmbeddedAssetPlugin::default(),
        bevy_xr_utils::hand_gizmos::HandGizmosPlugin,
        BillboardPlugin,
    ))
    // Setup
    .add_systems(Startup, setup)
    .add_systems(Update, set_parent)
    // Realtime lighting is expensive, use ambient light instead
    .insert_resource(AmbientLight {
        color: Default::default(),
        brightness: 500.0,
    })
    .insert_resource(Msaa::Off)
    /*.insert_resource(ClearColor(Color::NONE))*/
    .add_systems(Update, do_keyboard_tracking)
    .run();
}

#[derive(Component)]
pub struct Keyboard;

#[derive(Component)]
pub struct Output;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Spawns a plane
    // For improved performance set the `unlit` value to true on standard materials
    /*commands.spawn((PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(1.0, 1.0)),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
        transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
        ..default()
    },));*/

    let fira_sans_regular_handle = asset_server.load("embedded://FiraSans-Regular.ttf");
    commands.spawn((
        BillboardTextBundle {
            transform: Transform::from_scale(Vec3::splat(0.01))
                .with_rotation(Quat::from_euler(XYZ, 0.0, PI / 2.0, 0.0)),
            text: Text::from_sections([TextSection {
                value: "text".to_string(),
                style: TextStyle {
                    font_size: 60.0,
                    font: fira_sans_regular_handle.clone(),
                    color: Color::WHITE,
                },
            }])
            .with_justify(JustifyText::Center),
            ..default()
        },
        BillboardLockAxis {
            rotation: true,
            ..default()
        },
        Keyboard,
    ));

    commands.spawn((
        BillboardTextBundle {
            transform: Transform::from_scale(Vec3::splat(0.01))
                .with_translation(Vec3::new(0.0, 0.5, 0.0))
                .with_rotation(Quat::from_euler(XYZ, 0.0, PI / 2.0, 0.0)),
            text: Text::from_sections([TextSection {
                value: "".to_string(),
                style: TextStyle {
                    font_size: 60.0,
                    font: fira_sans_regular_handle.clone(),
                    color: Color::WHITE,
                },
            }])
            .with_justify(JustifyText::Center),
            ..default()
        },
        BillboardLockAxis {
            rotation: true,
            ..default()
        },
        Output,
    ));
}

fn set_parent(
    mut commands: Commands,
    right_hand: Query<(Entity, &HandBone), With<RightHand>>,
    left_hand: Query<(Entity, &HandBone), With<LeftHand>>,
    mut keyboard: Query<(Entity, &mut Transform), With<Keyboard>>,
    mut output: Query<(Entity, &mut Transform), (With<Output>, Without<Keyboard>)>,
    mut stop: Local<bool>,
) {
    if *stop {
        return;
    }

    let new_rotation = Quat::from_euler(XYZ, 90.0_f32.to_radians(), 180.0_f32.to_radians(), -180.0_f32.to_radians());

    for (right_bone_entity, right_bone) in right_hand.iter() {
        if !matches!(right_bone, HandBone::Palm) {
            continue;
        }
        for (k, mut t) in keyboard.iter_mut() {
            commands.entity(k).set_parent(right_bone_entity);
            *stop = true;
            t.translation.y = -0.02;
            t.rotation = new_rotation.clone();
            t.scale = Vec3::splat(0.002);
        }
    }
    for (left_bone_entity, right_bone) in left_hand.iter() {
        if !matches!(right_bone, HandBone::Palm) {
            continue;
        }
        for (k, mut t) in output.iter_mut() {
            commands.entity(k).set_parent(left_bone_entity);
            *stop = true;
            t.translation.y = -0.02;
            t.rotation = new_rotation;
            t.scale = Vec3::splat(0.001);
        }
    }
}

fn acceleration_curve(delta: f32) -> f32 {
    let numerator = 1.5;
    let exponent = -1.1 * delta;
    let denominator = 1.0 + f32::exp(exponent);
    let vertical_shift = -0.75;

    (numerator / denominator) + vertical_shift
}

fn do_keyboard_tracking(
    mut keyboard: Query<(&mut Text, &GlobalTransform), With<Keyboard>>,
    mut output: Query<&mut Text, (With<Output>, Without<Keyboard>)>,
    right_hand: Query<(&HandBone, &GlobalTransform), With<RightHand>>,
    mut pinch_state: Local<bool>,
    mut last: Local<f32>,
    mut extra: Local<f32>,
) {
    let mut char = 'a';

    for (mut text_section, transform) in keyboard.iter_mut() {
        for (bone, pose) in right_hand.iter() {
            if !matches!(bone, HandBone::Palm) {
                continue;
            }
        }
        let current_pos = transform.compute_transform();
        let mut chars = vec![
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
            'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4', '5', '6', '7',
            '8', '9',
        ];

        let length = chars.len();

        let delta = current_pos.translation.length() - *last;

        println!("amount: {}", delta);

        let readjustment = 1000.0;


        println!("readjusted: {}", delta * readjustment);

        *extra += (acceleration_curve(delta * readjustment * 2.0) / readjustment) * 4.0;

        println!("acc curve: {}", acceleration_curve(delta * readjustment) / readjustment);

        let mag = (*extra /*+ (current_pos.translation.length() / 6.0)*/);

        /*let mut scaled = mag.rem_euclid(1.0);*/


        let val = (mag * 1.5) as usize % length;

        /*let val = (scaled * length as f32) as usize;*/
        char = *chars.get(val).unwrap();
        text_section.sections.first_mut().unwrap().value = String::from(char);
        *last = current_pos.translation.length();
    }

    let mut pinch_activated = false;

    for (right_bone, index_pos) in right_hand.iter() {
        if !matches!(right_bone, HandBone::MiddleTip) {
            continue;
        }
        for (right_bone, thumb_pos) in right_hand.iter() {
            if !matches!(right_bone, HandBone::ThumbTip) {
                continue;
            }
            let pinched = index_pos.translation().distance(thumb_pos.translation()) < 0.02;
            let unpinched = index_pos.translation().distance(thumb_pos.translation()) > 0.05;
            if pinched && !*pinch_state {
                pinch_activated = true;
            }
            if pinched && !unpinched {
                *pinch_state = true;
            }
            if !pinched {
                *pinch_state = false;
            }
        }
    }

    if pinch_activated {
        let mut output = output.single_mut();
        output.sections.first_mut().unwrap().value.push(char);
    }
}
