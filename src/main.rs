use bevy			::	prelude :: *;
use bevy			::	app::AppExit;
use bevy_rapier3d	::	{ prelude :: * };
use bevy_fly_camera	::	{ FlyCamera, FlyCameraPlugin };
use bevy_egui		::	*;
use bevy_atmosphere	::	*;
use rapier3d		::	dynamics :: { JointAxis };

use serde			::	{ Deserialize, Serialize };

use std				:: 	{ path::PathBuf };

#[macro_use(defer)] extern crate scopeguard;

use bevy::render::mesh::shape as render_shape;

mod gchki_egui;
use gchki_egui		:: *;

#[derive(Component)]
struct NameComponent {
	name : String
}

#[derive(Component, Debug, Copy, Clone, PartialEq)]
enum VehiclePart {
	Wheel,
	Axle,
	Body,
	WheelJoint,
	AxleJoint,
}

#[derive(Component, Debug, Copy, Clone, PartialEq)]
enum SideX {
	Left,
	Center,
	Right
}

#[derive(Component, Debug, Copy, Clone, PartialEq)]
enum SideY {
	Top,
	Center,
	Bottom
}

#[derive(Component, Debug, Copy, Clone, PartialEq)]
enum SideZ {
	Front,
	Center,
	Rear
}

// TODO: all this looks like a bad design, most likely i need a different approach
use usize as WheelSideType;
const FRONT_RIGHT			: WheelSideType = 0;
const FRONT_LEFT			: WheelSideType = 1;
const FRONT_SPLIT			: WheelSideType	= 2;

const REAR_RIGHT			: WheelSideType = 2;
const REAR_LEFT				: WheelSideType = 3;
const REAR_SPLIT			: WheelSideType = 4;

const WHEELS_MAX			: WheelSideType = 4;

fn wheel_side_name(side: WheelSideType) -> &'static str {
	match side {
		  FRONT_RIGHT		=> "Front Right"
		, FRONT_LEFT		=> "Front Left"
		, REAR_RIGHT		=> "Rear Right"
		, REAR_LEFT			=> "Rear Left"
		, _					=> panic!("Only 4 sides are supported currently: 0 - 3 or FrontRight FrontLeft RearRight RearLeft"),
	}
}

fn wheel_side_to_zx(side: WheelSideType) -> (SideZ, SideX) {
	match side {
		FRONT_RIGHT			=> (SideZ::Front, SideX::Right)
	  , FRONT_LEFT			=> (SideZ::Front, SideX::Left)
	  , REAR_RIGHT			=> (SideZ::Rear, SideX::Right)
	  , REAR_LEFT			=> (SideZ::Rear, SideX::Left)
	  , _					=> panic!("Only 4 sides are supported currently: 0 - 3 or FrontRight FrontLeft RearRight RearLeft"),
  }
}

fn wheel_side_from_zx(sidez: SideZ, sidex: SideX) -> WheelSideType {
	match sidez {
		SideZ::Front		=> {
			match sidex {
				SideX::Left => return FRONT_LEFT,
				SideX::Center=> panic!("Only 4 sides are supported currently: 0 - 3 or FrontRight FrontLeft RearRight RearLeft"),
				SideX::Right=> return FRONT_RIGHT,
			};
		},
		SideZ::Center		=> panic!("Only 4 sides are supported currently: 0 - 3 or FrontRight FrontLeft RearRight RearLeft"),
		SideZ::Rear			=> {
			match sidex {
				SideX::Left => return REAR_LEFT,
				SideX::Center=> panic!("Only 4 sides are supported currently: 0 - 3 or FrontRight FrontLeft RearRight RearLeft"),
				SideX::Right=> return REAR_RIGHT,
			}
		}
	}
}

fn wheel_side_offset(side: WheelSideType, off: Vec3) -> Vec3 {
	match side {
		FRONT_RIGHT			=> Vec3::new( off.x, -off.y,  off.z),
		FRONT_LEFT			=> Vec3::new(-off.x, -off.y,  off.z),
		REAR_RIGHT			=> Vec3::new( off.x, -off.y, -off.z),
		REAR_LEFT 			=> Vec3::new(-off.x, -off.y, -off.z),
		WHEELS_MAX			=> panic!("Max shouldn't be used as a wheel side!"),
		_					=> panic!("Only 4 sides are supported currently: 0 - 3 or FrontRight FrontLeft RearRight RearLeft"),
	}
}

const WHEEL_SIDES: &'static [WheelSideType] = &[
	  FRONT_RIGHT
	, FRONT_LEFT
	, REAR_LEFT
	, REAR_RIGHT
];

#[derive(Debug, Clone, Copy)]
struct RespawnableEntity {
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

struct Game {
	  camera					: Option<Entity>
	, body 						: Option<RespawnableEntity>

	, wheels					: [Option<RespawnableEntity>; WHEELS_MAX as usize]
	, axles						: [Option<RespawnableEntity>; WHEELS_MAX as usize]

    , load_veh_dialog			: Option<FileDialog>
    , save_veh_dialog			: Option<FileDialog>

	, save_veh_file				: Option<PathBuf>
	, load_veh_file				: Option<PathBuf>
}

impl Default for Game {
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

// TODO: 41. refactor config: separate physics into a VehiclePhysicsConfig
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
struct PhysicsConfig {
	  density					: f32
	, mass						: f32
	, friction					: f32
	, restitution				: f32
	, lin_damping				: f32
	, ang_damping				: f32
}

impl Default for PhysicsConfig {
	fn default() -> Self {
		Self {
			  density			: 1.0
			, mass				: 0.0 // calculated at runtime
			, friction			: 0.5
			, restitution		: 0.0
			, lin_damping		: 0.0
			, ang_damping		: 0.0
		}
	}
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
struct WheelConfig {
	  hh						: f32
	, r							: f32
	, fixed						: bool
	, density					: f32
	, mass						: f32
	, friction					: f32
	, restitution				: f32
	, lin_damping				: f32
	, ang_damping				: f32
}

impl Default for WheelConfig {
	fn default() -> Self {
		Self {
			  hh				: 0.5
			, r					: 0.8
			, fixed				: false
			, density			: 1.0
			, mass				: 0.0 // calculated at runtime
			, friction			: 0.5
			, restitution		: 0.0
			, lin_damping		: 0.0
			, ang_damping		: 0.0
		}
	}
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
struct AxleConfig {
	  half_size					: Vec3
	, fixed						: bool
	, density					: f32
	, mass						: f32
	, wheel_offset				: Vec3
	, auto_offset				: bool
	, friction					: f32
	, restitution				: f32
	, lin_damping				: f32
	, ang_damping				: f32
}

impl Default for AxleConfig {
	fn default() -> Self {
		Self {
			  half_size			: Vec3::new(0.1, 0.2, 0.1)
			, fixed				: true
			, density			: 1000.0
			, mass				: 0.0
			, wheel_offset		: Vec3::new(0.8, 0.0, 0.0)
			, auto_offset		: true
			, friction			: 0.5
			, restitution		: 0.0
			, lin_damping		: 0.0
			, ang_damping		: 0.0
		}
	}
}

impl AxleConfig {
	fn wheel_offset(self, side: WheelSideType) -> Vec3 {
		wheel_side_offset		(side, self.wheel_offset)
	}
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
struct BodyConfig {
	  half_size					: Vec3
	, density					: f32
	, fixed						: bool
	, axle_offset				: Vec3
	, auto_offset				: bool
	, friction					: f32
	, restitution				: f32
	, lin_damping				: f32
	, ang_damping				: f32
}

impl Default for BodyConfig {
	fn default() -> Self {
		Self {
			  half_size			: Vec3::new(0.5, 0.5, 1.0)
			, density			: 2.0
			, fixed				: true
			, axle_offset		: Vec3::new(0.8, 0.8, 1.4)
			, auto_offset		: true
			, friction			: 0.5
			, restitution		: 0.0
			, lin_damping		: 0.0
			, ang_damping		: 0.0
		}
	}
}

impl BodyConfig {
	fn axle_offset(self, side: WheelSideType) -> Vec3 {
		wheel_side_offset		(side, self.axle_offset)
	}
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
struct AcceleratorConfig {
	  vel_fwd					: f32
	, vel_bwd					: f32
	, damping_fwd				: f32
	, damping_bwd				: f32
	, damping_stop				: f32
}

impl Default for AcceleratorConfig {
	fn default() -> Self {
		Self {
			  vel_fwd			: 10.0
			, vel_bwd			: 7.0
			, damping_fwd		: 100.0
			, damping_bwd		: 100.0
			, damping_stop		: 200.0
		}
	}
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
struct SteeringConfig {
	  stiffness					: f32
	, stiffness_release			: f32
	, damping					: f32
	, damping_release			: f32
	, angle						: f32
}

impl Default for SteeringConfig {
	fn default() -> Self {
		Self {
			  stiffness			: 5000.0
			, stiffness_release	: 10000.0
			, damping			: 300.0
			, damping_release	: 100.0
			, angle				: 20.0
		}
	}
}

#[derive(Default)]
struct DespawnResource {
    entities: Vec<Entity>,
}

fn main() {
	App::new()
		.insert_resource(ClearColor(Color::rgb(
			0xF9 as f32 / 255.0,
			0xF9 as f32 / 255.0,
			0xFF as f32 / 255.0,
		)))
		.insert_resource		(Msaa::default())
		.insert_resource		(Game::default())
		.insert_resource		(DespawnResource::default())
		.insert_resource		(AtmosphereMat::default()) // Default Earth sky

		.add_plugins			(DefaultPlugins)
		.add_plugin				(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin				(RapierDebugRenderPlugin::default())
		.add_plugin				(FlyCameraPlugin)
		.add_plugin				(EguiPlugin)
		.add_plugin				(AtmospherePlugin {
            dynamic				: false,  // Set to false since we aren't changing the sky's appearance
            sky_radius			: 10.0,
        })

		.add_startup_system		(setup_grab_system)
		.add_startup_system		(setup_graphics_system)
		.add_startup_system		(spawn_world_system)
		.add_startup_system_to_stage(StartupStage::PostStartup, setup_camera_system)

		.add_system				(cursor_grab_system)
		.add_system				(toggle_button_system)
		.add_system				(vehicle_controls_system)
		.add_system				(update_ui_system)
		.add_system				(save_vehicle_config_system)
		.add_system				(load_vehicle_config_system)

		.add_system_to_stage	(CoreStage::PostUpdate, display_events_system)
		.add_system_to_stage	(CoreStage::PostUpdate, respawn_vehicle_system)
		.add_system_to_stage	(CoreStage::PostUpdate, despawn_system)
		.run					();
}

fn setup_grab_system(mut windows: ResMut<Windows>) {
	let window = windows.get_primary_mut().unwrap();

	window.set_cursor_lock_mode	(true);
	window.set_cursor_visibility(false);
}

fn setup_graphics_system(
	mut	meshes					: ResMut<Assets<Mesh>>,
	mut	materials				: ResMut<Assets<StandardMaterial>>,
	mut game					: ResMut<Game>,
	mut commands				: Commands,
) {
	const HALF_SIZE: f32		= 100.0;

	commands.spawn_bundle(DirectionalLightBundle {
		directional_light: DirectionalLight {
			illuminance: 10000.0,
			// Configure the projection to better fit the scene
			shadow_projection	: OrthographicProjection {
				left			: -HALF_SIZE,
				right			: HALF_SIZE,
				bottom			: -HALF_SIZE,
				top				: HALF_SIZE,
				near			: -10.0 * HALF_SIZE,
				far				: 100.0 * HALF_SIZE,
				..Default::default()
			},
			shadows_enabled		: true,
			..Default::default()
		},
		transform				: Transform {
			translation			: Vec3::new(10.0, 2.0, 10.0),
			rotation			: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
			..Default::default()
		},
		..Default::default()
	});

	//

	spawn_world_axis	(&mut meshes, &mut materials, &mut commands);

	spawn_camera		(&mut game, &mut commands);
}

fn spawn_camera(
	game				: &mut ResMut<Game>,
	commands			: &mut Commands
) {
	let camera = commands.spawn_bundle(PerspectiveCameraBundle {
			transform: Transform {
				translation: Vec3::new(0., 1., 10.),
				..Default::default()
			},
			..Default::default()
		})
//		.insert			(Collider::ball(1.0))
		.insert			(FlyCamera::default())
		.insert			(NameComponent{ name: "Camera".to_string() })
		.id				();

	game.camera			= Some(camera);
	println!			("camera Entity ID {:?}", camera);
}

fn setup_camera_system(
		game			: ResMut<Game>,
	mut query			: Query<&mut FlyCamera>
) {
	// initialize camera with target to look at
	if game.camera.is_some() && game.body.is_some() {
		let mut camera 	= query.get_mut(game.camera.unwrap()).unwrap();
		camera.target 	= Some(game.body.unwrap().entity);
		println!		("camera.target Entity ID {:?}", camera.target);
	}
}

fn _spawn_gltf(
    mut commands: Commands,
    ass: Res<AssetServer>,
) {
    // note that we have to include the `Scene0` label
    let my_gltf = ass.load("corvette/wheel/corvette_wheel.gltf#Scene0");

    // to be able to position our 3d model:
    // spawn a parent entity with a TransformBundle
    // and spawn our gltf as a scene under it
    commands.spawn_bundle(TransformBundle {
        local: Transform::from_xyz(0.0, 0.0, 0.0),
        global: GlobalTransform::identity(),
    }).with_children(|parent| {
        parent.spawn_scene(my_gltf);
    });
}

fn spawn_world_system(
	mut _configuration	: ResMut<RapierConfiguration>,
	mut game			: ResMut<Game>,
	mut	meshes			: ResMut<Assets<Mesh>>,
	mut	materials		: ResMut<Assets<StandardMaterial>>,
		ass				: Res<AssetServer>,
	mut commands		: Commands
) {
//	configuration.timestep_mode = TimestepMode::VariableTimestep;

	spawn_ground		(&game, &mut meshes, &mut materials, &mut commands);

	if false {
		spawn_cubes		(&mut commands);
	}

	if true {
		spawn_friction_tests(&mut meshes, &mut materials, &mut commands);
	}

	let body_pos 		= Transform::from_xyz(0.0, 5.5, 0.0);
	let body_cfg		= BodyConfig::default();
	let axle_cfg		= AxleConfig::default();
	let wheel_cfg		= WheelConfig::default();
	let accel_cfg		= AcceleratorConfig::default();
	let steer_cfg		= SteeringConfig::default();

	spawn_vehicle(
		  &mut game
		, &mut meshes
		, &mut materials
		, body_cfg
		, accel_cfg
		, steer_cfg
		, axle_cfg
		, wheel_cfg
		, body_pos
		, &ass
		, &mut commands
	);
}

fn spawn_ground(
	_game				: &ResMut<Game>,
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	commands			: &mut Commands
) {
	let ground_size 	= 200.1;
	let ground_height 	= 0.1;

	let ground			= commands
        .spawn			()
		.insert_bundle	(PbrBundle {
			mesh		: meshes.add(Mesh::from(render_shape::Box::new(ground_size, ground_height, ground_size))),
			material	: materials.add(Color::rgb(0.8, 0.8, 0.8).into()),
			transform	: Transform::from_xyz(0.0, -ground_height, 0.0),
			..Default::default()
		})
		.insert			(Collider::cuboid(ground_size, ground_height, ground_size))
        .insert			(Transform::from_xyz(0.0, -ground_height, 0.0))
        .insert			(GlobalTransform::default())
		.id				();
		
	println!			("ground Entity ID {:?}", ground);
}

fn spawn_vehicle(
		game			: &mut ResMut<Game>,
	mut _meshes			: &mut ResMut<Assets<Mesh>>,
	mut _materials		: &mut ResMut<Assets<StandardMaterial>>,
		body_cfg		: BodyConfig, // TODO: use VehicleConfig instead
		accel_cfg		: AcceleratorConfig,
		steer_cfg		: SteeringConfig,
		axle_cfg		: AxleConfig,
		wheel_cfg		: WheelConfig,
		body_pos		: Transform,
		ass				: &Res<AssetServer>,
	mut commands		: &mut Commands
) {
	let body 			= spawn_body(body_pos, body_cfg, accel_cfg, steer_cfg, ass, &mut commands);
	game.body 			= Some(RespawnableEntity { entity : body, ..Default::default() });
	println!			("body Entity ID {:?}", body);

	//println!("body_pos {:?}", body_pos);

	// 0..1 {
	for side_ref in WHEEL_SIDES {
		let side 		= *side_ref;
		let axle_offset = body_cfg.axle_offset(side);
		let wheel_offset= axle_cfg.wheel_offset(side);
		let (axle, wheel) = spawn_attached_wheel(side, body, body_pos, axle_offset, axle_cfg, wheel_cfg, ass, &mut commands);
		game.axles[side] = Some(axle);
		game.wheels[side] = Some(wheel);

	//  	let wheel_model	= ass.load("corvette/wheel/corvette_wheel.gltf#Scene0");

	//  	let 	axle_pos	= body_pos * Transform::from_translation(axle_offset);
	// 	let mut wheel_pos 	= axle_pos * Transform::from_translation(wheel_offset);

	// 	let side_name	= wheel_side_name(side);
	// 	let (sidez, sidex) = wheel_side_to_zx(side);
	// 	let mut axle : Entity = Entity::from_bits(0);
	// 	let mut wheel : Entity = Entity::from_bits(0);
	//  	commands.entity(body)
	// // //		.insert		(body_type)
	//  		.with_children(|parent| {
	// 			axle = parent
	// 			.spawn()
	// 			.insert		(Transform::from_translation(axle_offset))//(axle_pos)
	// 			.insert		(GlobalTransform::default())
	// 			.insert		(RigidBody::Fixed)
	// 			.insert		(axle_cfg)
	// 			.insert		(MassProperties::default())
	// 			.insert		(Damping::default())
	// 			.insert		(NameComponent{ name: format!("{} Axle", side_name) })
	// 			.insert		(VehiclePart::Axle)
	// 			.insert		(sidex)
	// 			.insert		(sidez)

	// 			.with_children(|parent| {
	// 				parent
	// 				.spawn	()
	// 				.insert	(axle_pos)
	// 				.insert	(GlobalTransform::default())
	// 				.insert	(Collider::cuboid(axle_cfg.half_size.x, axle_cfg.half_size.y, axle_cfg.half_size.z))
	// 				.insert	(ColliderMassProperties::Density(axle_cfg.density))
	// 				.insert	(Friction::default())
	// 				.insert	(Restitution::default());
	// 			})

	// 			.with_children(|parent| {
	// 				wheel_pos.rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
	// 				wheel = parent.spawn()
	// 					.insert		(RigidBody::Fixed)
	// 					.insert		(wheel_cfg)
	// 					.insert		(Transform::from_translation(wheel_offset))//(wheel_pos)
	// 					.insert		(GlobalTransform::default())
	// 					.insert		(MassProperties::default())
	// 					.insert		(Damping{ linear_damping: wheel_cfg.lin_damping, angular_damping: wheel_cfg.ang_damping })
	// 					.insert		(NameComponent{ name: format!("{} Wheel", side_name) })
	// 					.insert		(VehiclePart::Wheel)
	// 					.insert		(sidex)
	// 					.insert		(sidez)
	// 					.with_children(|parent| {
	// 						parent
	// 						.spawn	()
	// 						.insert	(wheel_pos) // by default cylinder spawns with its flat surface on the ground and we want the round part
	// 						.insert (GlobalTransform::default())
	// 						.insert	(Collider::cylinder(wheel_cfg.hh, wheel_cfg.r))
	// 						.insert	(ColliderMassProperties::Density(wheel_cfg.density))
	// 						.insert	(Friction::new(wheel_cfg.friction))
	// 						.insert	(Restitution::new(wheel_cfg.restitution))
	// 						.insert	(ActiveEvents::COLLISION_EVENTS);
	// 					})

	// 					.with_children(|parent| {
	// 						parent.spawn_scene(wheel_model);
	// 					}).id();
	//  			}).id();
	//  		});

	// 	{
	// 		let anchor1		= axle_offset;
	// 		let anchor2 	= Vec3::ZERO;
	// 		spawn_axle_joint(body, axle, anchor1, anchor2, &mut commands);
	// 	}

	// 	{
	// 		let anchor1		= wheel_offset;
	// 		let anchor2 	= Vec3::ZERO;
	// 		spawn_wheel_joint(axle, wheel, anchor1, anchor2, &mut commands);
	// 	}

	// 	game.axles[side] = Some(RespawnableEntity{ entity : axle,	..Default::default() });
	// 	game.wheels[side] = Some(RespawnableEntity{ entity : wheel,	..Default::default() });
		 

	// 	println!("{} axle_offset {} axle_pos {:?}", side, axle_offset, axle_pos);

		println!		("{} Wheel spawned! {:?}", wheel_side_name(side), game.wheels[side]);
	}
}

fn spawn_attached_wheel(
	side			: WheelSideType,
	body			: Entity,
	body_pos		: Transform,
	axle_offset		: Vec3,
	axle_cfg		: AxleConfig,
	wheel_cfg		: WheelConfig,
	ass				: &Res<AssetServer>,
	commands		: &mut Commands
) -> (RespawnableEntity, RespawnableEntity) { // axle + wheel 
	let (axle, axle_pos) = spawn_axle_with_joint(side, body, body_pos, axle_offset, axle_cfg, ass, commands);

	let wheel_offset = axle_cfg.wheel_offset(side);

	let wheel		= spawn_wheel_with_joint(side, axle, axle_pos, wheel_offset, wheel_cfg, ass, commands);

	let wheel_pos	= axle_pos * wheel_offset;

	println!("wheel {} body_pos {:?} axle_offset {} wheel_offset {}", side, body_pos, axle_offset, wheel_offset);
	println!("wheel {} axle_pos {:?} wheel_pos {:?}", side, axle_pos, wheel_pos);

	(
	RespawnableEntity{ entity : axle,	..Default::default() },
	RespawnableEntity{ entity : wheel, 	..Default::default() }
	)
}

fn spawn_axle_with_joint(
	side			: WheelSideType,
	body			: Entity,
	body_pos		: Transform,
	offset			: Vec3,
	cfg				: AxleConfig,
	ass				: &Res<AssetServer>,
	mut	commands	: &mut Commands
) -> (Entity, Transform) {
	let axle		= spawn_axle(side, body, body_pos, offset, cfg, ass, &mut commands);
	let axle_pos	= body_pos * Transform::from_translation(offset);

	let anchor1		= offset;
	let anchor2 	= Vec3::ZERO;
	spawn_axle_joint(body, axle, anchor1, anchor2, &mut commands);

	(axle, axle_pos)
}

fn spawn_wheel_with_joint(
	side			: WheelSideType,
	axle			: Entity,
	axle_pos		: Transform,
	offset			: Vec3,
	cfg				: WheelConfig,
	ass				: &Res<AssetServer>,
	mut	commands	: &mut Commands
) -> Entity {
	let wheel 		= spawn_wheel(
		  side
		, axle
		, axle_pos
		, offset
		, cfg
		, ass
		, &mut commands
	);

	let anchor1		= offset;
	let anchor2 	= Vec3::ZERO;
	spawn_wheel_joint(axle, wheel, anchor1, anchor2, &mut commands);

	wheel
}

fn spawn_axle(
	side			: WheelSideType,
	body			: Entity,
	body_pos		: Transform,
	offset			: Vec3,
	cfg				: AxleConfig,
	_ass			: &Res<AssetServer>,
	commands		: &mut Commands,
) -> Entity {
	let side_name	= wheel_side_name(side);
	let (sidez, sidex) = wheel_side_to_zx(side);

	let mut axle_id = Entity::from_bits(0);
	let 	axle_pos= body_pos * Transform::from_translation(offset);

	commands
	.entity			(body)
	.with_children(|parent| {
		axle_id = parent
		.spawn		()

		.insert		(if cfg.fixed { RigidBody::Fixed } else { RigidBody::Dynamic })
		.insert		(cfg)
		.insert		(Transform::default())
		.insert		(GlobalTransform::default())
		.insert		(MassProperties::default())
		.insert		(Damping::default())
		.insert		(NameComponent{ name: format!("{} Axle", side_name) })
		.insert		(VehiclePart::Axle)
		.insert		(sidex)
		.insert		(sidez)
		

		.with_children(|parent| {
			parent
			.spawn	()
			.insert	(axle_pos) // Collider requires Transfrom in world space
			.insert	(GlobalTransform::default())
			.insert	(Collider::cuboid(cfg.half_size.x, cfg.half_size.y, cfg.half_size.z))
			.insert	(ColliderMassProperties::Density(cfg.density))
			.insert	(Friction::default())
			.insert	(Restitution::default());
		})
		.id			()
	});

	let axis_cube	= _ass.load("utils/axis_cube.gltf#Scene0");
	commands.spawn_bundle(
		TransformBundle {
			local: axle_pos,
			global: GlobalTransform::default(),
	}).with_children(|parent| {
		parent.spawn_scene(axis_cube);
	});

	axle_id
}

fn spawn_wheel(
	side			: WheelSideType,
	axle			: Entity,
	axle_pos		: Transform,
	offset			: Vec3,
	cfg				: WheelConfig,
	ass				: &Res<AssetServer>,
	commands		: &mut Commands,
) -> Entity {
	let side_name	= wheel_side_name(side);
	let (sidez, sidex) = wheel_side_to_zx(side);
	let mut wheel_id = Entity::from_bits(0);

	let local_pos	= Transform {
		  translation : offset
		, rotation 	: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2) // by default cylinder spawns with its flat surface on the ground and we want the round part
		,..Default::default()
	}; 
	let wheel_pos 	= axle_pos * local_pos;

//	let wheel_model	= ass.load("corvette/wheel/corvette_wheel.gltf#Scene0");

	commands
	.entity			(axle)
	.with_children	(|parent| {
		wheel_id = parent.spawn()
		.insert		(if cfg.fixed { RigidBody::Fixed } else { RigidBody::Dynamic })
		.insert		(cfg)
		.insert		(Transform::default())//from_translation(offset))
		.insert		(GlobalTransform::default())
		.insert		(MassProperties::default())
		.insert		(Damping{ linear_damping: cfg.lin_damping, angular_damping: cfg.ang_damping })
		.insert		(NameComponent{ name: format!("{} Wheel", side_name) })
		.insert		(VehiclePart::Wheel)
		.insert		(sidex)
		.insert		(sidez)
		// collider
		.with_children(|parent| {
			parent.spawn()
			.insert	(wheel_pos)
			.insert (GlobalTransform::default())
			.insert	(Collider::cylinder(cfg.hh, cfg.r))
			.insert	(ColliderMassProperties::Density(cfg.density))
			.insert	(Friction::new(cfg.friction))
			.insert	(Restitution::new(cfg.restitution))
			.insert	(ActiveEvents::COLLISION_EVENTS);
		})
		// render model
		// .with_children(|parent| {
		// 	parent.spawn_bundle(
		// 		TransformBundle {
		// 			local: Transform::default(),
		// 			global: GlobalTransform::default(),
		// 	}).with_children(|parent| {
		// 		parent.spawn_scene(wheel_model);
		// 	});
		// })
		.id			()
	});

	let axis_cube	= ass.load("utils/axis_cube.gltf#Scene0");
	commands.spawn_bundle(
		TransformBundle {
			local: wheel_pos,
			global: GlobalTransform::default(),
	}).with_children(|parent| {
		parent.spawn_scene(axis_cube);
	});

	wheel_id
}

fn spawn_axle_joint(
	entity1			: Entity,
	entity2			: Entity,
	anchor1			: Vec3,
	anchor2			: Vec3,
	commands		: &mut Commands,
) {
	let axle_joint = RevoluteJointBuilder::new(Vec3::Y)
		.local_anchor1(anchor1)
		.local_anchor2(anchor2)
		.limits		([0.000001, 0.000001]);

	commands
		.entity		(entity2)
		.insert		(ImpulseJoint::new(entity1, axle_joint));
}

fn spawn_wheel_joint(
	entity1			: Entity,
	entity2			: Entity,
	anchor1			: Vec3,
	anchor2			: Vec3,
	commands		: &mut Commands,
) {
	let wheel_joint = RevoluteJointBuilder::new(Vec3::X)
		.local_anchor1(anchor1)
		.local_anchor2(anchor2);

	commands
		.entity		(entity2)
		.insert		(ImpulseJoint::new(entity1, wheel_joint));
}

fn spawn_body(
	pos				: Transform,
	cfg				: BodyConfig,
	accel_cfg		: AcceleratorConfig,
	steer_cfg		: SteeringConfig,
	ass				: &Res<AssetServer>,
	commands		: &mut Commands,
) -> Entity {
	let body_type	= if cfg.fixed { RigidBody::Fixed } else { RigidBody::Dynamic };
	let half_size	= cfg.half_size;
	let density		= cfg.density;

//	let body_model	= ass.load("corvette/body/corvette_body.gltf#Scene0");

	commands
		.spawn		()
		.insert_bundle(TransformBundle::default())
		.insert		(body_type)
		.insert		(cfg)
		.insert		(accel_cfg)
		.insert		(steer_cfg)
		.insert		(MassProperties::default())
		.insert		(Damping::default())
		.insert		(NameComponent{ name: "Body".to_string() })
		.insert		(VehiclePart::Body)
		.insert		(SideX::Center)
		.insert		(SideZ::Center)
		.with_children(|children| {
		children.spawn()
			.insert	(pos)
			.insert	(GlobalTransform::default())
			.insert	(Collider::cuboid(half_size.x, half_size.y, half_size.z))
			.insert	(ColliderMassProperties::Density(density)) // joints like it when there is an hierarchy of masses and we want body to be the heaviest
			.insert	(Friction::default())
			.insert	(Restitution::default());
		})
		// .with_children(|children| {
		// children.spawn_bundle(
		// 	TransformBundle {
		// 		local: Transform::from_xyz(0.0, -1.0, 0.0),
		// 		global: GlobalTransform::default(),
		// 	}).with_children(|parent| {
		// 		parent.spawn_scene(body_model);
		// 	});
		// })
		.id			()
}

fn spawn_cubes(commands: &mut Commands) {
	let num = 8;
	let rad = 1.0;

	let shift = rad * 2.0 + rad;
	let centerx = shift * (num / 2) as f32;
	let centery = shift / 2.0;
	let centerz = shift * (num / 2) as f32;

	let mut offset = -(num as f32) * (rad * 2.0 + rad) * 0.5;
	let mut color = 0;
	let colors = [
        Color::hsl(220.0, 1.0, 0.3),
        Color::hsl(180.0, 1.0, 0.3),
        Color::hsl(260.0, 1.0, 0.7),
    ];

	for j in 0usize..20 {
		for i in 0..num {
			for k in 0usize..num {
				let x = i as f32 * shift - centerx + offset;
				let y = j as f32 * shift + centery + 3.0;
				let z = k as f32 * shift - centerz + offset;
				color += 1;

				commands
					.spawn()
					.insert(RigidBody::Dynamic)
					.insert(Transform::from_xyz(x, y, z))
					.insert(GlobalTransform::default())
					.insert(Collider::cuboid(rad, rad, rad))
					.insert(ColliderDebugColor(colors[color % 3]));
			}
		}

		offset -= 0.05 * rad * (num as f32 - 1.0);
	}
}

fn spawn_friction_tests(
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	commands			: &mut Commands
) {
	let num = 5;
	let offset = Vec3::new(0.0, 0.0, 3.0);
	let line_hsize = Vec3::new(5.0, 0.025, 30.0);

	for i in 0..=num {
		let mut pos = offset.clone();
		pos.x = i as f32 * ((line_hsize.x * 2.0) + 0.5);

		let friction = i as f32 * (1.0 / num as f32); // so that when i == num => friction == 1
		let friction_inv = 1.0 - friction;
		let color = Color::rgb(friction_inv, friction_inv, friction_inv);

		println!("fr {} fri {} c {:?}", friction, friction_inv, color);

		commands
			.spawn_bundle(PbrBundle {
				mesh		: meshes.add			(Mesh::from(render_shape::Box::new(line_hsize.x * 2.0, line_hsize.y * 2.0, line_hsize.z * 2.0))),
				material	: materials.add			(color.into()),
				..Default::default()
			})
			.insert(RigidBody::Fixed)
			.insert(Transform::from_translation(pos))
			.insert(GlobalTransform::default())
			.insert(Collider::cuboid(line_hsize.x, line_hsize.y, line_hsize.z))
			.insert(Friction{ coefficient : friction, combine_rule : CoefficientCombineRule::Average });
//			.insert(ColliderDebugColor(color));
	}
}

fn spawn_world_axis(
	meshes			: &mut ResMut<Assets<Mesh>>,
	materials		: &mut ResMut<Assets<StandardMaterial>>,
	commands		: &mut Commands,
) {
	// X
	commands.spawn_bundle(PbrBundle {
		mesh		: meshes.add			(Mesh::from(render_shape::Box::new(1.0, 0.1, 0.1))),
		material	: materials.add			(Color::rgb(0.8, 0.1, 0.1).into()),
		transform	: Transform::from_xyz	(0.5, 0.0 + 0.05, 0.0),
		..Default::default()
	});
	// Y
	commands.spawn_bundle(PbrBundle {
		mesh		: meshes.add			(Mesh::from(render_shape::Box::new(0.1, 1.0, 0.1))),
		material	: materials.add			(Color::rgb(0.1, 0.8, 0.1).into()),
		transform	: Transform::from_xyz	(0.0, 0.5 + 0.05, 0.0),
		..Default::default()
	});
	// Z
	commands.spawn_bundle(PbrBundle {
		mesh		: meshes.add			(Mesh::from(render_shape::Box::new(0.1, 0.1, 1.0))),
		material	: materials.add			(Color::rgb(0.1, 0.1, 0.8).into()),
		transform	: Transform::from_xyz	(0.0, 0.0 + 0.05, 0.5),
		..Default::default()
	});
}

fn cursor_grab_system(
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

fn toggle_button_system(
		_btn		: Res<Input<MouseButton>>,
		key			: Res<Input<KeyCode>>,
		_game		: Res<Game>,
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

		if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Escape) {
			exit.send(AppExit);
		}
	}
}

fn motor_velocity(
	velocity		: f32,
	factor			: f32,
	joint_e			: Option<RespawnableEntity>,
	query			: &mut Query<&mut ImpulseJoint>
) {
	match joint_e {
		Some(j) => {
			let mut	joint	= query.get_mut(j.entity).unwrap();
			joint.data.set_motor_velocity(JointAxis::AngX, velocity, factor);
		}
		_ => ()
	}
}

fn motor_steer(
	angle			: f32,
	stiffness		: f32,
	damping			: f32,
	joint_re		: Option<RespawnableEntity>,
	query			: &mut Query<&mut ImpulseJoint>) {
	match joint_re {
		Some(re) => {
			let mut joint 	= query.get_mut(re.entity).unwrap();
			let	angle_rad	= angle.to_radians();
			joint.data.set_motor_position(JointAxis::AngX, angle_rad, stiffness, damping);
			
			if angle.abs() > 0.0001 {
				joint.data.set_limits(JointAxis::AngX, [-angle_rad.abs(), angle_rad.abs()]);// [-3.14, 3.14]);
			}
		}
		_ => ()
	}
}

fn vehicle_controls_system(
		key			: Res<Input<KeyCode>>,
		game		: ResMut<Game>,
		q_accel_cfg	: Query<&AcceleratorConfig>,
		q_steer_cfg	: Query<&SteeringConfig>,
	mut	query		: Query<&mut ImpulseJoint>,
) {
	let fr_axle_joint	= game.axles[FRONT_RIGHT];
	let fl_axle_joint	= game.axles[FRONT_LEFT];

	let rr_wheel_joint	= game.wheels[REAR_RIGHT];
	let rl_wheel_joint	= game.wheels[REAR_LEFT];

	let accel_cfg = match q_accel_cfg.get(game.body.unwrap().entity) {
		Ok(c) => c,
		Err(_) => return,
	};

	let steer_cfg = match q_steer_cfg.get(game.body.unwrap().entity) {
		Ok(c) => c,
		Err(_) => return,
	};

	if key.just_pressed(KeyCode::W) {
		motor_velocity	(accel_cfg.vel_fwd, accel_cfg.damping_fwd, rr_wheel_joint, &mut query);
		motor_velocity	(accel_cfg.vel_fwd, accel_cfg.damping_fwd, rl_wheel_joint, &mut query);
	} else if key.just_released(KeyCode::W) {
		motor_velocity	(0.0, accel_cfg.damping_stop, rr_wheel_joint, &mut query);
		motor_velocity	(0.0, accel_cfg.damping_stop, rl_wheel_joint, &mut query);
	}
	
	if key.just_pressed(KeyCode::S) {
		motor_velocity	(-accel_cfg.vel_bwd, accel_cfg.damping_bwd, rr_wheel_joint, &mut query);
		motor_velocity	(-accel_cfg.vel_bwd, accel_cfg.damping_bwd, rl_wheel_joint, &mut query);
	} else if key.just_released(KeyCode::S) {
		motor_velocity	(0.0, accel_cfg.damping_stop, rr_wheel_joint, &mut query);
		motor_velocity	(0.0, accel_cfg.damping_stop, rl_wheel_joint, &mut query);
	}
 
	let steer_angle 	= steer_cfg.angle;
	let stiffness 		= steer_cfg.stiffness;
	let stiffness_release = steer_cfg.stiffness_release;
	let damping 		= steer_cfg.damping;
	let damping_release = steer_cfg.damping_release;
	if key.just_pressed(KeyCode::D) {
		motor_steer		(-steer_angle, stiffness, damping, fr_axle_joint, &mut query);
		motor_steer		(-steer_angle, stiffness, damping, fl_axle_joint, &mut query);
	} else if key.just_released(KeyCode::D) {
		motor_steer		(0.0, stiffness_release, damping_release, fr_axle_joint, &mut query);
		motor_steer		(0.0, stiffness_release, damping_release, fl_axle_joint, &mut query);
	}

 	if key.just_pressed(KeyCode::A) {
		motor_steer		(steer_angle, stiffness, damping, fr_axle_joint, &mut query);
		motor_steer		(steer_angle, stiffness, damping, fl_axle_joint, &mut query);
	} else if key.just_released(KeyCode::A) {
		motor_steer		(0.0, stiffness_release, damping_release, fr_axle_joint, &mut query);
		motor_steer		(0.0, stiffness_release, damping_release, fl_axle_joint, &mut query);
	}
}

fn display_events_system(
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

use bevy_egui::egui::{Slider, Ui};
use bevy_egui::{egui, EguiContext};

fn set_cylinder_hh(
	new_hh: f32,
	shared_shape: &mut Mut<Collider>,
) {
	let 	shape 	= shared_shape.raw.make_mut();
	let mut cylinder= shape.as_cylinder_mut().unwrap();
	cylinder.half_height = new_hh;
}

fn set_cylinder_r(
	new_r: f32,
	shared_shape: &mut Mut<Collider>,
) {
	let 	shape 	= shared_shape.raw.make_mut();
	let mut cylinder= shape.as_cylinder_mut().unwrap();
	cylinder.radius = new_r;
}

fn set_box_half_size(
	new_hs: Vec3,
	shared_shape: &mut Mut<Collider>,
) {
	let 	shape 	= shared_shape.raw.make_mut();
	let mut cuboid	= shape.as_cuboid_mut().unwrap();
	cuboid.half_extents	= new_hs.into();
}

fn set_density(
		density_in			: f32,
	mut mass_props_co		: &mut Mut<ColliderMassProperties>,
) {
	match &mut mass_props_co as &mut ColliderMassProperties {
		ColliderMassProperties::Density(density) => {
			*density 		= density_in;
			**mass_props_co = ColliderMassProperties::Density(*density);
		},
		ColliderMassProperties::MassProperties(_) => (),
	};
}

fn set_friction(
		friction_in			: f32,
	mut friction			: &mut Mut<Friction>,
) {
	friction.as_mut().coefficient = friction_in;
}

fn set_restitution(
		restitution_in		: f32,
	mut restitution			: &mut Mut<Restitution>,
) {
	restitution.as_mut().coefficient = restitution_in;
}

fn set_damping(
	lin_damping_in			: f32,
	ang_damping_in			: f32,
mut damping					: &mut Mut<Damping>,
) {
	damping.as_mut().linear_damping = lin_damping_in;
	damping.as_mut().angular_damping = ang_damping_in;
}

fn draw_density_param_ui(
		ui					: &mut Ui,
		name				: &String,
		range				: [f32; 2],
		density				: &mut f32,
		mass				: f32,
) -> bool {
	ui.add(
		Slider::new			(density, std::ops::RangeInclusive::new(range[0], range[1])).text(format!("{} Density (Mass {:.3})", name, mass))
	).changed()
}

fn draw_body_params_ui_collapsing(
	ui						: &mut Ui,
	name					: &String,
	density_range			: [f32; 2],
	mass					: f32,
	cfg						: &mut BodyConfig,
	section_name			: &str
) -> bool {
	let mut changed			= false;		
	let cache				= cfg.clone();	

	ui.collapsing(section_name, |ui| {
		ui.vertical(|ui| {
			changed			|= draw_density_param_ui(ui, name, density_range, &mut cfg.density, mass);

			ui.separator	();

			changed 		|= ui.add(
				Slider::new(&mut cfg.half_size[0], 0.05 ..= 5.0).text("Half Size X"),
			).changed();
			
			changed 		|= ui.add(
				Slider::new(&mut cfg.half_size[1], 0.05 ..= 5.0).text("Half Size Y"),
			).changed();

			changed 		|= ui.add(
				Slider::new(&mut cfg.half_size[2], 0.05 ..= 5.0).text("Half Size Z"),
			).changed();

			ui.checkbox		(&mut cfg.auto_offset, "Auto Offset Axle");

			if changed && cfg.auto_offset {
				let delta 	= cfg.half_size - cache.half_size;
				cfg.axle_offset += delta;
			}

			ui.separator	();

			changed 		|= ui.add(
				Slider::new(&mut cfg.axle_offset[0], 0.05 ..= 5.0).text("Axle Offset X"),
			).changed();
			
			changed 		|= ui.add(
				Slider::new(&mut cfg.axle_offset[1], 0.05 ..= 5.0).text("Axle Offset Y"),
			).changed();

			changed 		|= ui.add(
				Slider::new(&mut cfg.axle_offset[2], 0.05 ..= 5.0).text("Axle Offset Z"),
			).changed();

			ui.separator	();

			changed |= ui.add(
				Slider::new(&mut cfg.lin_damping, 0.0 ..= 100.0).text("Linear Damping"),
			).changed();
		
			changed |= ui.add(
				Slider::new(&mut cfg.ang_damping, 0.0 ..= 100.0).text("Angular Damping"),
			).changed();

			ui.separator	();

			changed			|= ui.checkbox(&mut cfg.fixed, "Fixed").changed();
		}); // ui.vertical
	}); // ui.collapsing

	changed
}

fn draw_axle_params_ui(
	ui						: &mut Ui
  , cfg						: &mut AxleConfig
  , section_name			: String
) -> bool {

	let mut changed			= false;

	ui.collapsing(section_name, |ui| {
	ui.vertical(|ui| {

	let cache = cfg.clone();

	changed |= ui.add(
		Slider::new(&mut cfg.half_size[0], 0.05 ..= 5.0).text("Half Size X"),
	).changed();
	
	changed |= ui.add(
		Slider::new(&mut cfg.half_size[1], 0.05 ..= 5.0).text("Half Size Y"),
	).changed();

	changed |= ui.add(
		Slider::new(&mut cfg.half_size[2], 0.05 ..= 5.0).text("Half Size Z"),
	).changed();

	ui.checkbox(&mut cfg.auto_offset, "Auto Offset Wheel");

	if changed && cfg.auto_offset {
		let mut delta		= cfg.half_size - cache.half_size;
		// we care only for x axis
		delta				*= Vec3::X;
		cfg.wheel_offset 	+= delta;
	}

	ui.separator();

	changed |= ui.add(
		Slider::new(&mut cfg.wheel_offset[0], 0.05 ..= 5.0).text("Wheel Offset X"),
	).changed();
	
	changed |= ui.add(
		Slider::new(&mut cfg.wheel_offset[1], 0.05 ..= 5.0).text("Wheel Offset Y"),
	).changed();

	changed |= ui.add(
		Slider::new(&mut cfg.wheel_offset[2], 0.05 ..= 5.0).text("Wheel Offset Z"),
	).changed();

	ui.separator();

	changed |= ui.add(
		Slider::new(&mut cfg.density, 0.05 ..= 10000.0)
			.text(format!("Axle Density (Mass: {:.3})", cfg.mass)),
	).changed();

	}); // ui.vertical
	}); // ui.collapsing

	changed
}

fn draw_wheel_params_ui(
	  ui					: &mut Ui
	, cfg					: &mut WheelConfig
	, section_name			: String
) -> bool {

	let mut changed	= false;

	ui.collapsing(section_name, |ui| {
	ui.vertical(|ui| {

	changed |= ui.add(
		Slider::new(&mut cfg.r, 0.05 ..= 2.0).text("Radius"),
	).changed();

	changed |= ui.add(
		Slider::new(&mut cfg.hh, 0.05 ..= 2.0).text("Half Height"),
	).changed();

	changed |= ui.add(
		Slider::new(&mut cfg.density, 0.05 ..= 100.0)
			.text(format!("Wheel Density (Mass: {:.3})", cfg.mass)),
	).changed();

	changed |= ui.add(
		Slider::new(&mut cfg.friction, 0.0 ..= 1.0).text("Friction"),
	).changed();

	changed |= ui.add(
		Slider::new(&mut cfg.restitution, 0.0 ..= 1.0).text("Restitution"),
	).changed();

	changed |= ui.add(
		Slider::new(&mut cfg.lin_damping, 0.0 ..= 100.0).text("Linear Damping"),
	).changed();

	changed |= ui.add(
		Slider::new(&mut cfg.ang_damping, 0.0 ..= 100.0).text("Angular Damping"),
	).changed();

	}); // ui.vertical
	}); // ui.collapsing

	changed
}

fn draw_acceleration_params_ui(
	ui				: &mut Ui,
	accel_cfg		: &mut AcceleratorConfig,
) -> bool {
	let mut changed = false;

	ui.collapsing("Acceleration".to_string(), |ui| {
	ui.vertical(|ui| {

	changed |= ui.add(
		Slider::new(&mut accel_cfg.vel_fwd, 0.05 ..= 400.0).text("Target Speed Forward"),
	).changed();

	changed |= ui.add(
		Slider::new(&mut accel_cfg.damping_fwd, 0.05 ..= 1000.0).text("Acceleration Damping Forward"),
	).changed();

	ui.add_space(1.0);
	
	changed |= ui.add(
		Slider::new(&mut accel_cfg.vel_bwd, 0.05 ..= 400.0).text("Target Speed Backward"),
	).changed();
	changed |= ui.add(
		Slider::new(&mut accel_cfg.damping_bwd, 0.05 ..= 1000.0).text("Acceleration Damping Backward"),
	).changed();

	ui.add_space(1.0);

	changed |= ui.add(
		Slider::new(&mut accel_cfg.damping_stop, 0.05 ..= 1000.0).text("Stopping Damping"),
	).changed();

	}); // ui.vertical
	}); // ui.collapsing

	changed
}

fn draw_steering_params_ui(
	ui				: &mut Ui,
	steer_cfg		: &mut SteeringConfig,
) -> bool{
	let mut changed = false;

	ui.collapsing("Steering".to_string(), |ui| {
	ui.vertical(|ui| {

	changed |= ui.add(
		Slider::new(&mut steer_cfg.angle, 0.05 ..= 180.0).text("Steering Angle"),
	).changed();
	changed |= ui.add(
		Slider::new(&mut steer_cfg.damping, 0.05 ..= 10000.0).text("Steering Damping"),
	).changed();
	changed |= ui.add(
		Slider::new(&mut steer_cfg.damping_release, 0.05 ..= 10000.0).text("Steering Release Damping"),
	).changed();
	changed |= ui.add(
		Slider::new(&mut steer_cfg.stiffness, 0.05 ..= 100000.0).text("Steering Stiffness"),
	).changed();
	changed |= ui.add(
		Slider::new(&mut steer_cfg.stiffness_release, 0.05 ..= 100000.0).text("Steering Release Stiffness"),
	).changed();

	}); // ui.vertical
	}); // ui.collapsing

	changed
}

fn update_ui_system(
	mut ui_context	: ResMut<EguiContext>,
	mut	game		: ResMut<Game>,

	mut q_child		: Query<(
		&Parent,
		&mut Collider,
		&mut ColliderMassProperties,
		&mut Friction,
		&mut Restitution,
	)>,
    mut	q_parent	: Query<(
		&VehiclePart,
		&SideZ,
		&NameComponent,
		&MassProperties,
		&mut Damping,
	)>,
	mut q_body_cfg	: Query<
		&mut BodyConfig
	>,
	mut q_wheel_cfg	: Query<(
		&mut WheelConfig,
		&SideX,
		&SideZ
	)>,
	mut q_axle_cfg	: Query<(
		&mut AxleConfig,
		&SideX,
		&SideZ
	)>,
	mut q_accel_cfg	: Query<
		&mut AcceleratorConfig
	>,
	mut q_steer_cfg	: Query<
		&mut SteeringConfig
	>,
) {
	let body			= game.body.unwrap().entity;
	let mut body_cfg	= q_body_cfg.get_mut(body).unwrap();
	let mut accel_cfg	= q_accel_cfg.get_mut(body).unwrap();
	let mut steer_cfg	= q_steer_cfg.get_mut(body).unwrap();

	let window 			= egui::Window::new("Parameters");
	//let out = 
	window.show(ui_context.ctx_mut(), |ui| {
		ui.horizontal(|ui| {
			if ui.add(toggle_switch::toggle(&mut body_cfg.fixed))
				.on_hover_text("Put vehicle in the air and keep it fixed there.")
				.clicked()
			{
				game.body 			= Some(RespawnableEntity{ entity : game.body.unwrap().entity, respawn: true });
			}
			ui.label("Lifted Car Mode");
		});
		
		ui.separator();
		return;

		// common wheel/axle configs
		// ^^
		let (fl_wheel, _, _)		= q_wheel_cfg.get(game.wheels[FRONT_LEFT].unwrap().entity).unwrap();
		let (rl_wheel, _, _)		= q_wheel_cfg.get(game.wheels[REAR_LEFT].unwrap().entity).unwrap();
		let (fl_axle, _, _)			= q_axle_cfg.get(game.axles[FRONT_LEFT].unwrap().entity).unwrap();
		let (rl_axle, _, _)			= q_axle_cfg.get(game.axles[REAR_LEFT].unwrap().entity).unwrap();

		let mut front_wheel_common 	= fl_wheel.clone();
		let mut rear_wheel_common 	= rl_wheel.clone();
		let mut front_axle_common 	= fl_axle.clone();
		let mut rear_axle_common 	= rl_axle.clone();

		let front_wheels_changed 	=
			draw_wheel_params_ui	(ui, &mut front_wheel_common, String::from("Front Wheels"));

		let rear_wheels_changed		=
			draw_wheel_params_ui	(ui, &mut rear_wheel_common, String::from("Rear Wheels"));

		let front_axles_changed 	=
			draw_axle_params_ui		(ui, &mut front_axle_common, String::from("Front Axles"));

		let rear_axles_changed		=
			draw_axle_params_ui		(ui, &mut rear_axle_common, String::from("Rear Axles"));

		for (mut wheel_cfg, _sidex, sidez) in q_wheel_cfg.iter_mut() {
			if *sidez == SideZ::Front {
				*wheel_cfg.as_mut() = front_wheel_common;
			} else if *sidez == SideZ::Rear {
				*wheel_cfg.as_mut() = rear_wheel_common;
			}
		}

		for (mut axle_cfg, _sidex, sidez) in q_axle_cfg.iter_mut() {
			if *sidez == SideZ::Front {
				*axle_cfg.as_mut() = front_axle_common;
			}
			if *sidez == SideZ::Rear {
				*axle_cfg.as_mut() = rear_axle_common;
			}
		}

		let writeback_axle_collider = |
			  cfg					: &AxleConfig
			, collider				: &mut Mut<Collider>
			, mass_props_co			: &mut Mut<ColliderMassProperties>
		| {
			set_box_half_size		(cfg.half_size, collider);
			set_density				(cfg.density, mass_props_co);
		};

		let writeback_wheel_collider = |
			  cfg					: &WheelConfig
			, collider				: &mut Mut<Collider>
			, mass_props_co			: &mut Mut<ColliderMassProperties>
			, friction				: &mut Mut<Friction>
			, restitution			: &mut Mut<Restitution>
			, damping				: &mut Mut<Damping>
		| {
			set_cylinder_hh			(cfg.hh, collider);
			set_cylinder_r			(cfg.r, collider);
			set_density				(cfg.density, mass_props_co);
			set_friction			(cfg.friction, friction);
			set_restitution			(cfg.restitution, restitution);
			set_damping				(cfg.lin_damping, cfg.ang_damping, damping);
		};

		// write changes back to physics + per component ui 
		for (parent, mut collider, mut mass_props_co, mut friction, mut restitution) in q_child.iter_mut() {
			let (vehicle_part, sidez, name_comp, mass_props_rb, mut damping) = q_parent.get_mut(parent.0).unwrap();
			let name 				= &name_comp.name;
			let vp 					= *vehicle_part;

			let mut body_changed	= false;
			let 	body_cfg_cache 	= body_cfg.clone();

			if vp == VehiclePart::Body {
				let mass			= mass_props_rb.mass;
				body_changed 		= draw_body_params_ui_collapsing(ui, name, [0.05, 100.0], mass, body_cfg.as_mut(), "Body");

				draw_acceleration_params_ui	(ui, accel_cfg.as_mut());
				draw_steering_params_ui		(ui, steer_cfg.as_mut());
			} else if vp == VehiclePart::Wheel && *sidez == SideZ::Front && front_wheels_changed {
				writeback_wheel_collider(&front_wheel_common, &mut collider, &mut mass_props_co, &mut friction, &mut restitution, &mut damping);
			} else if vp == VehiclePart::Wheel && *sidez == SideZ::Rear && rear_wheels_changed {
				writeback_wheel_collider(&rear_wheel_common, &mut collider, &mut mass_props_co, &mut friction, &mut restitution, &mut damping);
			} else if vp == VehiclePart::Axle && *sidez == SideZ::Front && front_axles_changed {
				writeback_axle_collider(&front_axle_common, &mut collider, &mut mass_props_co);
			} else if vp == VehiclePart::Axle && *sidez == SideZ::Rear && rear_axles_changed {
				writeback_axle_collider(&rear_axle_common, &mut collider, &mut mass_props_co);
			}

			// FIXME: dont rewrite every frame here and below
			if vp == VehiclePart::Wheel {
				let (mut wheel_cfg, _, _) = q_wheel_cfg.get_mut(parent.0).unwrap();
				wheel_cfg.mass		= mass_props_rb.mass;
			}

			if vp == VehiclePart::Axle {
				let (mut axle_cfg, _, _) = q_axle_cfg.get_mut(parent.0).unwrap();
				axle_cfg.mass		= mass_props_rb.mass;
			}

			if front_axles_changed || rear_axles_changed {
				// respawn child wheel
				for side_ref in WHEEL_SIDES {
					let side 		= *side_ref;
					game.wheels[side] = Some(RespawnableEntity{ entity : game.wheels[side].unwrap().entity, respawn: true });
				}
			}

			if body_changed {
				*mass_props_co	 	= ColliderMassProperties::Density(body_cfg.density);

				damping.as_mut().linear_damping = body_cfg.lin_damping;
				damping.as_mut().angular_damping = body_cfg.ang_damping;

				let cuboid 			= collider.as_cuboid_mut().unwrap();
				cuboid.raw.half_extents = body_cfg.half_size.into();

				for side_ref in WHEEL_SIDES {
					let side 		= *side_ref;
					// respawn
					game.axles[side] = Some(RespawnableEntity{ entity : game.axles[side].unwrap().entity, respawn: true }); // TODO: hide the ugly
					game.wheels[side] = Some(RespawnableEntity{ entity : game.wheels[side].unwrap().entity, respawn: true });
				}

				// respawn
				if body_cfg_cache.fixed != body_cfg.fixed {
					game.body 		= Some(RespawnableEntity{ entity : game.body.unwrap().entity, respawn: true })
				}
			}
		}

		ui.separator();

		if (ui.button("Save Vehicle")).clicked() {
			let mut dialog			= FileDialog::save_file(None);
			dialog.open				();
			game.save_veh_dialog	= Some(dialog);
		}

		if (ui.button("Load Vehicle")).clicked() {
			let mut dialog			= FileDialog::open_file(None);
			dialog.open				();
			game.load_veh_dialog 	= Some(dialog);
		}

		if let Some(dialog) = &mut game.save_veh_dialog {
			if dialog.show(&ui.ctx()).selected() {
				game.save_veh_file 	= dialog.path();
			}
		}

		if let Some(dialog) = &mut game.load_veh_dialog {
			if dialog.show(&ui.ctx()).selected() {
				game.load_veh_file 	= dialog.path();
			}
		}

		ui.separator();

		if ui.button("Respawn Vehicle").clicked() {
			game.body 				= Some(RespawnableEntity{ entity : game.body.unwrap().entity, respawn: true });
		}

	});

// uncomment when we need to catch a closed window
//	match out {
//		Some(response) => {
//			if response.inner == None { println!("PEWPEWPEWPEW") }; 
//		}
//		_ => ()
//	}
}

fn file_path_to_string(buf: &Option<PathBuf>) -> String {
    match buf {
        Some(path) => path.display().to_string(),
        None => String::from(""),
    }
}

use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

use ron::ser::{ to_string_pretty, PrettyConfig };

use directories :: { BaseDirs, UserDirs, ProjectDirs };

#[derive(Default, Serialize, Deserialize)]
struct VehicleConfig {
    body 	: Option<BodyConfig>
  , axles	: [Option<AxleConfig>; WHEELS_MAX as usize]
  , wheels	: [Option<WheelConfig>; WHEELS_MAX as usize]
  , accel	: Option<AcceleratorConfig>
  , steer	: Option<SteeringConfig>
}

impl VehicleConfig {
	fn version() -> u32 { return 0; }
}

const VERSION_STR : &str = "version";

fn save_vehicle_config_system(
	mut game: ResMut<Game>,

	q_body	: Query	<(Entity, &BodyConfig)>,
	q_axle	: Query	<(Entity, &AxleConfig)>,
	q_wheel	: Query	<(Entity, &WheelConfig)>,
	q_accel	: Query <(Entity, &AcceleratorConfig)>,
	q_steer	: Query <(Entity, &SteeringConfig)>,
) {
	if game.save_veh_file.is_none() { return; }

	let mut veh_cfg = VehicleConfig::default();

	match game.body {
		Some(re) => {
			let (_, body) = q_body.get(re.entity).unwrap();
			veh_cfg.body = Some(*body);
			let (_, accel) = q_accel.get(re.entity).unwrap();
			veh_cfg.accel = Some(*accel);
			let (_, steer) = q_steer.get(re.entity).unwrap();
			veh_cfg.steer = Some(*steer);
		},
		_ => (),
	};

	for i in 0..WHEELS_MAX {
		match game.axles[i] {
			Some(re) => {
				let (_, axle) = q_axle.get(re.entity).unwrap();
				veh_cfg.axles[i] = Some(*axle);
			},
			_ => ()
		};

		match game.wheels[i] {
			Some(re) => {
				let (_, wheel) = q_wheel.get(re.entity).unwrap();
				veh_cfg.wheels[i] = Some(*wheel);
			},
			_ => ()
		}
	}

	let pretty = PrettyConfig::new()
		.depth_limit(5)
		.enumerate_arrays(true)
		.separate_tuple_members(true);

	let version_str = format!("{}: {}\n", VERSION_STR, VehicleConfig::version());
	let serialized	= to_string_pretty(&veh_cfg, pretty).expect("Serialization failed");
	let save_content = [
		  version_str
		, serialized
	].concat();

	let mut save_name = file_path_to_string(&game.save_veh_file);
	// if let Some(proj_dirs) = ProjectDirs::from("lol", "Gryazevicki Inc",  "Gryazevichki") {
	// 	save_name = [ proj_dirs.config_dir(), &save_name ].concat();
	// 	// Lin: /home/user/.config/gryazevichki
	// 	// Win: C:\Users\User\AppData\Roaming\Gryazevicki Inc\Gryazevicki\config
	// 	// Mac: /Users/User/Library/Application Support/lol.Gryazevicki-Inc.Gryazevicki
	// }

	let path = Path::new(&save_name);
    let display = path.display();

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    match file.write_all(save_content.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }

	game.save_veh_file = None;
}

fn load_vehicle_config_system(
	mut game	: ResMut<Game>,

	mut q_body	: Query	<&mut BodyConfig>,
	mut q_axle	: Query	<&mut AxleConfig>,
	mut q_wheel	: Query	<&mut WheelConfig>,
	mut q_accel	: Query <&mut AcceleratorConfig>,
	mut q_steer	: Query <&mut SteeringConfig>,
) {
	if game.load_veh_file.is_none() { return; }

    let load_name 	= file_path_to_string(&game.load_veh_file);
	let path 		= Path::new(&load_name);
    let display 	= path.display();

	game.load_veh_file = None;

    let mut file = match File::open(&path) {
        Err(why) 	=> { println!("couldn't open {}: {}", display, why); return; },
        Ok(file) 	=> file,
    };

    let mut save_content = String::new();
    match file.read_to_string(&mut save_content) {
        Err(why)	=> { println!("couldn't read {}: {}", display, why); return; },
        Ok(_) 		=> println!("Opened file {} for reading", display.to_string()),
    }

	let mut lines	= save_content.lines();
	let line 		= match lines.next() {
		Some(l)		=> l,
		None		=> { println!("{0} not found! Config should start with {0}", VERSION_STR); return; }
	};
	
	let version_value : String = match line.split_terminator(':').last() {
		Some(v)		=> v.chars().filter(|c| c.is_digit(10)).collect(),
		None		=> { println!("{0} value not found! Config should start with \"{0}: {1}\"", VERSION_STR, VehicleConfig::version()); return; }
	};
	let version 	= match version_value.parse::<u32>() {
		Ok(v) 		=> v,
		Err(why) 	=> { println!("Failed to parse version value ({})! Reason: {}", version_value, why); return; },
	};

	if version > VehicleConfig::version() {
		println!	("Invalid config version! Expected: <={} found: {}", VehicleConfig::version(), version);
		return;
	}

	let pos			= match save_content.find('(') {
		Some(p)		=> p - 1, // -1 to capture the brace as well lower, see save_content.get
		None		=> { println!("Failed to find first opening brace \"(\". Most likely invalid format or corrupted file!"); return; }
	};

	save_content	= match save_content.get(pos..) {
		Some(c)		=> c.to_string(),
		None		=> { return; }
	};

	let veh_cfg: VehicleConfig = ron::from_str(save_content.as_str()).unwrap();

	match game.body {
		Some(re) => {
			match q_body.get_mut(re.entity) {
				Ok(mut body) => *body = veh_cfg.body.unwrap_or_default(), _ => (),
			}
			match q_accel.get_mut(re.entity) {
				Ok(mut accel) => *accel = veh_cfg.accel.unwrap_or_default(), _ => (),
			}
			match q_steer.get_mut(re.entity) {
				Ok(mut steer) => *steer = veh_cfg.steer.unwrap_or_default(), _ => (),
			}
		},
		_ => (),
	};

	for i in 0..WHEELS_MAX {
		match game.axles[i] {
			Some(re) => {
				match q_axle.get_mut(re.entity) {
					Ok(mut axle) => *axle = veh_cfg.axles[i].unwrap_or_default(), _ => (),
				}
			},
			_ => ()
		};

		match game.wheels[i] {
			Some(re) => {
				match q_wheel.get_mut(re.entity) {
					Ok(mut wheel) => *wheel = veh_cfg.wheels[i].unwrap_or_default(), _ => (),
				}
			},
			_ => ()
		}
	}

	// respawn
	game.body = Some(RespawnableEntity{ entity : game.body.unwrap().entity, respawn: true });
}

fn respawn_vehicle_system(
	mut	game		: ResMut<Game>,
	mut	q_body		: Query<(
		&	 BodyConfig,
		&mut Transform
	)>,
		q_accel_cfg	: Query<&AcceleratorConfig>,
		q_steer_cfg	: Query<&SteeringConfig>,
		q_axle_cfg	: Query<&AxleConfig>,
		q_wheel_cfg	: Query<&WheelConfig>,
	mut	q_camera	: Query<&mut FlyCamera>,
		ass			: Res<AssetServer>,
	mut	commands	: Commands,
) {
	let (mut body, respawn_body) = match game.body {
		Some(re)		=> (re.entity, re.respawn),
		_				=> return,
	};
	let (body_cfg, mut body_pos) = q_body.get_mut(body).unwrap();
	let accel_cfg		= q_accel_cfg.get(body).unwrap();
	let steer_cfg		= q_steer_cfg.get(body).unwrap();

	if true == respawn_body {
		commands.entity(body).despawn_recursive();

		body_pos.translation = Vec3::new(0.0, 5.5, 0.0);
		body_pos.rotation = Quat::IDENTITY;
		body 			= spawn_body(*body_pos, *body_cfg, *accel_cfg, *steer_cfg, &ass, &mut commands);
		game.body 		= Some(RespawnableEntity { entity : body, ..Default::default() });
		// TODO: is there an event we can attach to? 
		let mut camera 	= q_camera.get_mut(game.camera.unwrap()).unwrap();
		camera.target 	= Some(body);
		println!		("camera.target Entity ID {:?}", camera.target);

		println!		("respawned body Entity ID {:?}", body);
	}
	return;
	for side_ref in WHEEL_SIDES {
		let side 		= *side_ref;
		let re_axle 	= game.axles[side].unwrap();
		let re_wheel	= game.wheels[side].unwrap();

		let mut axle	= re_axle.entity;
		let mut axle_pos : Transform;

		let axle_cfg 	= q_axle_cfg.get(axle).unwrap().clone();

		if !re_axle.respawn && !re_wheel.respawn && !respawn_body {
			continue;
		}

		commands.entity(axle).despawn_recursive();

		let axle_offset = body_cfg.axle_offset(side);
		(axle, axle_pos) = spawn_axle_with_joint(
			  side
			, body
			, *body_pos
			, axle_offset
			, axle_cfg
			, &ass
			, &mut commands
		);

		game.axles[side] = Some(RespawnableEntity{ entity : axle, respawn: false });

		println!		("respawned {} axle Entity ID {:?}", side, axle);
		
		let mut wheel	= re_wheel.entity;
		let wheel_cfg 	= q_wheel_cfg.get(wheel).unwrap().clone();

		commands.entity(wheel).despawn_recursive();

		let wheel_offset = axle_cfg.wheel_offset(side);
		wheel = spawn_wheel_with_joint(
			  side
			, axle
			, axle_pos
			, wheel_offset
			, wheel_cfg
			, &ass
			, &mut commands
		);

		game.wheels[side] = Some(RespawnableEntity{ entity : wheel, respawn: false });

		println!		("respawned {} wheel Entity ID {:?}", side, wheel);
	}
}

fn despawn_system(mut commands: Commands, time: Res<Time>, mut despawn: ResMut<DespawnResource>) {
    if time.seconds_since_startup() > 5.0 {
        for entity in &despawn.entities {
            println!("Despawning entity {:?}", entity);
            commands.entity(*entity).despawn_recursive();
        }
        despawn.entities.clear();
    }
}