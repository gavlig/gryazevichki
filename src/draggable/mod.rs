use bevy					:: { prelude :: * };
use bevy					:: { input :: mouse :: * };
use bevy_mod_picking		:: { * };
use bevy_polyline			:: { prelude :: * };
use bevy_debug_text_overlay	:: { screen_print };

use crate           		:: { game :: * };

pub mod spawn;

#[derive(Component)]
pub struct RootHandle;

#[derive(Component, Default)]
pub struct Draggable {
	pub init_transform		: GlobalTransform,
	pub pick2object_offset	: Vec3,
	pub started_picking		: bool,
	pub init_pick_distance 	: f32,
	pub pick_distance		: f32,
}

#[derive(Component)]
pub struct DraggableActive;

#[derive(Component)]
pub struct DraggableRaycast;

pub fn dragging_start_system(
		btn					: Res<Input<MouseButton>>,
		q_draggable_pick	: Query<&PickingObject, With<DraggableRaycast>>,
		q_mouse_pick		: Query<&PickingCamera, With<Camera>>,
	mut interactions 		: Query<
	(
		Entity,
		&Interaction,
		&GlobalTransform,
		&mut Draggable,
	), Without<DraggableActive>
	>,
	mut commands			: Commands
) {
	let just_clicked = btn.just_pressed(MouseButton::Left);
	if !just_clicked {
		return;
	}

	if !q_draggable_pick.is_empty() {
		return;
	}

	let mouse_pick 	= q_mouse_pick.single();
	let top_pick 	= mouse_pick.intersect_top();

	// There is at least one entity under the cursor
	if top_pick.is_none() {
		return;
	}
	
	let (topmost_entity, intersection) = top_pick.unwrap();
	
	if let Ok((clicked_entity, interaction, global_transform, mut drag)) = interactions.get_mut(topmost_entity) {
		if *interaction != Interaction::Hovered {
			return;
		}

		drag.init_transform = global_transform.clone();
		drag.pick2object_offset = drag.init_transform.translation - intersection.position();

		commands.entity(clicked_entity).insert(DraggableActive);

		// rotating a child caster so that raycast points to the ground. It is used to keep draggable on the same-ish offset above the ground
		let mut transform	= Transform::identity();
		transform.look_at	(-Vec3::Y, -Vec3::Z);

		screen_print!("started dragging {:?}", clicked_entity);

		commands.entity(clicked_entity).with_children(|parent| {
			parent.spawn_bundle(TransformBundle{ local : transform, ..default() })
				.insert		(PickingObject::new_transform_empty())
				.insert		(DraggableRaycast);
		});
	}
}

pub fn dragging_system(
	mut polylines			: ResMut<Assets<Polyline>>,

	mut mouse_wheel_events	: EventReader<MouseWheel>,
		q_transform			: Query<&GlobalTransform, Without<DraggableActive>>,
		q_draggable_pick	: Query<&PickingObject, With<DraggableRaycast>>,
		q_mouse_pick		: Query<&PickingObject, With<Camera>>,
	mut draggable			: Query<(Entity, Option<&Parent>, &mut Transform, &mut Draggable), With<DraggableActive>>,
		q_gizmo				: Query<Entity, With<Gizmo>>, // TODO: move this outside into a lambda or something
		q_tile				: Query<Entity, With<Tile>>,
) {
	if draggable.is_empty() {
		return;
	}

	let mouse_pick 			= q_mouse_pick.single();
	let mouse_ray 			= mouse_pick.ray().unwrap();

	let draggable_pick 		= q_draggable_pick.single();
	let draggable_ray		= draggable_pick.ray().unwrap();

	let (draggable_e, draggable_parent, mut transform, mut drag) = draggable.single_mut();

	let filter_intersection_e = |e_ref : &Entity| -> bool {
		let e				= *e_ref;
		if e == draggable_e {
			return false;
		}

		if q_gizmo.get(e).is_ok() {
			return false;
		}

		if q_tile.get(e).is_ok() {
			return false;
		}

		return true;
	};

	if let Some(intersections) = draggable_pick.intersect_list() {
		let mut picked		= false;
		let mut new_distance = 0.0;
		for (e_ref, data) in intersections.iter() {
			if !filter_intersection_e(e_ref) {
				continue;
			}

			if !picked {
				picked 		= true;
				new_distance = data.distance();
			} else {
				new_distance = std::primitive::f32::min(data.distance(), new_distance);
			}
		}

		if !drag.started_picking {
			drag.init_pick_distance = new_distance;
			drag.started_picking = true;
		}

		drag.pick_distance = new_distance;
	}

	//

	let mut picked			= false;
	let mut new_distance	= 0.0;
	if let Some(intersections) = mouse_pick.intersect_list() {
		for (e_ref, data) in intersections.iter() {
			if !filter_intersection_e(e_ref) {
				continue;
			}

			if !picked {
				picked 	= true;
				new_distance = data.distance();
			} else {
				new_distance = std::primitive::f32::min(data.distance(), new_distance);
			}
		}
	}

	// putting draggable object on the line between camera and mouse cursor intersection while preserving y coordinate
	let mut picked_pos  = mouse_ray.origin() + mouse_ray.direction() * new_distance;

	// add pick2object offset that keeps offset between mouse ray intersection position and draggable transform.
	// it is used to prevent initial "jerk" of an object when we start dragging it.
	// TODO: there is still a small jerk, will have to return to this after uneven terrain
	picked_pos += drag.pick2object_offset;

	let (x1, y1, z1) = mouse_ray.origin().into();
	let (x2, y2, z2) = picked_pos.into();

	// TODO: fix this when we start having uneven terrain
	let y = drag.init_transform.translation.y + (drag.pick_distance - drag.init_pick_distance);
	// this is x and z derived from line equation, you put your known "y" here. x1, x2 and so on
	// are coordinates of two known points, in our case: mouse_ray origin and picked position.
	let x = ((y - y1) * (x2 - x1)) / (y2 - y1) + x1;
	let z = ((y - y1) * (z2 - z1)) / (y2 - y1) + z1;

	let mut final_translation : Vec3 = [x, y, z].into();

	// put in local space if draggable has parent
	if let Some(draggable_parent_e) = draggable_parent {
		let parent_transform	= q_transform.get(draggable_parent_e.0).unwrap();
		let mat					= parent_transform.compute_matrix();
		final_translation 		= mat.inverse().transform_point3(final_translation);
	}

	transform.translation = final_translation;

	// TODO: implement blender-like controls g + axis etc
	for event in mouse_wheel_events.iter() {
		let dy = match event.unit {
			MouseScrollUnit::Line => event.y * 5.,
			MouseScrollUnit::Pixel => event.y,
		};
		transform.rotation *= Quat::from_rotation_y(dy.to_radians());
    }
}

pub fn dragging_stop_system(
		btn				: Res<Input<MouseButton>>,
	mut	q_draggable_active : Query<(Entity, &mut Draggable), With<DraggableActive>>,
		q_draggable_picking : Query<Entity, With<DraggableRaycast>>,

	mut despawn			: ResMut<DespawnResource>,
	mut commands		: Commands
) {
	let just_released	= btn.just_released(MouseButton::Left);
	if !just_released {
		return;
	}

	if q_draggable_active.is_empty() {
		return;
	}

	let (draggable_e, mut drag)	= q_draggable_active.single_mut();
	commands.entity(draggable_e).remove::<DraggableActive>();

	drag.started_picking = false;

	// despawn a child that is used for picking
	let picking			= q_draggable_picking.single();
	despawn.entities.push(picking);
}

pub struct DraggablePlugin;

// This plugin is responsible to control the game audio
impl Plugin for DraggablePlugin {
    fn build(&self, app: &mut App) {
        app
			.add_system_to_stage(CoreStage::PostUpdate, dragging_start_system)
			.add_system_to_stage(CoreStage::PostUpdate, dragging_system)
			.add_system_to_stage(CoreStage::PostUpdate, dragging_stop_system)
            ;
    }
}