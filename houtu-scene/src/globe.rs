use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::box3d::Box3d;
use crate::ellipsoid::{Ellipsoid, EllipsoidShape};
use crate::oriented_bounding_box;
#[derive(Component)]
pub struct Shape;
pub struct GlobePlugin {}
impl Default for GlobePlugin {
    fn default() -> Self {
        Self {}
    }
}
impl bevy::app::Plugin for GlobePlugin {
    fn build(&self, app: &mut App) {
        // app.add_system_set(systems::system_set());
        app.add_startup_system(setup);
    }
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });
    let ellipsoid = Ellipsoid::WGS84;
    let x = ellipsoid.semimajor_axis() as f32;
    let y = ellipsoid.semiminor_axis() as f32;
    let z = ellipsoid.semiminor_axis() as f32;
    let mesh: Mesh = EllipsoidShape::from_ellipsoid(ellipsoid).into();

    let points = houtu_utils::getPointsFromMesh(&mesh);
    let obb = oriented_bounding_box::OrientedBoundingBox::fromPoints(points.as_slice());

    commands.spawn(PbrBundle {
        mesh: meshes.add(Box3d::frmo_obb(obb).into()),
        material: materials.add(Color::WHITE.into()),
        ..Default::default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            // material:  materials.add(Color::SILVER.into()),
            material: debug_material.into(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Shape,
    ));
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(x + 1000., x + 1000., x + 1000.),
        ..default()
    });
}
/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}
