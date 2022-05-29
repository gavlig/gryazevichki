use bevy			:: { prelude :: * };
use bevy			:: { app::AppExit };
use bevy_rapier3d	:: { prelude :: * };
use bevy_fly_camera	:: { FlyCamera };
use bevy_mod_picking:: { * };
use iyes_loopless	:: { prelude :: * };

use std				:: { path::PathBuf };

use bevy::render::mesh::shape as render_shape;
use std::f32::consts:: 	{ * };

use super           ::  { * };
use super			::	{ spawn };

pub fn setup_camera_system(
		game			: ResMut<GameState>,
	mut query			: Query<&mut FlyCamera>
) {
	// initialize camera with target to look at
	if game.camera.is_some() && game.body.is_some() {
		let mut camera 	= query.get_mut(game.camera.unwrap()).unwrap();
		camera.target 	= Some(game.body.unwrap().entity);
		println!		("camera.target Entity ID {:?}", camera.target);
	}
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

pub fn setup_graphics_system(
	mut	meshes					: ResMut<Assets<Mesh>>,
	mut	materials				: ResMut<Assets<StandardMaterial>>,
	mut game					: ResMut<GameState>,
	mut commands				: Commands,
) {
	const HALF_SIZE: f32		= 100.0;

	commands.spawn_bundle(DirectionalLightBundle {
		directional_light: DirectionalLight {
			illuminance: 10000.0,
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

	spawn::world_axis	(&mut meshes, &mut materials, &mut commands);

	spawn::camera		(&mut game, &mut commands);
}

pub fn setup_world_system(
	mut _configuration	: ResMut<RapierConfiguration>,
	mut	phys_ctx		: ResMut<DebugRenderContext>,
	mut game			: ResMut<GameState>,
	mut herr_io			: ResMut<HerringboneIO>,
	mut	meshes			: ResMut<Assets<Mesh>>,
	mut	materials		: ResMut<Assets<StandardMaterial>>,
		ass				: Res<AssetServer>,
	mut commands		: Commands
) {
//	configuration.timestep_mode = TimestepMode::VariableTimestep;
	phys_ctx.enabled	= false;

	spawn::ground		(&game, &mut meshes, &mut materials, &mut commands);

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
//		spawn::herringbone_brick_road(&mut meshes, &mut materials, &mut commands);
		setup_herringbone_brick_road(&mut herr_io, &mut meshes, &mut materials, &mut commands);
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

#[derive(Default)]
pub struct DespawnResource {
	pub entities: Vec<Entity>,
}

pub fn despawn_system(mut commands: Commands, time: Res<Time>, mut despawn: ResMut<DespawnResource>) {
	if time.seconds_since_startup() > 5.0 {
		for entity in &despawn.entities {
			println!("Despawning entity {:?}", entity);
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

pub fn cursor_visibility_system(
	mut windows		: ResMut<Windows>,
	_btn			: Res<Input<MouseButton>>,
	key				: Res<Input<KeyCode>>,
	time			: Res<Time>,
	mut window_focused : EventReader<bevy::window::WindowFocused>,
	mut picking		: ResMut<PickingPluginsState>,
	mut	commands	: Commands
) {
	let window 		= windows.get_primary_mut().unwrap();
	let cursor_visible = window.cursor_visible();
	// let window_id	= window.id();

	let mut set_visibility = |v| {
		window.set_cursor_visibility(v);
		window.set_cursor_lock_mode(!v);

		picking.enable_picking = v;
		picking.enable_highlighting = v;
		picking.enable_interacting = v;

		commands.insert_resource(NextState(
			if v { GameMode::Editor } else { GameMode::InGame }
		));
	};

	if key.just_pressed(KeyCode::Escape) {
		let toggle 	= !cursor_visible;
		// println!("set visibility: {} because esc pressed", toggle);
		set_visibility(toggle);
	}

	// if time.seconds_since_startup() > 1.0 {
	// 	for ev in window_focused.iter() {
	// 		if ev.id == window_id {
	// 			println!("set visibility: {} because focus changed", ev.focused);
	// 			set_visibility(ev.focused);
	// 		}
	// 	}
	// }
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

	if key.pressed(KeyCode::RControl) && key.just_pressed(KeyCode::T) {
		step.animate	= true;
		step.last_update = time.seconds_since_startup();
	}
}

pub fn setup_herringbone_brick_road(
	io					: &mut ResMut<HerringboneIO>,
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	commands			: &mut Commands
) {
	let body_type		= RigidBody::Fixed;
	let num_x			= 3u32 * 3u32;
	let num_z			= 3u32 * 3u32;
	
//	let hsize 			= Vec3::new(0.1075 / 2.0, 0.065 / 2.0, 0.215 / 2.0);
	let hsize 			= Vec3::new(0.2 / 2.0, 0.05 / 2.0, 0.4 / 2.0);
//	let hsize 			= Vec3::new(0.1075, 0.065, 0.215);

	let offset 			= Vec3::new(1.0, hsize.y, 1.0);

	io.set_default		();

	io.x_limit			= 3.0;
	io.z_limit			= 3.0;
	io.limit			= 30;

	io.body_type		= body_type;
	io.offset			= offset;
	io.hsize			= hsize;
	io.mesh				= meshes.add(Mesh::from(render_shape::Box::new(hsize.x * 2.0, hsize.y * 2.0, hsize.z * 2.0)));
	io.material			= materials.add(StandardMaterial { base_color: Color::ALICE_BLUE,..default() });
}

pub fn herringbone_brick_road_system(
	mut step			: ResMut<HerringboneStepRequest>,
	mut io				: ResMut<HerringboneIO>,
	mut despawn			: ResMut<DespawnResource>,
		time			: Res<Time>,

	mut meshes			: ResMut<Assets<Mesh>>,
	mut materials		: ResMut<Assets<StandardMaterial>>,

		query			: Query<Entity, With<Herringbone>>,

	mut commands		: Commands
) {
	if step.reset {
		for e in query.iter() {
			despawn.entities.push(e);
		}

		setup_herringbone_brick_road(&mut io, &mut meshes, &mut materials, &mut commands);

		step.reset		= false;
		step.next		= false;
		step.animate	= false;
		return;
	}

	let do_spawn 		= step.next || step.animate;
	if !do_spawn || io.finished {
		return;
	}

	let cur_time		= time.seconds_since_startup();
	if (cur_time - step.last_update) < step.anim_delay_sec {
		return;
	}

	step.last_update 	= cur_time;

	spawn::herringbone_brick_road_iter(&mut io, &mut commands);
	step.next			= false;

	if io.finished {
		step.animate	= false;
	}
}