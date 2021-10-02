mod countries;

use std::{mem::swap, time::Duration};

use bevy::{prelude::*, tasks::ComputeTaskPool};
use countries::PATHS;

const RADIUS: f32 = 1.45;
const VELOCITY: f32 = 1.;
const ORBIT_RADIUS: f32 = RADIUS + 0.2;

#[derive(Debug, Clone, Copy)]
struct PlaneId(pub usize);
struct AnimFrom(pub Quat);
struct AnimTo(pub Quat);
struct AnimTimer(pub Timer);

struct ChildOrientation(pub Quat);

#[derive(Clone, Copy, Default)]
pub struct LatLong {
    pub lat: f32,
    pub long: f32,
}

fn latlong_to_quat(l: LatLong) -> Quat {
    Quat::from_axis_angle(Vec3::X, l.lat) * Quat::from_axis_angle(Vec3::Y, l.long)
}

fn proj_vec_onto_plane(u: Vec3, n: Vec3) -> Vec3 {
    debug_assert!(
        (n.length() - 1.0).abs() < 1e-6,
        "Plane normal must be normalized. {:?}",
        n
    );
    u - (u.dot(n) * n)
}

fn update_plane_orient_system(
    mut q: Query<(&mut ChildOrientation, &GlobalTransform, &AnimFrom, &AnimTo)>,
    pool: Res<ComputeTaskPool>,
) {
    // TODO:
    // this actually this doesn't have to run every update, just when the plane changes
    // destination...
    q.par_for_each_mut(&*pool, 8, |(mut orient, tr, from, to)| {
        let from_point = from.0 * Vec3::Z;
        let to_point = to.0 * Vec3::Z;

        let plane_normal = (tr.rotation * Vec3::Z).normalize();

        let delta = to_point - from_point;
        let proj_d = proj_vec_onto_plane(delta, plane_normal).normalize();

        let fw = tr.rotation * Vec3::Y;
        let proj_fw = proj_vec_onto_plane(fw, plane_normal).normalize();

        let d = proj_d.dot(proj_fw);

        let ang = d.acos();
        let mut axis = Vec3::Z;

        let triple = delta.cross(fw).dot(plane_normal);

        // absolute what the fuckity fuck
        //
        // orientation is _sometimes_ fucked up, depending if the sign of the triple prod and the
        // dot prod are the same or not...
        if triple * d < 0.0 {
            if d < 0.0 {
                axis *= -1.0;
            }
        } else if d > 0.0 {
            axis *= -1.0;
        }

        orient.0 = Quat::from_axis_angle(axis, ang);
    });
}

fn update_child_orients_system(
    q: Query<(&Children, &ChildOrientation)>,
    mut qc: Query<&mut Transform>,
) {
    for (children, orient) in q.iter() {
        for child in children.iter() {
            let mut tr = qc.get_mut(*child).unwrap();
            tr.rotation = orient.0;
        }
    }
}

fn update_latlong_system(
    t: Res<Time>,
    mut q: Query<(&mut Transform, &mut AnimFrom, &mut AnimTo, &mut AnimTimer)>,
    pool: Res<ComputeTaskPool>,
) {
    let delta = t.delta();
    q.par_for_each_mut(&*pool, 8, |(mut current, mut from, mut to, mut t)| {
        t.0.tick(delta);

        if t.0.finished() {
            t.0.reset();
            swap(&mut from.0, &mut to.0);
        }

        let t = t.0.percent();

        current.rotation = from.0.slerp(to.0, t);
    });
}

fn setup_planes_system(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let plane_handle = asset_server.load("monke.gltf#Mesh0/Primitive0");

    for (i, path) in PATHS.iter().enumerate() {
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
            let t = from.angle_between(to) * ORBIT_RADIUS / VELOCITY;
            let plane_handle = plane_handle.clone();
            cmd.spawn_bundle((
                AnimFrom(from),
                AnimTo(to),
                AnimTimer(Timer::new(Duration::from_secs_f32(t), true)),
                Transform::default(),
                GlobalTransform::default(),
                ChildOrientation(Quat::default()),
            ))
            .with_children(|chld| {
                let mut transform = Transform::from_translation(Vec3::Z * ORBIT_RADIUS);
                transform.scale = Vec3::splat(0.1);
                chld.spawn_bundle(PbrBundle {
                    mesh: plane_handle,
                    material: materials.add(StandardMaterial {
                        base_color: Color::YELLOW,
                        ..Default::default()
                    }),
                    transform,
                    ..Default::default()
                })
                .insert(PlaneId(i));
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
                    transform: Transform::from_translation(Vec3::Z * (ORBIT_RADIUS)),
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
            ..Default::default()
        }),
        transform: Transform::from_translation(Vec3::ZERO),
        ..Default::default()
    });
    // light
    cmd.spawn_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, -1.0, 5.0)),
        ..Default::default()
    });
    // camera
    cmd.spawn_bundle(OrthographicCameraBundle {
        transform: Transform::from_translation(Vec3::new(0.0, -2.0, 2.0))
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
        .add_system(update_plane_orient_system.system())
        .add_system(update_child_orients_system.system())
        .run();
}
