// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::f32::consts::PI;

use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
    window::{PrimaryWindow, WindowResized},
};
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

const NUM_BALLS: u32 = 100;
// const POS_MIN: f32 = -50.;
const POS_MAX: f32 = 5.;
const RADIUS: f32 = 5.;
const VEL_MIN: f32 = -1000.;
const VEL_MAX: f32 = 1000.;
const RESTITUTION: f32 = 1.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .init_resource::<MyWorldCoords>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_cursor,
                on_resize_system,
                update_balls.after(update_cursor),
            ),
        )
        .run();
}

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Ball;

fn setup(
    mut commands: Commands,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2dBundle::default(), MainCamera));

    let window = q_window.single();

    // setup walls
    commands
        .spawn(Collider::cuboid(10.0, 1000.))
        .insert(Ccd::enabled())
        .insert(TransformBundle::from(Transform::from_xyz(
            -window.width() / 2.,
            0.,
            0.,
        )))
        .insert(CollisionGroups::new(Group::GROUP_1, Group::GROUP_2));

    commands
        .spawn(Collider::cuboid(10.0, 1000.))
        .insert(Ccd::enabled())
        .insert(TransformBundle::from(Transform::from_xyz(
            window.width() / 2.,
            0.,
            0.,
        )))
        .insert(CollisionGroups::new(Group::GROUP_1, Group::GROUP_2));

    commands
        .spawn(Collider::cuboid(2000., 10.))
        .insert(Ccd::enabled())
        .insert(TransformBundle::from(Transform::from_xyz(
            0.,
            -window.height() / 2.,
            0.,
        )))
        .insert(CollisionGroups::new(Group::GROUP_1, Group::GROUP_2));

    commands
        .spawn(Collider::cuboid(2000., 10.))
        .insert(Ccd::enabled())
        .insert(TransformBundle::from(Transform::from_xyz(
            0.,
            window.height() / 2.,
            0.,
        )))
        .insert(CollisionGroups::new(Group::GROUP_1, Group::GROUP_2));

    for _ in 0..NUM_BALLS {
        let position = Vec3::new(0., 0., 0.);

        let velocity = Vec2::new(0., 0.);

        commands
            .spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(RADIUS).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::GREEN)),
                    transform: Transform::from_translation(position),
                    ..default()
                },
                Ball,
            ))
            .insert(RigidBody::Dynamic)
            .insert(Collider::ball(RADIUS))
            .insert(GravityScale(5.0))
            .insert(Velocity::linear(velocity))
            .insert(Restitution::coefficient(RESTITUTION))
            .insert(CollisionGroups::new(Group::GROUP_2, Group::GROUP_1));
    }
}

fn update_balls(
    cursor: Res<MyWorldCoords>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut balls: Query<(&mut Transform, &mut Velocity), With<Ball>>,
) {
    if mouse_buttons.pressed(MouseButton::Left) {
        let mut rng = rand::thread_rng();

        for (mut transform, mut velocity) in &mut balls {
            let new_position = Vec3::new(cursor.0.x, cursor.0.y, 0.)
                + Vec2::from_angle(rng.gen_range(0.0..2. * PI)).extend(0.0)
                    * rng.gen_range(0.0..POS_MAX);

            let new_velocity = Vec2::new(
                rng.gen_range(VEL_MIN..VEL_MAX),
                rng.gen_range(VEL_MIN..VEL_MAX),
            );

            transform.translation = new_position;

            velocity.linvel = new_velocity;


        }
    }
}

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
struct MyWorldCoords(Vec2);

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

fn update_cursor(
    mut mycoords: ResMut<MyWorldCoords>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        mycoords.0 = world_position;
    }
}

fn on_resize_system(mut resize_reader: EventReader<WindowResized>) {
    for e in resize_reader.read() {
        // When resolution is being changed
        println!("{:.1} x {:.1}", e.width, e.height);
    }
}
