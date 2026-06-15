use std::f32::consts::PI;
use bevy::prelude::*;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::window::{PrimaryWindow, CursorGrabMode, CursorOptions};

#[derive(Component)]
struct PlayerCamera;

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
struct CursorInteraction {
    radius: f32,
}

#[derive(Component)]
struct MinDistText;

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
        .add_plugins(DefaultPlugins)
    .add_systems(Startup, setup)
    .add_systems(Update, (move_camera, cursor_interaction))
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 1.5, 0.0),
        CursorInteraction { radius: 0.7 },
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
    commands.spawn((
        Text::new(
            "min_dist: N/A",
        ),
        MinDistText,
    ));
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
        || (mouse.just_pressed(MouseButton::Left)
            && !input.pressed(settings.hold_release_cursor)) {
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
        let center = Vec2::new(
            window.width() * 0.5,
            window.height() * 0.5,
        );
        window.set_cursor_position(Some(center));
    }

    if input.just_pressed(settings.release_cursor)
            || input.just_pressed(settings.hold_release_cursor) {
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;
        let center = Vec2::new(
            window.width() * 0.5,
            window.height() * 0.5,
        );
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

fn ray_dist_to_sphere(ray: Ray3d , sphere_center: Vec3, sphere_radius: f32) -> Option<f32> {
    let to_sphere = sphere_center - ray.origin;
    let t = to_sphere.dot(ray.direction.normalize());
    if t < 0.0 {
        return None;
    }
    let closest_point = ray.origin + ray.direction * t;
    let distance = (sphere_center - closest_point).length() - sphere_radius;
    if distance < 0.0 {
        Some(t)
    } else {
        None
    }
}

fn cursor_interaction(
    player_camera_components: Single<(&Camera, &GlobalTransform), With<PlayerCamera>>,
    window: Single<&mut Window, With<PrimaryWindow>>,
    interactables: Query<(&GlobalTransform, &CursorInteraction)>,
    mut text: Single<&mut Text, With<MinDistText>>,
) {
    const ACTIVATION_TIME: f32 = 2.0;
    const MAX_DISTANCE: f32 = 6.0;
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let (camera, camera_transform) = player_camera_components.into_inner();
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };

    let mut min_dist = f32::INFINITY;
    for (transform, interaction) in interactables.iter() {
        if let Some(dist) = ray_dist_to_sphere(ray, transform.translation(), interaction.radius) {
            if dist < MAX_DISTANCE {
                min_dist = min_dist.min(dist);
            }
        }
    }
    let display = if min_dist.is_finite() {
        format!("min_dist: {:.3}", min_dist)
    } else {
        "min_dist: inf".to_string()
    };
    ***text = display;
}