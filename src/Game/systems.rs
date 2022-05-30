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

		// temp
		camera.enabled_follow = false;
		camera.enabled_translation = true;
		camera.enabled_rotation = true;
	}
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
		setup_herringbone_brick_road(&mut herr_io, &mut meshes, &mut materials, &ass);
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

	// use splines::{Interpolation, Key, Spline};

	// let k0 = Key::new(0., Vec3::new(0.0, 0.5, 0.0), Interpolation::Bezier(Vec3::new(3.0, 0.5, 0.0)));
	// let k1 = Key::new(1., Vec3::new(0.0, 0.5, 3.0), Interpolation::Bezier(Vec3::new(-3.0, 0.5, 3.0)));
	// let k2 = Key::new(2., Vec3::new(0.0, 0.5, 6.0), Interpolation::Bezier(Vec3::new(3.0, 0.5, 6.0)));
	// let spline = Spline::from_vec(vec![k0, k1, k2]);

	// let num = 20;
	// for i in 0..num {
	// 	let t = (2.0 / num as f32) * i as f32;

	// 	let p = spline.sample(t).unwrap();

	// 	let axis_cube	= ass.load("utils/axis_cube.gltf#Scene0");
	// 	commands.spawn_bundle(
	// 		TransformBundle {
	// 			local: Transform::from_translation(p),
	// 			global: GlobalTransform::default(),
	// 	}).with_children(|parent| {
	// 		parent.spawn_scene(axis_cube);
	// 	});
	// }

	let axis_cube	= ass.load("utils/axis_cube.gltf#Scene0");
	let mut p = Transform::from_translation(Vec3::new(0.0, 2.0, 10.0));
	p.scale = Vec3::new(2.0, 2.0, 2.0);
		commands.spawn_bundle(
			TransformBundle {
				local: p,
				global: GlobalTransform::default(),
		}).with_children(|parent| {
			parent.spawn_scene(axis_cube);
		});
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
	_ass				: &Res<AssetServer>,
) {
	let body_type		= RigidBody::Fixed;
	
//	let hsize 			= Vec3::new(0.1075 / 2.0, 0.065 / 2.0, 0.215 / 2.0);
	let hsize 			= Vec3::new(0.2 / 2.0, 0.05 / 2.0, 0.4 / 2.0);
//	let hsize 			= Vec3::new(0.1075, 0.065, 0.215);

	let offset 			= Vec3::new(1.0, hsize.y, 1.0);

	io.set_default		();

	io.x_limit			= 3.0;
	io.z_limit			= 10.0;
	io.limit			= 100;

	io.body_type		= body_type;
	io.offset			= offset;
	io.hsize			= hsize;
	io.seam				= 0.01;
	io.mesh				= meshes.add(Mesh::from(render_shape::Box::new(hsize.x * 2.0, hsize.y * 2.0, hsize.z * 2.0)));
	io.material			= materials.add(StandardMaterial { base_color: Color::ALICE_BLUE,..default() });

	use splines :: { Interpolation, Key, Spline };

	let y_offset		= 0.5;
	// let k0 = Key::new(0., Vec3::new(0.0, y_offset, 0.0), Interpolation::Bezier(Vec3::new(0.0, y_offset, -2.5)));
	// let k1 = Key::new(2.5, Vec3::new(2.5, y_offset, 2.5), Interpolation::Bezier(Vec3::new(5.0, y_offset, 2.5)));
	// let k2 = Key::new(5., Vec3::new(0.0, y_offset, 5.0), Interpolation::Bezier(Vec3::new(0.0, y_offset, 7.5)));
	// let k3 = Key::new(7.5, Vec3::new(-2.5, y_offset, 2.5), Interpolation::Bezier(Vec3::new(-5.0, y_offset, 2.5)));
	// let k4 = Key::new(10., Vec3::new(0.0, y_offset, 0.0), Interpolation::Bezier(Vec3::new(0.0, y_offset, -2.5)));
	let k0 = Key::new(0., Vec3::new(0.0, 0.5, 0.0), Interpolation::Bezier(Vec3::new(1.0, 0.5, 0.0)));
	let k1 = Key::new(5., Vec3::new(2.0, 0.5, 5.0), Interpolation::Bezier(Vec3::new(1.0, 0.5, 5.0)));
	let k2 = Key::new(10., Vec3::new(0.0, 0.5, 10.0), Interpolation::Bezier(Vec3::new(1.0, 0.5, 10.0)));
	io.spline 			= Some(Spline::from_vec(vec![k0, k1, k2]));
}

pub fn herringbone_brick_road_system(
	mut step			: ResMut<HerringboneStepRequest>,
	mut io				: ResMut<HerringboneIO>,
	mut despawn			: ResMut<DespawnResource>,
		time			: Res<Time>,

	mut meshes			: ResMut<Assets<Mesh>>,
	mut materials		: ResMut<Assets<StandardMaterial>>,
		ass				: Res<AssetServer>,

		query			: Query<Entity, With<Herringbone>>,

	mut commands		: Commands
) {
	if step.reset {
		for e in query.iter() {
			despawn.entities.push(e);
		}

		setup_herringbone_brick_road(&mut io, &mut meshes, &mut materials, &ass);

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

	spawn::herringbone_brick_road_iter(&mut io, &ass, &mut commands);
	step.next			= false;

	if io.finished {
		step.animate	= false;
	}
}