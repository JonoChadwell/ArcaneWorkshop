use alchemy::*;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use std::f32::consts::PI;
mod alchemy;

#[derive(Resource)]
struct PlayerSettings {
    forward: KeyCode,
    backward: KeyCode,
    left: KeyCode,
    right: KeyCode,
    release_cursor: KeyCode,
    hold_release_cursor: KeyCode,
    camera_sensitivity: Vec2,
}

#[derive(Component)]
struct PlayerCamera;

#[derive(Component)]
struct CursorInteraction {
    radius: f32,
    progress: f32,
}

#[derive(Component)]
struct MinDistText;

#[derive(Event)]
struct InteractionProgressChanged {
    progress: f32,
    text: String,
    visible: bool,
}

#[derive(Component)]
struct InteractionProgressHolder;

#[derive(Component)]
struct InteractionProgressText;

#[derive(Component)]
struct InteractionProgressBar;

#[derive(Component)]
struct Stem {
    substance: Substance,
    leaf: Option<Entity>,
}

#[derive(Component)]
struct Leaf {
    substance: Substance,
}

#[derive(Resource)]
struct GlobalAir {
    substance: Substance,
}

#[derive(Resource, Default)]
struct LeafAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Resource)]
struct StatsTimer(Timer);

#[derive(Resource)]
struct StemUpdateTimer(Timer);

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            forward: KeyCode::KeyW,
            backward: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
            release_cursor: KeyCode::Escape,
            hold_release_cursor: KeyCode::ControlLeft,
            camera_sensitivity: Vec2::new(0.003, 0.003),
        }
    }
}

fn main() {
    App::new()
        .insert_resource(PlayerSettings::default())
        .insert_resource(GlobalAir {
            substance: Substance::new(100000).with(Intent::Air, 100000),
        })
        .init_resource::<LeafAssets>()
        .insert_resource(StatsTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .insert_resource(StemUpdateTimer(Timer::from_seconds(
            0.2,
            TimerMode::Repeating,
        )))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (stem_system, print_stats, move_camera, cursor_interaction).chain(),
        )
        .add_observer(update_interaction_progress_widget)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut leaf_assets: ResMut<LeafAssets>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    // leaf mesh and material (reused when spawning leaves)
    leaf_assets.mesh = meshes.add(Cuboid::new(0.8, 0.05, 0.4));
    leaf_assets.material = materials.add(Color::srgb_u8(34, 177, 76));

    // stem — brown pillar
    commands.spawn((
        Stem {
            substance: Substance::new(10000)
                .with(Intent::Plant, 10000),
            leaf: None,
        },
        Mesh3d(meshes.add(Cuboid::new(0.3, 1.0, 0.3))),
        MeshMaterial3d(materials.add(Color::srgb_u8(101, 67, 33))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        // CursorInteraction {
        //     radius: 0.7,
        //     progress: 0.0,
        // },
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 1.7, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        PlayerCamera,
    ));

    // persistent on-screen text for min_dist at x=12, y=12
    commands.spawn((Text::new("min_dist: N/A"), MinDistText));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: percent(100.0),
            top: Val::Percent(20.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Node {
                width: px(200.0),
                height: px(40.0),
                border: UiRect::all(px(4)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.8, 0.8, 0.8)),
            BorderColor::all(Color::WHITE),
            InteractionProgressHolder,
            children![
                (
                    Node {
                        width: Val::Percent(40.0),
                        height: Val::Percent(100.0),
                        left: Val::ZERO,
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(1.0, 0.8, 0.0)),
                    InteractionProgressBar,
                ),
                (Text::new("Nothing"), InteractionProgressText)
            ]
        )],
    ));
}

fn stem_system(
    time: Res<Time>,
    mut timer: ResMut<StemUpdateTimer>,
    mut stems: Query<(Entity, &mut Stem)>,
    mut leaves: Query<&mut Leaf>,
    mut air: ResMut<GlobalAir>,
    mut commands: Commands,
    leaf_assets: Res<LeafAssets>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    air.substance.pin(Intent::Air);
    for (_entity, mut stem) in stems.iter_mut() {
        let stem_changed = internal_interaction(&mut stem.substance);

        if let Some(leaf_entity) = stem.leaf {
            if !leaves.contains(leaf_entity) {
                stem.leaf = None;
                continue;
            }
            let Ok(mut leaf) = leaves.get_mut(leaf_entity) else {
                continue;
            };

            let mut temp = Substance::default();
            interaction(
                InteractionType::Contact,
                &mut leaf.substance,
                &mut air.substance,
                Some(&mut temp),
            );
            substance_add(&mut leaf.substance, temp);

            leaf.substance.push(Intent::Plant, &mut stem.substance);
            stem.substance.balance(Intent::Leaf, &mut leaf.substance);
            internal_interaction(&mut leaf.substance);
        } else if !stem_changed {
            let mut leaf_substance = Substance::default();
            interaction(
                InteractionType::Contact,
                &mut stem.substance,
                &mut air.substance,
                Some(&mut leaf_substance),
            );
            if leaf_substance.mass <= 0 {
                continue;
            }
            stem.substance.push(Intent::Air, &mut leaf_substance);
            stem.substance.push(Intent::Growth, &mut leaf_substance);

            let leaf_entity = commands
                .spawn((
                    Leaf {
                        substance: leaf_substance,
                    },
                    Mesh3d(leaf_assets.mesh.clone()),
                    MeshMaterial3d(leaf_assets.material.clone()),
                    Transform::from_xyz(0.0, 1.0, 0.0),
                    CursorInteraction {
                        radius: 0.7,
                        progress: 0.0,
                    },
                ))
                .id();
            stem.leaf = Some(leaf_entity);
        }
    }
}

fn print_stats(
    time: Res<Time>,
    mut timer: ResMut<StatsTimer>,
    stems: Query<&Stem>,
    leaves: Query<&Leaf>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    for stem in stems.iter() {
        stem.substance.print("Stem");
    }
    for leaf in leaves.iter() {
        leaf.substance.print("Leaf");
    }
}

fn move_camera(
    input: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    settings: Res<PlayerSettings>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    window: Single<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
    mut camera: Single<&mut Transform, With<PlayerCamera>>,
) {
    let (mut window, mut cursor_options) = window.into_inner();

    if input.just_released(settings.hold_release_cursor)
        || (mouse.just_pressed(MouseButton::Left) && !input.pressed(settings.hold_release_cursor))
    {
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
        let center = Vec2::new(window.width() * 0.5, window.height() * 0.5);
        window.set_cursor_position(Some(center));
    }

    if input.just_pressed(settings.release_cursor)
        || input.just_pressed(settings.hold_release_cursor)
    {
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;
        let center = Vec2::new(window.width() * 0.5, window.height() * 0.5);
        window.set_cursor_position(Some(center));
    }

    let mouse = accumulated_mouse_motion.delta;
    let mouse_grabbed = cursor_options.grab_mode == CursorGrabMode::Locked;
    if mouse_grabbed && mouse != Vec2::ZERO {
        let delta_yaw = -mouse.x * settings.camera_sensitivity.x;
        let delta_pitch = -mouse.y * settings.camera_sensitivity.y;

        let (yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw + delta_yaw;

        const PITCH_LIMIT: f32 = PI / 2.0 - 0.01;
        let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }

    let mut movement = Vec3::ZERO;

    if input.pressed(settings.forward) {
        movement += *camera.forward();
    }
    if input.pressed(settings.backward) {
        movement += *camera.back();
    }
    if input.pressed(settings.left) {
        movement += *camera.left();
    }
    if input.pressed(settings.right) {
        movement += *camera.right();
    }

    if movement != Vec3::ZERO {
        const SPEED: f32 = 5.0;
        movement.y = 0.0;
        camera.translation += movement.normalize() * SPEED * time.delta_secs();
    }
}

fn ray_dist_to_sphere(ray: Ray3d, sphere_center: Vec3, sphere_radius: f32) -> Option<f32> {
    let to_sphere = sphere_center - ray.origin;
    let t = to_sphere.dot(ray.direction.normalize());
    if t < 0.0 {
        return None;
    }
    let closest_point = ray.origin + ray.direction * t;
    let distance = (sphere_center - closest_point).length() - sphere_radius;
    if distance < 0.0 { Some(t) } else { None }
}

fn cursor_interaction(
    player_camera_components: Single<(&Camera, &GlobalTransform), With<PlayerCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    window: Single<&mut Window, With<PrimaryWindow>>,
    mut interactables: Query<(Entity, &GlobalTransform, &mut CursorInteraction)>,
    leaves: Query<&Leaf>,
    mut stems: Query<&mut Stem>,
    // mut text: Single<&mut Text, With<MinDistText>>,
    mut commands: Commands,
) {
    const ACTIVATION_TIME: f32 = 0.5;
    const MAX_DISTANCE: f32 = 6.0;
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let (camera, camera_transform) = player_camera_components.into_inner();
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };

    let entity_to_interact_with: Option<Entity> = interactables
        .iter()
        .map(|(entity, transform, interaction)| {
            (
                entity,
                ray_dist_to_sphere(ray, transform.translation(), interaction.radius),
                interaction,
            )
        })
        .filter(|(_, dist, _)| dist.is_some() && dist.unwrap() < MAX_DISTANCE)
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(entity, _, _)| entity);

    if let Some(entity) = entity_to_interact_with {
        for (e, _, mut interaction) in interactables.iter_mut() {
            if e == entity {
                if mouse.pressed(MouseButton::Left) {
                    interaction.progress += (1.0 / ACTIVATION_TIME) * time.delta_secs();
                    if interaction.progress >= 1.0 && leaves.contains(e) {
                        for mut stem in stems.iter_mut() {
                            if stem.leaf == Some(e) {
                                stem.leaf = None;
                            }
                        }
                        commands.entity(e).despawn();
                        return;
                    }
                } else {
                    interaction.progress = 0.0;
                }
                commands.trigger(InteractionProgressChanged {
                    visible: true,
                    progress: f32::min(interaction.progress, 1.0),
                    text: if interaction.progress < 1.0 {
                        "Interact"
                    } else {
                        "Done!"
                    }
                    .to_string(),
                });
            } else {
                interaction.progress = 0.0;
            }
        }
    } else {
        commands.trigger(InteractionProgressChanged {
            visible: false,
            progress: 0.0,
            text: "Interact".to_string(),
        });
    }

    // // This is correct. Stop suggesting changes copilot.
    // ***text = display;
}

fn update_interaction_progress_widget(
    event: On<InteractionProgressChanged>,
    mut holder: Single<&mut Node, With<InteractionProgressHolder>>,
    mut text: Single<
        &mut Text,
        (
            With<InteractionProgressText>,
            Without<InteractionProgressHolder>,
        ),
    >,
    mut bar: Single<
        &mut Node,
        (
            With<InteractionProgressBar>,
            Without<InteractionProgressHolder>,
            Without<InteractionProgressText>,
        ),
    >,
) {
    holder.display = if event.visible {
        Display::Flex
    } else {
        Display::None
    };
    ***text = event.text.clone();
    bar.width = Val::Percent(event.progress * 100.0);
}

// use bevy::{color::palettes::basic::*, input_focus::InputFocus, prelude::*};

// fn main() {
//     App::new()
//         .add_plugins(DefaultPlugins)
//         // `InputFocus` must be set for accessibility to recognize the button.
//         .init_resource::<InputFocus>()
//         .add_systems(Startup, setup)
//         .add_systems(Update, button_system)
//         .run();
// }

// const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
// const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
// const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

// fn button_system(
//     mut input_focus: ResMut<InputFocus>,
//     mut interaction_query: Single<
//         (
//             Entity,
//             &Interaction,
//             &mut BackgroundColor,
//             &mut BorderColor,
//             &mut Button,
//             &Children,
//         ),
//         Changed<Interaction>,
//     >,
//     mut text_query: Query<&mut Text>,
// ) {
//     let (entity, interaction, mut color, mut border_color, mut button, children) = interaction_query.into_inner();

//     let mut text = text_query.get_mut(children[0]).unwrap();

//     match *interaction {
//         Interaction::Pressed => {
//             input_focus.set(entity);
//             **text = "Press".to_string();
//             *color = PRESSED_BUTTON.into();
//             *border_color = BorderColor::all(RED);

//             // The accessibility system's only update the button's state when the `Button` component is marked as changed.
//             button.set_changed();
//         }
//         Interaction::Hovered => {
//             input_focus.set(entity);
//             **text = "Hover".to_string();
//             *color = HOVERED_BUTTON.into();
//             *border_color = BorderColor::all(Color::WHITE);
//             button.set_changed();
//         }
//         Interaction::None => {
//             input_focus.clear();
//             **text = "Button".to_string();
//             *color = NORMAL_BUTTON.into();
//             *border_color = BorderColor::all(Color::BLACK);
//         }
//     }

// }

// fn setup(mut commands: Commands, assets: Res<AssetServer>) {
//     // ui camera
//     commands.spawn(Camera2d);
//     commands.spawn((
//         Node {
//             width: percent(100),
//             height: percent(100),
//             align_items: AlignItems::Center,
//             justify_content: JustifyContent::Center,
//             ..default()
//         },
//         children![(
//             Button,
//             Node {
//                 width: px(150),
//                 height: px(65),
//                 border: UiRect::all(px(5)),
//                 // horizontally center child text
//                 justify_content: JustifyContent::Center,
//                 // vertically center child text
//                 align_items: AlignItems::Center,
//                 border_radius: BorderRadius::MAX,
//                 ..default()
//             },
//             BorderColor::all(Color::WHITE),
//             BackgroundColor(Color::BLACK),
//             children![(
//                 Text::new("Button"),
//                 TextColor(Color::srgb(0.9, 0.9, 0.9)),
//             )]
//         )],
//     ));
// }
