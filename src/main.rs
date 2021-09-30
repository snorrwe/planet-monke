use std::{mem::swap, time::Duration};

use bevy::prelude::*;

const RADIUS: f32 = 1.45;

struct FromLatLong(pub LatLong);
struct ToLatLong(pub LatLong);
struct CurrentLatLong(pub LatLong);
struct AnimTimer(pub Timer);

#[derive(Clone, Copy, Default)]
struct LatLong {
    pub lat: f32,
    pub long: f32,
}

fn latlong_to_cartesian(l: LatLong, r: f32) -> Vec3 {
    let (latsin, latcos) = l.lat.sin_cos();
    let (longsin, longcos) = l.long.sin_cos();

    let x = r * latcos * longcos;
    let y = r * latcos * longsin;
    let z = r * latsin;
    return Vec3::new(x, y, z);
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn update_latlong_system(
    t: Res<Time>,
    mut q: Query<(
        &mut CurrentLatLong,
        &mut FromLatLong,
        &mut ToLatLong,
        &mut AnimTimer,
    )>,
) {
    let delta = t.delta();
    for (mut current, mut from, mut to, mut t) in q.iter_mut() {
        t.0.tick(delta);

        if t.0.finished() {
            t.0.reset();
            swap(&mut from.0, &mut to.0);
        }

        let t = t.0.percent();

        current.0.lat = lerp(from.0.lat, to.0.lat, t);
        current.0.long = lerp(from.0.long, to.0.long, t);
    }
}

fn update_transforms_system(mut q: Query<(&mut Transform, &CurrentLatLong)>) {
    let r = RADIUS + 0.2;

    for (mut tr, curr) in q.iter_mut() {
        tr.translation = latlong_to_cartesian(curr.0, r);
    }
}

fn setup_planes_system(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    cmd.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.2 })),
        material: materials.add(StandardMaterial {
            base_color: Color::YELLOW,
            metallic: 0.3,
            unlit: false,
            ..Default::default()
        }),
        ..Default::default()
    })
    .insert_bundle((
        FromLatLong(LatLong { lat: 0., long: 0. }),
        ToLatLong(LatLong {
            lat: 3.141592,
            long: 3.141592 / 2.,
        }),
        CurrentLatLong(LatLong::default()),
        AnimTimer(Timer::new(Duration::from_secs(5), true)),
    ));
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
            subdivisions: 32,
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::BLUE,
            metallic: 0.3,
            unlit: false,
            ..Default::default()
        }),
        transform: Transform::from_translation(Vec3::ZERO),
        ..Default::default()
    });
    // camera
    cmd.spawn_bundle(OrthographicCameraBundle {
        transform: Transform::from_translation(Vec3::new(2.0, 0.0, 2.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        orthographic_projection: bevy::render::camera::OrthographicProjection {
            scale: 0.01,
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
        .add_system(update_transforms_system.system())
        .run();
}
