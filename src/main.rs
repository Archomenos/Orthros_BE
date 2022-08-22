//! Load a cubemap texture onto a cube like a skybox and cycle through different compressed texture formats
mod environment;
mod player_controller;
mod skybox;
mod ownable;

use bevy::prelude::*;
//use bevy::render::render_resource::Texture;
use crate::environment::Environment;
use crate::player_controller::PlayerController;
use crate::ownable::{Selectable, SelectionCircle, Selected};
use bevy_rapier3d::geometry::Collider;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(Environment)
        .add_plugin(PlayerController)
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let texture_handle = asset_server.load("textures/selection_texture.png");
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    let parent_id = commands
        .spawn()
        .insert(Transform::from_scale(Vec3::splat(0.2)))
        .insert_bundle(SceneBundle {
            scene: asset_server.load("3d_models/blade_starship/scene.gltf#Scene0"),
            ..default()
        })
        
        .insert(Selectable{})
        .insert(Collider::capsule_z(1.0, 1.5))
        .insert(RigidBody::Dynamic)
        .insert(GravityScale(0.0))
        .id();
    let child_id = commands
        .spawn_bundle(MaterialMeshBundle {
            mesh: meshes.add(shape::Plane { size: 5. }.into()),
            material: material_handle,
            transform: Transform::from_scale(Vec3::splat(1.0)),
            visibility: Visibility{is_visible : false},
            ..default()
        })
        .insert(SelectionCircle {})
        .id();
    commands.entity(parent_id).push_children(&[child_id]);
}

// fn disable_visual_select(material : &mut Handle<StandardMaterial>){
//     material.
// }
// fn deselect_test(
//     mut units: Query<Entity, With<(Selectable)>>,
//     mut selection_circles: Query<(&Parent, &mut Visibility), With<(SelectionCircle)>>
// ){
//
//     for (parent, mut circle_visibility) in selection_circles.iter_mut(){
//         let unit = units.get(parent.get());
//         if circle_visibility.is_visible{
//             circle_visibility.is_visible = false;
//         }else{
//             circle_visibility.is_visible = true;
//         }
//     }
// }
