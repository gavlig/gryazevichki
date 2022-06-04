use bevy			:: { prelude :: * };
use bevy			:: { app::AppExit };
use bevy			:: { input :: mouse :: * };
use bevy_rapier3d	:: { prelude :: * };
use bevy_fly_camera	:: { FlyCamera };
use bevy_mod_picking:: { * };
use bevy_mod_raycast:: { * };
use bevy_polyline	:: { prelude :: * };
use bevy_prototype_debug_lines :: { * };
use iyes_loopless	:: { prelude :: * };
use bevy_debug_text_overlay :: { screen_print };

use std				:: { path::PathBuf };
use splines			:: { Interpolation, Key, Spline };

use bevy::render::mesh::shape as render_shape;
use std::f32::consts:: { * };

use super           :: { * };
use super			:: { spawn };

pub fn setup_camera_system(
		game			: ResMut<GameState>,
	mut query			: Query<&mut FlyCamera>
) {
	// initialize camera with target to look at
	if game.camera.is_some() && game.body.is_some() {
		let mut camera 	= query.get_mut(game.camera.unwrap()).unwrap();
		camera.target 	= Some(game.body.unwrap().entity);
		println!		("camera.target Entity ID {:?}", camera.target);

		// temp
		camera.enabled_follow = false;
		camera.enabled_translation = true;
		camera.enabled_rotation = true;
	}
}

#[derive(Component)]
pub struct RedLine;

#[derive(Component)]
pub struct GreenLine;

#[derive(Component)]
pub struct BlueLine;

pub fn setup_world_system(
	mut _configuration	: ResMut<RapierConfiguration>,
	mut	phys_ctx		: ResMut<DebugRenderContext>,
	mut game			: ResMut<GameState>,
	mut herr_io			: ResMut<Herringbone::IO>,
	mut	meshes			: ResMut<Assets<Mesh>>,
	mut	materials		: ResMut<Assets<StandardMaterial>>,
		ass				: Res<AssetServer>,
	mut commands		: Commands,

	mut polylines		: ResMut<Assets<Polyline>>,
	mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
) {
//	configuration.timestep_mode = TimestepMode::VariableTimestep;
	phys_ctx.enabled	= false;

	spawn::camera		(&mut game, &mut commands);

	spawn::ground		(&game, &mut meshes, &mut materials, &mut commands);

	spawn::world_axis	(&mut meshes, &mut materials, &mut commands);

	if false {
		spawn::cubes	(&mut commands);
	}

	if false {
		spawn::friction_tests(&mut meshes, &mut materials, &mut commands);
	}

	if false {
		spawn::obstacles(&mut meshes, &mut materials, &mut commands);
	}

	if false {
		spawn::spheres	(&mut meshes, &mut materials, &mut commands);
	}

	if false {
		spawn::wall		(&mut meshes, &mut materials, &mut commands);
	}

	if true {
		// TODO: make it startup system instead
		Herringbone::brick_road_setup(&mut herr_io, &mut meshes, &mut materials, &ass, &mut commands);

		let y_offset	= 0.5;

		let root_pos	= Vec3::new(1.0, y_offset, 1.0);
		let root_e		= Herringbone::spawn::object_root(root_pos, &mut meshes, &mut materials, &mut commands);

		herr_io.parent = Some(root_e);

		match herr_io.spline.as_ref() {
		Some(spline) => {
			let key0	= spline.get(0).unwrap();
			match key0.interpolation {
				Interpolation::StrokeBezier(V0, V1) => {
					Herringbone::spawn::spline_tangent(root_e, V0, SplineTangent::ID(0), &mut meshes, &mut materials, &mut commands);
				},
				_ => (),
			};
			Herringbone::spawn::spline_control_point(root_e, key0.value, SplineControlPoint::ID(0), &mut meshes, &mut materials, &mut commands);

			let key1	= spline.get(1).unwrap();
			match key1.interpolation {
				Interpolation::StrokeBezier(V0, V1) => {
					Herringbone::spawn::spline_tangent(root_e, V0, SplineTangent::ID(1), &mut meshes, &mut materials, &mut commands);
				},
				_ => (),
			};
			Herringbone::spawn::spline_control_point(root_e, key1.value, SplineControlPoint::ID(1), &mut meshes, &mut materials, &mut commands);
		}
		None => (),
		}
	}

	// polyline
	if true {
		commands.spawn_bundle(PolylineBundle {
			polyline: polylines.add(Polyline {
				vertices: vec![-Vec3::Z, Vec3::Z],
				..default()
			}),
			material: polyline_materials.add(PolylineMaterial {
				width: 100.0,
				color: Color::RED,
				perspective: true,
				..default()
			}),
			..default()
		})
		.insert(RedLine);
		
		commands.spawn_bundle(PolylineBundle {
			polyline: polylines.add(Polyline {
				vertices: vec![-Vec3::Z, Vec3::Z],
				..default()
			}),
			material: polyline_materials.add(PolylineMaterial {
				width: 100.0,
				color: Color::SEA_GREEN,
				perspective: true,
				..default()
			}),
			..default()
		})
		.insert(GreenLine);
	
		commands.spawn_bundle(PolylineBundle {
			polyline: polylines.add(Polyline {
				vertices: vec![-Vec3::Z, Vec3::Z],
				..default()
			}),
			material: polyline_materials.add(PolylineMaterial {
				width: 100.0,
				color: Color::MIDNIGHT_BLUE,
				perspective: true,
				..default()
			}),
			..default()
		})
		.insert(BlueLine);
	}

	let veh_file		= Some(PathBuf::from("corvette.ron"));
	let veh_cfg			= load_vehicle_config(&veh_file).unwrap();

	let body_pos 		= Transform::from_xyz(0.0, 5.5, 0.0);

	Vehicle::spawn(
		  &veh_cfg
		, body_pos
		, &mut game
		, &ass
		, &mut commands
	);
}

pub fn setup_lighting_system(
	mut	meshes					: ResMut<Assets<Mesh>>,
	mut	materials				: ResMut<Assets<StandardMaterial>>,
	mut game					: ResMut<GameState>,
	mut commands				: Commands,
) {
	const HALF_SIZE: f32		= 100.0;

	commands.spawn_bundle(DirectionalLightBundle {
		directional_light: DirectionalLight {
			illuminance: 8000.0,
			// Configure the projection to better fit the scene
			shadow_projection	: OrthographicProjection {
				left			: -HALF_SIZE,
				right			:  HALF_SIZE,
				bottom			: -HALF_SIZE,
				top				:  HALF_SIZE,
				near			: -10.0 * HALF_SIZE,
				far				: 100.0 * HALF_SIZE,
				..default()
			},
			shadows_enabled		: true,
			..default()
		},
		transform				: Transform {
			translation			: Vec3::new(10.0, 2.0, 10.0),
			rotation			: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
			..default()
		},
		..default()
	});

	// commands
	//     .spawn_bundle(DirectionalLightBundle {
	//         ..Default::default()
	//     })
	//     .insert(Sun); // Marks the light as Sun

	//
}

pub fn setup_cursor_visibility_system(
	mut windows	: ResMut<Windows>,
	mut picking	: ResMut<PickingPluginsState>,
) {
	let window = windows.get_primary_mut().unwrap();

	window.set_cursor_lock_mode	(true);
	window.set_cursor_visibility(false);

	picking.enable_picking 		= false;
	picking.enable_highlighting = false;
	picking.enable_interacting 	= false;
}

pub fn cursor_visibility_system(
	mut windows		: ResMut<Windows>,
	btn				: Res<Input<MouseButton>>,
	key				: Res<Input<KeyCode>>,
	time			: Res<Time>,
	mut window_focused : EventReader<bevy::window::WindowFocused>,
		game_mode	: Res<CurrentState<GameMode>>,
	mut picking		: ResMut<PickingPluginsState>,
	mut	commands	: Commands
) {
	let window 		= windows.get_primary_mut().unwrap();
	let cursor_visible = window.cursor_visible();
	let window_id	= window.id();

	let mut set_cursor_visibility = |v| {
		window.set_cursor_visibility(v);
		window.set_cursor_lock_mode(!v);
	};

	let mut set_visibility = |v| {
		set_cursor_visibility(v);

		picking.enable_picking = v;
		picking.enable_highlighting = v;
		picking.enable_interacting = v;

		commands.insert_resource(NextState(
			if v { GameMode::Editor } else { GameMode::InGame }
		));
	};

	if key.just_pressed(KeyCode::Escape) {
		let toggle 	= !cursor_visible;
		set_visibility(toggle);
	}

	if btn.just_pressed(MouseButton::Left) && game_mode.0 == GameMode::InGame{
		set_cursor_visibility(false);
	}

	if time.seconds_since_startup() > 1.0 {
		for ev in window_focused.iter() {
			if ev.id == window_id {

				if !ev.focused {
					set_cursor_visibility(true);
				} else {
					// this works bad because winit says we cant grab cursor right after window gets alt-tabbed back to focused
					set_cursor_visibility(game_mode.0 == GameMode::Editor);
				}
			}
		}
	}
}

pub fn input_misc_system(
		_btn		: Res<Input<MouseButton>>,
		key			: Res<Input<KeyCode>>,
		_game		: Res<GameState>,
		time		: Res<Time>,
	mut	phys_ctx	: ResMut<DebugRenderContext>,
	mut step		: ResMut<Herringbone::StepRequest>,
	mut exit		: EventWriter<AppExit>,
	mut query		: Query<&mut FlyCamera>,
) {
	for mut camera in query.iter_mut() {
		if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Space) {
			let toggle 	= !camera.enabled_follow;
			camera.enabled_follow = toggle;
		}

		if key.just_pressed(KeyCode::Escape) {
			let toggle 	= !camera.enabled_rotation;
			camera.enabled_rotation = toggle;
		}
	}

	if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Escape) {
		exit.send(AppExit);
	}

	if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Key1) {
		phys_ctx.enabled = !phys_ctx.enabled;
	}

	if key.pressed(KeyCode::LControl) && key.pressed(KeyCode::T) {
		step.next 		= true;
	}

	if key.just_released(KeyCode::T) {
		step.next 		= false;
	}

	if key.pressed(KeyCode::LAlt) && key.just_pressed(KeyCode::T) {
		step.reset		= true;
	}

	if key.pressed(KeyCode::RControl) && !key.pressed(KeyCode::RShift) && key.just_pressed(KeyCode::T) {
		step.animate	= true;
		step.last_update = time.seconds_since_startup();
	}

	if key.pressed(KeyCode::RControl) && key.pressed(KeyCode::RShift) && key.just_pressed(KeyCode::T) {
		step.instant	= true;
		step.next 		= true;
		step.last_update = time.seconds_since_startup();
	}
}

#[derive(Component, Default)]
pub struct Draggable {
	pub init_transform	: GlobalTransform,
	pub started_picking	: bool,
	pub init_pick_distance : f32,
	pub pick_distance	: f32,
}

#[derive(Component)]
pub struct DraggableActive;

#[derive(Component)]
pub struct DraggableRaycast;

pub fn mouse_dragging_start_system(
		btn					: Res<Input<MouseButton>>,
		pick_source_query	: Query<&PickingObject, With<Camera>>,
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

	let pick_source = pick_source_query.single();
	let ray 		= pick_source.ray().unwrap();
	let top_pick 	= pick_source.intersect_top();

	// There is at least one entity under the cursor
	if top_pick.is_none() {
		return;
	}
	
	let (topmost_entity, intersection) = top_pick.unwrap();
	
	if let Ok((clicked_entity, interaction, global_transform, mut drag)) = interactions.get_mut(topmost_entity) {
		if *interaction != Interaction::Clicked {
			return;
		}

		drag.init_transform = global_transform.clone();

		commands.entity(clicked_entity).insert(DraggableActive);

		// rotating a child caster so that raycast points to the ground. It is used to keep draggable on the same-ish offset above the ground
		let mut transform	= Transform::identity();
		transform.look_at	(-Vec3::Y, -Vec3::Z);

		commands.entity(clicked_entity).with_children(|parent| {
			parent.spawn_bundle(TransformBundle{ local : transform, ..default() })
				.insert		(PickingObject::new_transform_empty())
				.insert		(DraggableRaycast);
		});
	}
}

pub fn mouse_dragging_system(
	mut polylines			: ResMut<Assets<Polyline>>,
	mut q_green				: Query<&Handle<Polyline>, With<GreenLine>>,
	mut q_blue				: Query<&Handle<Polyline>, With<BlueLine>>,

	mut mouse_wheel_events	: EventReader<MouseWheel>,
		q_transform			: Query<&GlobalTransform, Without<DraggableActive>>,
		q_draggable_pick	: Query<&PickingObject, With<DraggableRaycast>>,
		q_mouse_pick		: Query<&PickingCamera, With<Camera>>,
	mut draggable			: Query<
	(
		Entity,
		Option<&Parent>,
		&GlobalTransform,
		&mut Transform,
		&mut Draggable
	), With<DraggableActive>
	>,
		q_gizmo				: Query<Entity, With<Gizmo>>,
		q_tile				: Query<Entity, With<Herringbone::Tile>>,
) {
	if draggable.is_empty() {
		return;
	}
	
	let mouse_pick 			= q_mouse_pick.single();
	let mouse_ray 			= mouse_pick.ray().unwrap();

	let draggable_pick 		= q_draggable_pick.single();
	let draggable_ray		= draggable_pick.ray().unwrap();

	let (draggable_e, draggable_parent, global_transform, mut transform, mut drag) = draggable.single_mut();

	if let Some(intersections) = draggable_pick.intersect_list() {
		let mut picked		= false;
		let mut new_distance = 0.0;
		for (e_ref, data) in intersections.iter() {
			let e			= *e_ref;
			if e == draggable_e {
				continue;
			}

			if q_gizmo.get(e).is_ok() {
				continue;
			}

			if q_tile.get(e).is_ok() {
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
		let mut i			= 0;
		let cnt				= intersections.len();
		
		for (e_ref, data) in intersections.iter() {
			let e			= *e_ref;
			if e == draggable_e {
				continue;
			}

			if q_gizmo.get(e).is_ok() {
				continue;
			}

			if q_tile.get(e).is_ok() {
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

	let picked_pos  = mouse_ray.origin() + mouse_ray.direction() * new_distance;

	let (x1, y1, z1) = mouse_ray.origin().into();
	let (x2, y2, z2) = picked_pos.into();

	let y = drag.init_transform.translation.y + (drag.pick_distance - drag.init_pick_distance);
	let x = ((y - y1) * (x2 - x1)) / (y2 - y1) + x1;
	let z = ((y - y1) * (z2 - z1)) / (y2 - y1) + z1;

	let mut final_translation : Vec3 = [x, y, z].into();

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
		screen_print!("event.y: {:.3} dy: {:.3}", event.y, dy);
		transform.rotation *= Quat::from_rotation_y(dy.to_radians());
    }

	// TODO: make utils from this
	if false {
		let origin = draggable_ray.origin();
		let target = draggable_ray.origin() + draggable_ray.direction() * 2.0;
		let line	= polylines.get_mut(q_green.single()).unwrap();
		line.vertices.resize(10, Vec3::ZERO);
		line.vertices[0] = origin;
		line.vertices[1] = target;
		line.vertices[2] = target + Vec3::X * 0.2;
		line.vertices[3] = target;
		line.vertices[4] = target - Vec3::X * 0.2;
		line.vertices[5] = target;
		line.vertices[6] = target + Vec3::Z * 0.2;
		line.vertices[7] = target;
		line.vertices[8] = target - Vec3::Z * 0.2;
		line.vertices[9] = target;
	}

	if false {
		let origin = mouse_ray.origin();
		let target = mouse_ray.origin() + mouse_ray.direction() * new_distance;
		let line	= polylines.get_mut(q_blue.single()).unwrap();
		line.vertices.resize(10, Vec3::ZERO);
		line.vertices[0] = origin;
		line.vertices[1] = target;
		line.vertices[2] = target + Vec3::X * 0.2;
		line.vertices[3] = target;
		line.vertices[4] = target - Vec3::X * 0.2;
		line.vertices[5] = target;
		line.vertices[6] = target + Vec3::Z * 0.2;
		line.vertices[7] = target;
		line.vertices[8] = target - Vec3::Z * 0.2;
		line.vertices[9] = target;
	}
}

pub fn mouse_dragging_stop_system(
		btn				: Res<Input<MouseButton>>,
		q_draggable_active : Query<Entity, With<DraggableActive>>,
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

	let draggable		= q_draggable_active.single();
	commands.entity(draggable).remove::<DraggableActive>();

	// despawn a child that is used for picking
	let picking			= q_draggable_picking.single();
	despawn.entities.push(picking);
}

#[derive(Default)]
pub struct DespawnResource {
	pub entities: Vec<Entity>,
}

pub fn despawn_system(mut commands: Commands, time: Res<Time>, mut despawn: ResMut<DespawnResource>) {
	if time.seconds_since_startup() > 5.0 {
		for entity in &despawn.entities {
//			println!("Despawning entity {:?}", entity);
			commands.entity(*entity).despawn_recursive();
		}
		despawn.entities.clear();
	}
}

pub fn display_events_system(
	mut _collision_events: EventReader<CollisionEvent>,
) {
//	for intersection_event in intersection_events.iter() {
//		println!("Received intersection event: collider1 {:?} collider2 {:?}", intersection_event.collider1.entity(), intersection_event.collider2.entity());
//	}
//
//	for contact_event in contact_events.iter() {
//		match contact_event {
//			ContactEvent::Started(collider1, collider2) => println!("Received contact START event: collider1 {:?} collider2 {:?}", collider1.entity(), collider2.entity()),
//			ContactEvent::Stopped(collider1, collider2) => println!("Received contact STOP event: collider1 {:?} collider2 {:?}", collider1.entity(), collider2.entity()),
//		}
//	}
}