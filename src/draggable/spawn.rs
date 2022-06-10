use bevy				:: prelude :: { * };
use bevy_mod_picking	:: { * };

use bevy::render::mesh::shape as render_shape;

use super				:: { * };
use crate::game			:: { * };

pub fn root_handle(
	transform			: &Transform,
	sargs				: &mut SpawnArguments,
) -> Entity {
	sargs.commands.spawn_bundle(
	PbrBundle {
		// mesh			: sargs.meshes.add		(Mesh::from(render_shape::Box::new(0.4, 0.3, 0.4))),
		mesh			: sargs.meshes.add		(Mesh::from(render_shape::UVSphere{ radius: 0.225, ..default() })),
		material		: sargs.materials.add(
		StandardMaterial {
			base_color	: Color::LIME_GREEN.into(),
			unlit		: true,
			..default()
		}),
		transform		: *transform,
		..Default::default()
	})
	.insert				(RootHandle)
	.insert				(Gizmo)
	.insert_bundle		(PickableBundle::default())
	.insert				(Draggable::default())
	.id					()
}  