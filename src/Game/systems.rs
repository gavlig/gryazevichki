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
	mut herr_io			: ResMut<HerringboneIO>,
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
		setup_herringbone_brick_road(&mut herr_io, &mut meshes, &mut materials, &ass, &mut commands);

		let y_offset	= 0.5;

		let root_pos	= Vec3::new(1.0, 0.5, 1.0);
		let root_e		= spawn::object_root(root_pos, &mut meshes, &mut materials, &mut commands);

		herr_io.parent = Some(root_e);

		match herr_io.spline.as_ref() {
		Some(spline) => {
			let key0	= spline.get(0).unwrap();
			match key0.interpolation {
				Interpolation::StrokeBezier(V0, V1) => {
					spawn::spline_tangent(root_e, V0, SplineTangent::ID(0), &mut meshes, &mut materials, &mut commands);
				},
				_ => (),
			};
			spawn::spline_control_point(root_e, key0.value, SplineControlPoint::ID(0), &mut meshes, &mut materials, &mut commands);

			let key1	= spline.get(1).unwrap();
			match key1.interpolation {
				Interpolation::StrokeBezier(V0, V1) => {
					spawn::spline_tangent(root_e, V0, SplineTangent::ID(1), &mut meshes, &mut materials, &mut commands);
				},
				_ => (),
			};
			spawn::spline_control_point(root_e, key1.value, SplineControlPoint::ID(1), &mut meshes, &mut materials, &mut commands);
		}
		None => (),
		}
	}

	// polyline
	if false {
		commands.spawn_bundle(PolylineBundle {
			polyline: polylines.add(Polyline {
				vertices: vec![-Vec3::Z, Vec3::Z],
				..default()
			}),
			material: polyline_materials.add(PolylineMaterial {
				width: 20.0,
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
				width: 20.0,
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
				width: 20.0,
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
	mut step		: ResMut<HerringboneStepRequest>,
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
	pub start_pos		: Vec3,
	pub distance		: f32,
	pub init_transform	: Transform,
}

#[derive(Component)]
pub struct DraggableActive;

pub fn mouse_dragging_start_system(
		btn					: Res<Input<MouseButton>>,
		pick_source_query	: Query<&PickingCamera>,
	mut interactions 		: Query<
	(
		Entity,
		&Interaction,
		&Transform,
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
	let top_pick = pick_source.intersect_top();

	// There is at least one entity under the cursor
	if top_pick.is_none() {
		return;
	}
	
	let (topmost_entity, intersection) = top_pick.unwrap();
	
	if let Ok((entity, interaction, transform, mut drag)) = interactions.get_mut(topmost_entity) {
		if *interaction != Interaction::Clicked {
			return;
		}

		let cur_pos = intersection.position();

		drag.start_pos = cur_pos;
		drag.distance = intersection.distance();
		drag.init_transform = transform.clone();

		commands.entity(entity).insert(DraggableActive);
	}
}

pub fn mouse_dragging_system(
	mut mouse_wheel_events	: EventReader<MouseWheel>,
		pick_source_query	: Query<&PickingCamera>,
	mut draggable			: Query<
	(
		Entity,
		&mut Transform,
		&mut Draggable
	), With<DraggableActive>
	>,
		q_gizmo				: Query<Entity, With<Gizmo>>,
		q_tile				: Query<Entity, With<Tile>>,
) {
	let pick_source 	= pick_source_query.single();
	if pick_source.ray().is_none() {
		return;
	}

	if draggable.is_empty() {
		return;
	}
	
	let (entity, mut transform, mut drag) = draggable.single_mut();
	let ray 			= pick_source.ray().unwrap();

	let mut picked		= false;
	if let Some(intersections) = pick_source.intersect_list() {
		let mut i		= 0;
		let cnt			= intersections.len();
		
		while i < cnt {
			let (e, data) = intersections[i];
			i			+= 1;

			if e == entity {
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
				drag.distance = data.distance();
			} else {
				drag.distance = std::primitive::f32::min(data.distance(), drag.distance);
			}
		}
	}


	let offset_from_picked = 0.5;

	let mut cur_pos 	= ray.origin() + ray.direction() * (drag.distance - offset_from_picked);
	let mut delta 		= cur_pos - drag.start_pos;
	// TODO: implement blender-like controls g + axis etc

	transform.translation = drag.init_transform.translation + delta;

	for event in mouse_wheel_events.iter() {
		let dy = match event.unit {
			MouseScrollUnit::Line => event.y * 5.,
			MouseScrollUnit::Pixel => event.y,
		};
		screen_print!("event.y: {:.3} dy: {:.3}", event.y, dy);
		transform.rotation *= Quat::from_rotation_y(dy.to_radians());
    }
}

pub fn mouse_dragging_stop_system(
		btn				: Res<Input<MouseButton>>,
		draggable_active: Query<Entity, With<DraggableActive>>,
	mut commands		: Commands
) {
	let just_released	= btn.just_released(MouseButton::Left);
	if !just_released {
		return;
	}

	if draggable_active.is_empty() {
		return;
	}

	let draggable			= draggable_active.single();
	commands.entity(draggable).remove::<DraggableActive>();
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

pub fn setup_herringbone_brick_road(
	io					: &mut ResMut<HerringboneIO>,
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	ass					: &Res<AssetServer>,
	mut commands		: &mut Commands
) {
//	let hsize 			= Vec3::new(0.1075 / 2.0, 0.065 / 2.0, 0.215 / 2.0);
	let hsize 			= Vec3::new(0.2 / 2.0, 0.05 / 2.0, 0.4 / 2.0);
//	let hsize 			= Vec3::new(0.1075, 0.065, 0.215);

	io.set_default		();

	io.width			= 10.0;
	io.length			= 10.0;
	io.limit			= 100;

	io.body_type		= RigidBody::Fixed;
	io.hsize			= hsize;
	io.seam				= 0.01;
	io.mesh				= meshes.add(Mesh::from(render_shape::Box::new(hsize.x * 2.0, hsize.y * 2.0, hsize.z * 2.0)));
	io.material			= materials.add(StandardMaterial { base_color: Color::ALICE_BLUE,..default() });

	let y_offset		= 0.5;
	
	// spline requires at least 4 points: 2 control points(Key) and 2 tangents
	//
	//
	let tangent0		= Vec3::new(0.0, y_offset, 2.5);
	let tangent1		= Vec3::new(0.0, y_offset, 7.5);
	// z_limit is used both for final coordinate and for final value of t to have road length tied to spline length and vice versa
	let control_point0_pos = Vec3::new(0.0, y_offset, 0.0);
	let control_point1_pos = Vec3::new(0.0, y_offset, io.length);
	// z_limit as final 't' value lets us probe spline from any z offset of a tile
	let t0				= 0.0;
	let t1				= io.length;

	let control_point0	= Key::new(t0, control_point0_pos, Interpolation::StrokeBezier(tangent0, tangent0));
	let control_point1	= Key::new(t1, control_point1_pos, Interpolation::StrokeBezier(tangent1, tangent1));

	io.spline 			= Some(Spline::from_vec(vec![control_point0, control_point1]));
}

pub fn herringbone_brick_road_system(
	mut step			: ResMut<HerringboneStepRequest>,
	mut io				: ResMut<HerringboneIO>,
	mut despawn			: ResMut<DespawnResource>,
		time			: Res<Time>,
		ass				: Res<AssetServer>,

		query			: Query<Entity, With<Herringbone>>,

	mut commands		: Commands
) {
	if step.reset {
		for e in query.iter() {
			despawn.entities.push(e);
		}
	
		io.reset_changed();
	
		step.reset		= false;
	}

	let do_spawn 		= step.next || step.animate;
	if !do_spawn || io.finished {
		return;
	}

	let cur_time		= time.seconds_since_startup();
	if (cur_time - step.last_update) < step.anim_delay_sec && !step.instant {
		return;
	}

	step.last_update 	= cur_time;

	if !step.instant {
		spawn::herringbone_brick_road_iter(&mut io, &ass, &mut commands);
	} else {
		loop {
			spawn::herringbone_brick_road_iter(&mut io, &ass, &mut commands);
			if io.finished {
				step.instant = false;
				break;
			}
		}
	}

	step.next			= false;

	if io.finished {
		step.animate	= false;
	}
}

pub fn on_spline_tangent_moved(
	mut step			: ResMut<HerringboneStepRequest>,
	mut io				: ResMut<HerringboneIO>,
		time			: Res<Time>,
		q_tangent 		: Query<(&Transform, &SplineTangent), Changed<Transform>>,
) {
	if time.seconds_since_startup() < 1.0 {
		return;
	}

	for (tform, tan) in q_tangent.iter() {
		let tan_pos		= tform.translation;
		match tan {
			SplineTangent::ID(id) => {
				io.set_spline_interpolation(*id, Interpolation::StrokeBezier(tan_pos, tan_pos));
				step.reset = true;
				step.next = true;
				step.instant = true;
			},
		}
	}
}

pub fn on_spline_control_point_moved(
	mut step			: ResMut<HerringboneStepRequest>,
	mut io				: ResMut<HerringboneIO>,
		time			: Res<Time>,
		q_controlp 		: Query<(&Transform, &SplineControlPoint), Changed<Transform>>,
) {
	if time.seconds_since_startup() < 1.0 {
		return;
	}

	for (tform, controlp) in q_controlp.iter() {
		let controlp_pos = tform.translation;
		match controlp {
			SplineControlPoint::ID(id) => {
				io.set_spline_control_point(*id, controlp_pos);

				// io.x_limit = 

				step.reset = true;
				step.next = true;
				step.instant = true;
			},
		}
	}
}

pub fn on_object_root_moved(
	mut step			: ResMut<HerringboneStepRequest>,
	mut io				: ResMut<HerringboneIO>,
		time			: Res<Time>,
		q_root			: Query<&Transform, (With<ObjectRoot>, Changed<Transform>)>,
) {
	if time.seconds_since_startup() < 1.0 {
		return;
	}

	if q_root.is_empty() {
		return;
	}

	let root_pos		= match q_root.get_single() {
		Ok(pos)			=> *pos,
		Err(_)			=> Transform::identity(),
	};

	step.reset 			= true;
	step.next 			= true;
	step.instant 		= true;
}