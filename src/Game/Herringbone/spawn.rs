use bevy			::	prelude :: { * };
use bevy_rapier3d	::	prelude :: { * };
use bevy_fly_camera	::	FlyCamera;
use bevy_mod_picking::	{ * };
// use bevy_transform_gizmo :: { * };
use splines			::	{ Interpolation, Key, Spline };

use bevy::render::mesh::shape as render_shape;
use std::f32::consts::	{ * };

use super			::	{ * };

pub fn spline_tangent(
	parent_e			: Entity,
	pos					: Vec3,
	handle				: SplineTangent,
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	mut commands		: &mut Commands
) -> Entity {
	let mut id 			= Entity::from_raw(0);
	let transform		= Transform::from_translation(pos);

	commands.entity(parent_e).with_children(|parent| {
	id = parent.spawn_bundle(PbrBundle {
		mesh			: meshes.add		(Mesh::from(render_shape::Box::new(0.3, 0.3, 0.3))),
		material		: materials.add		(Color::INDIGO.into()),
		transform		: transform,
		..Default::default()
	})
	.insert				(handle)
	.insert				(Gizmo)
	.insert_bundle		(PickableBundle::default())
	.insert				(Draggable::default())
	.id();
	});
	id
}

pub fn spline_control_point(
	parent_e			: Entity,
	pos					: Vec3,
	handle				: SplineControlPoint,
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	mut commands		: &mut Commands
) -> Entity {
	let mut id 			= Entity::from_raw(0);
	let transform		= Transform::from_translation(pos);

	commands.entity(parent_e).with_children(|parent| {
	id = parent.spawn_bundle(PbrBundle {
		mesh			: meshes.add		(Mesh::from(render_shape::Box::new(0.4, 0.3, 0.4))),
		material		: materials.add		(Color::BEIGE.into()),
		transform		: transform,
		..Default::default()
	})
	.insert				(handle)
	.insert				(Gizmo)
	.insert_bundle		(PickableBundle::default())
	.insert				(Draggable::default())
	.id();
	});
	id
}

pub fn object_root(
	pos					: Vec3,
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	mut commands		: &mut Commands
) -> Entity {
	let transform		= Transform::from_translation(pos);

	commands.spawn_bundle(PbrBundle {
		mesh			: meshes.add		(Mesh::from(render_shape::Box::new(0.4, 0.3, 0.4))),
		material		: materials.add		(Color::LIME_GREEN.into()),
		transform		: transform,
		..Default::default()
	})
	.insert				(ObjectRoot)
	.insert				(Gizmo)
	.insert_bundle		(PickableBundle::default())
	.insert				(Draggable::default())
	// .insert				(GizmoTransformable)
	.id()
}