use std::{mem::swap, time::Duration};

use bevy::prelude::*;

const RADIUS: f32 = 1.45;
const VELOCITY: f32 = 2.;

struct AnimFrom(pub Quat);
struct AnimTo(pub Quat);
struct AnimTimer(pub Timer);

#[derive(Clone, Copy, Default)]
struct LatLong {
    pub lat: f32,
    pub long: f32,
}

fn latlong_to_quat(l: LatLong) -> Quat {
    Quat::from_axis_angle(Vec3::X, l.lat) * Quat::from_axis_angle(Vec3::Y, l.long)
}

fn update_latlong_system(
    t: Res<Time>,
    mut q: Query<(&mut Transform, &mut AnimFrom, &mut AnimTo, &mut AnimTimer)>,
) {
    let delta = t.delta();
    for (mut current, mut from, mut to, mut t) in q.iter_mut() {
        t.0.tick(delta);

        if t.0.finished() {
            t.0.reset();
            swap(&mut from.0, &mut to.0);
        }

        let t = t.0.percent();

        current.rotation = from.0.slerp(to.0, t);
    }
}

fn setup_planes_system(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    const PATHS: &[[LatLong; 2]] = &[
        [
            LatLong {
                lat: -0.2,
                long: 1.3,
            },
            LatLong {
                lat: 3.141592 / 4.,
                long: -3.141592 / 3.,
            },
        ],
        [
            LatLong {
                lat: 0.,
                long: -0.2,
            },
            LatLong {
                lat: 3.14 / 5.,
                long: -3.14 / 8.,
            },
        ],
    ];

    const PLANE_RADIUS: f32 = RADIUS + 0.2;

    for path in PATHS {
        let [from, to] = *path;
        // spawn our 'plane'
        {
            let mut from = latlong_to_quat(from);
            let to = latlong_to_quat(to);
            let d = from.dot(to);
            if d < 0. {
                // choose the shortest path for interpolation
                // Quat doesn't have mul (f32) operator...
                let [x, y, z, w] = from.as_mut();
                *x *= -1.;
                *y *= -1.;
                *z *= -1.;
                *w *= -1.;
            }
            let t = from.angle_between(to) * PLANE_RADIUS / VELOCITY;
            cmd.spawn_bundle((
                AnimFrom(from),
                AnimTo(to),
                AnimTimer(Timer::new(Duration::from_secs_f32(t), true)),
                Transform::default(),
                GlobalTransform::default(),
            ))
            .with_children(|chld| {
                chld.spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 0.12 })),
                    material: materials.add(StandardMaterial {
                        base_color: Color::YELLOW,
                        metallic: 0.3,
                        unlit: false,
                        ..Default::default()
                    }),
                    transform: Transform::from_translation(Vec3::Z * (PLANE_RADIUS)),
                    ..Default::default()
                });
            });
        }

        // destinations
        for point in [from, to] {
            cmd.spawn_bundle((
                Transform::from_rotation(latlong_to_quat(point)),
                GlobalTransform::default(),
            ))
            .with_children(|chld| {
                chld.spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Icosphere {
                        radius: 0.05,
                        subdivisions: 16,
                    })),
                    material: materials.add(StandardMaterial {
                        base_color: Color::RED,
                        metallic: 0.3,
                        unlit: false,
                        ..Default::default()
                    }),
                    transform: Transform::from_translation(Vec3::Z * (PLANE_RADIUS)),
                    ..Default::default()
                });
            });
        }
    }
}

fn setup_system(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // globe
    cmd.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Icosphere {
            radius: RADIUS,
            subdivisions: 64,
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::BLUE,
            metallic: 0.8,
            unlit: false,
            ..Default::default()
        }),
        transform: Transform::from_translation(Vec3::ZERO),
        ..Default::default()
    });
    // light
    cmd.spawn_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, 5.0)),
        ..Default::default()
    });
    // camera
    cmd.spawn_bundle(OrthographicCameraBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 2.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        orthographic_projection: bevy::render::camera::OrthographicProjection {
            scale: 0.005,
            ..Default::default()
        },
        ..OrthographicCameraBundle::new_3d()
    });
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_system.system())
        .add_startup_system(setup_planes_system.system())
        .add_system(update_latlong_system.system())
        .run();
}
