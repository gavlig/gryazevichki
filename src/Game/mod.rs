use bevy			::	{ prelude :: * };
use bevy			::	{ app::AppExit };
use bevy_rapier3d	::	{ prelude :: * };
use bevy_fly_camera	::	{ FlyCamera };

use std				:: 	{ path::PathBuf };
use serde			::	{ Deserialize, Serialize };

pub mod Vehicle;
pub use Vehicle		:: *;
pub mod Ui;
pub use Ui			:: *;			

mod spawn;

pub struct GameState {
	  pub camera				: Option<Entity>
	, pub body 					: Option<RespawnableEntity>

	, pub wheels				: [Option<RespawnableEntity>; WHEELS_MAX as usize]
	, pub axles					: [Option<RespawnableEntity>; WHEELS_MAX as usize]

	, pub load_veh_dialog		: Option<FileDialog>
	, pub save_veh_dialog		: Option<FileDialog>

	, pub save_veh_file			: Option<PathBuf>
	, pub load_veh_file			: Option<PathBuf>
}

impl Default for GameState {
	fn default() -> Self {
		Self {
			  camera			: None
			, body 				: None
	  
			, wheels			: [None; WHEELS_MAX as usize]
			, axles				: [None; WHEELS_MAX as usize]
	  
			, load_veh_dialog	: None
			, save_veh_dialog	: None

			, save_veh_file		: None
			, load_veh_file		: None
	  }
  }
}

#[derive(Component)]
pub struct NameComponent {
	pub name : String
}

#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub enum SideX {
	Left,
	Center,
	Right
}

#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub enum SideY {
	Top,
	Center,
	Bottom
}

#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub enum SideZ {
	Front,
	Center,
	Rear
}

#[derive(Debug, Clone, Copy)]
pub struct RespawnableEntity {
	entity			: Entity,
	respawn			: bool
}

impl Default for RespawnableEntity {
	fn default() -> Self {
		Self {
			  entity			: Entity::from_bits(0)
			, respawn			: false
		}
	}
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PhysicsConfig {
	  pub fixed					: bool
	, pub density				: f32
	, pub mass					: f32
	, pub friction				: f32
	, pub restitution			: f32
	, pub lin_damping			: f32
	, pub ang_damping			: f32
}

impl Default for PhysicsConfig {
	fn default() -> Self {
		Self {
			  fixed				: false
			, density			: 1.0
			, mass				: 0.0 // calculated at runtime
			, friction			: 0.5
			, restitution		: 0.0
			, lin_damping		: 0.0
			, ang_damping		: 0.0
		}
	}
}

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

pub fn setup_cursor_visibility_system(mut windows: ResMut<Windows>) {
	let window = windows.get_primary_mut().unwrap();

	window.set_cursor_lock_mode	(true);
	window.set_cursor_visibility(false);
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
	mut game			: ResMut<GameState>,
	mut	meshes			: ResMut<Assets<Mesh>>,
	mut	materials		: ResMut<Assets<StandardMaterial>>,
		ass				: Res<AssetServer>,
	mut commands		: Commands
) {
//	configuration.timestep_mode = TimestepMode::VariableTimestep;

	spawn::ground		(&game, &mut meshes, &mut materials, &mut commands);

	if false {
		spawn::cubes	(&mut commands);
	}

	if false {
		spawn::friction_tests(&mut meshes, &mut materials, &mut commands);
	}

	if true {
		spawn::obstacles(&mut meshes, &mut materials, &mut commands);
	}

	if true {
		spawn::spheres	(&mut meshes, &mut materials, &mut commands);
	}

	if true {
		spawn::wall		(&mut meshes, &mut materials, &mut commands);
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

pub fn cursor_grab_system(
	mut windows		: ResMut<Windows>,
	_btn			: Res<Input<MouseButton>>,
	key				: Res<Input<KeyCode>>,
) {
	let window = windows.get_primary_mut().unwrap();

	if key.just_pressed(KeyCode::Escape) {
		let toggle 	= !window.cursor_visible();
		window.set_cursor_visibility(toggle);
		window.set_cursor_lock_mode(!toggle);
	}
}

pub fn input_misc_system(
		_btn		: Res<Input<MouseButton>>,
		key			: Res<Input<KeyCode>>,
		_game		: Res<GameState>,
	mut	phys_ctx	: ResMut<DebugRenderContext>,
	mut exit		: EventWriter<AppExit>,
	mut query		: Query<&mut FlyCamera>,
) {
	for mut camera in query.iter_mut() {
		if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Space) {
			let toggle = !camera.enabled_follow;
			camera.enabled_follow = toggle;
		}

		if key.just_pressed(KeyCode::Escape) {
			let toggle = !camera.enabled_rotation;
			camera.enabled_rotation = toggle;
		}
	}

	if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Escape) {
		exit.send(AppExit);
	}

	if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Key1) {
		phys_ctx.enabled = !phys_ctx.enabled;
	}
}