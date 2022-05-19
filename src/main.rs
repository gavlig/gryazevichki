use bevy			::	prelude :: *;
use bevy			::	app::AppExit;
use bevy_rapier3d	::	{ prelude :: * };
use bevy_fly_camera	::	{ FlyCamera, FlyCameraPlugin };
use rapier3d		::	dynamics :: { JointAxis };

use serde			::	{ Deserialize, Serialize };

mod gchki_egui;

use gchki_egui		:: *;

use std				:: { path::PathBuf };

use bevy::render::mesh::shape as render_shape;

#[derive(Component)]
pub struct NameComponent {
	pub name : String
}

#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub enum VehiclePart {
	Wheel,
	Axle,
	Body,
	WheelJoint,
	AxleJoint,
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

// TODO: all this looks like a bad design, most likely i need a different approach
use usize as WheelSideType;
pub const FRONT_RIGHT		: WheelSideType = 0;
pub const FRONT_LEFT		: WheelSideType = 1;
pub const FRONT_SPLIT		: WheelSideType	= 2;

pub const REAR_RIGHT		: WheelSideType = 2;
pub const REAR_LEFT			: WheelSideType = 3;
pub const REAR_SPLIT		: WheelSideType = 4;

pub const WHEELS_MAX		: WheelSideType = 4;

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

pub struct Game {
	  camera					: Option<Entity>
	, body 						: Option<RespawnableEntity>

	, wheels					: [Option<RespawnableEntity>; WHEELS_MAX as usize]
	, axles						: [Option<RespawnableEntity>; WHEELS_MAX as usize]

	, opened_file				: Option<PathBuf>
    , open_file_dialog			: Option<FileDialog>
    , save_file_dialog			: Option<FileDialog>
}

impl Default for Game {
	fn default() -> Self {
		Self {
			  camera			: None
			, body 				: None
		
			, wheels			: [None; WHEELS_MAX as usize]
			, axles				: [None; WHEELS_MAX as usize]
		
			, opened_file		: None
			, open_file_dialog	: None
			, save_file_dialog	: None
		}
	}
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WheelConfig {
	  version					: u32
	, hh						: f32
	, r							: f32
	, density					: f32
	, mass						: f32
}

impl Default for WheelConfig {
	fn default() -> Self {
		Self {
			  version			: 0
			, hh				: 0.5
			, r					: 0.8
			, density			: 1.0
			, mass				: 0.0
		}
	}
}

#[derive(Default, Debug, Clone, Copy)]
pub struct WheelConfigChanged {
	  hh						: bool
	, r							: bool
	, density					: bool
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AxleConfig {
	  version					: u32
	, half_size					: Vec3
	, density					: f32
	, mass						: f32
}

impl Default for AxleConfig {
	fn default() -> Self {
		Self {
			  version			: 0
			, half_size			: Vec3::new(0.1, 0.2, 0.1)
			, density			: 1000.0
			, mass				: 0.0
		}
	}
}

#[derive(Default, Debug, Clone, Copy)]
pub struct AxleConfigChanged {
	  half_size					: bool
	, density					: bool
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BodyConfig {
	  version					: u32
	, half_size					: Vec3
	, density					: f32
	, fixed						: bool
	, wheel_offset_abs			: Vec3
}

impl Default for BodyConfig {
	fn default() -> Self {
		Self {
			  version			: 0
			, half_size			: Vec3::new(0.5, 0.5, 1.0)
			, density			: 2.0
			, fixed				: false
			, wheel_offset_abs	: Vec3::new(0.8, 0.8, 1.4)
		}
	}
}

impl BodyConfig {
	fn wheel_offset(self, side: WheelSideType) -> Vec3 {
		let off 				= &self.wheel_offset_abs;
		match side {
			FRONT_RIGHT			=> Vec3::new( off.x, -off.y,  off.z),
			FRONT_LEFT			=> Vec3::new(-off.x, -off.y,  off.z),
			REAR_RIGHT			=> Vec3::new( off.x, -off.y, -off.z),
			REAR_LEFT 			=> Vec3::new(-off.x, -off.y, -off.z),
			WHEELS_MAX			=> panic!("Max shouldn't be used as a wheel side!"),
			_					=> panic!("Only 4 sides are supported currently: 0 - 3 or FrontRight FrontLeft RearRight RearLeft"),
		}
	}
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AcceleratorConfig {
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
pub struct SteeringConfig {
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
pub struct DespawnResource {
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
		.insert_resource		(AcceleratorConfig::default())
		.insert_resource		(SteeringConfig::default())
		.insert_resource		(DespawnResource::default())
		.add_plugins			(DefaultPlugins)
		.add_plugin				(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin				(RapierDebugRenderPlugin::default())
		.add_plugin				(FlyCameraPlugin)
		.add_plugin				(bevy_egui::EguiPlugin)

		.add_startup_system		(setup_graphics_system)
		.add_startup_system		(setup_physics_system)
		.add_startup_system		(setup_grab_system)
		.add_startup_system_to_stage(StartupStage::PostStartup, setup_camera_system)

		.add_system				(cursor_grab_system)
		.add_system				(toggle_button_system)
		.add_system				(vehicle_controls_system)
		.add_system				(update_ui_system)
		.add_system				(save_vehicle_config_system)

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
		.insert			(Collider::ball(1.0))
		.insert			(FlyCamera::default())
		.insert			(NameComponent{ name: "Camera".to_string() })
		.id				();

	game.camera			= Some(camera);
	println!			("camera Entity ID {:?}", camera);
}

pub fn setup_physics_system(
	mut _configuration	: ResMut<RapierConfiguration>,
	mut game			: ResMut<Game>,
	mut	meshes			: ResMut<Assets<Mesh>>,
	mut	materials		: ResMut<Assets<StandardMaterial>>,
	mut commands		: Commands
) {
//	configuration.timestep_mode = TimestepMode::VariableTimestep;

	spawn_ground		(&game, &mut meshes, &mut materials, &mut commands);

	if false {
		spawn_cubes		(&mut commands);
	}

	let body_pos 		= Transform::from_xyz(0.0, 5.5, 0.0);
	let body_cfg		= BodyConfig::default();
	let axle_cfg		= AxleConfig::default();
	let wheel_cfg		= WheelConfig::default();
	spawn_vehicle		(&mut game, &mut meshes, &mut materials, body_cfg, axle_cfg, wheel_cfg, body_pos, &mut commands);
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
		body_cfg		: BodyConfig,
		axle_cfg		: AxleConfig,
		wheel_cfg		: WheelConfig,
		body_pos		: Transform,
	mut commands		: &mut Commands
) {
	let body 			= spawn_body(body_pos, body_cfg, &mut commands);
	game.body 			= Some(RespawnableEntity { entity : body, ..Default::default() });
	println!			("body Entity ID {:?}", body);

	// 0..1 {
	for side_ref in WHEEL_SIDES {
		let side 		= *side_ref;
		let offset 		= body_cfg.wheel_offset(side);
		let (axle, wheel) = spawn_attached_wheel(side, body, body_pos, offset, axle_cfg, wheel_cfg, &mut commands);
		game.axles[side] = Some(axle);
		game.wheels[side] = Some(wheel);
			
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
	commands		: &mut Commands
) -> (RespawnableEntity, RespawnableEntity) { // axle + wheel 
	let axle		= spawn_axle_with_joint(side, body, body_pos, axle_offset, axle_cfg, commands);
	let wheel		= spawn_wheel_with_joint(side, axle, body_pos, axle_offset, wheel_cfg, commands);

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
	mut	commands	: &mut Commands
) -> Entity {
	let axle_pos	= body_pos * Transform::from_translation(offset);
	let axle		= spawn_axle(side, body, axle_pos, RigidBody::Dynamic, cfg, &mut commands);

	let anchor1		= offset;
	let anchor2 	= Vec3::ZERO;
	spawn_axle_joint(body, axle, anchor1, anchor2, &mut commands);

	axle
}

fn spawn_wheel_with_joint(
	side			: WheelSideType,
	axle			: Entity,
	body_pos		: Transform,
	axle_offset		: Vec3,
	cfg				: WheelConfig,
	mut	commands	: &mut Commands
) -> Entity {
	let x_sign		= axle_offset.x * (1.0 / axle_offset.x.abs());
	let wheel_offset= Vec3::X * 0.8 * x_sign; // 0.2 offset by x axis
	let wheel_pos 	= body_pos * Transform::from_translation(axle_offset) * Transform::from_translation(wheel_offset);
	let wheel 		= spawn_wheel(
		  side
		, axle
		, wheel_pos
		, RigidBody::Dynamic
		, cfg
		, &mut commands
	);

	let anchor1		= wheel_offset;
	let anchor2 	= Vec3::ZERO;
	spawn_wheel_joint(axle, wheel, anchor1, anchor2, &mut commands);

	wheel
}

fn spawn_axle(
	side			: WheelSideType,
	body			: Entity,
	pos				: Transform,
	body_type		: RigidBody,
	cfg				: AxleConfig,
	commands		: &mut Commands,
) -> Entity {
	let side_name	= wheel_side_name(side);
	let (sidez, sidex) = wheel_side_to_zx(side);

	let mut axle_id = Entity::from_bits(0);
	commands
	.entity			(body)
	.with_children(|children| {
		axle_id = children
		.spawn		()
		.insert		(body_type)
		.insert		(cfg)
		.insert		(pos)
		.insert		(GlobalTransform::default())
		.insert		(MassProperties::default())
		.with_children(|children| {
			children
			.spawn	()
			.insert	(Transform::from_translation(Vec3::new(0.0, 0.3, 0.0)))
			.insert	(GlobalTransform::default())
			.insert	(Collider::cuboid(cfg.half_size.x, cfg.half_size.y, cfg.half_size.z))
			.insert	(ColliderMassProperties::Density(cfg.density));
		})
		.insert		(NameComponent{ name: format!("{} Axle", side_name) })
		.insert		(VehiclePart::Axle)
		.insert		(sidex)
		.insert		(sidez)
		.id			()
	});

	axle_id
}

fn spawn_wheel(
	side			: WheelSideType,
	axle			: Entity,
	pos				: Transform,
	body_type		: RigidBody,
	cfg				: WheelConfig,
	commands		: &mut Commands,
) -> Entity {
	let side_name	= wheel_side_name(side);
	let (sidez, sidex) = wheel_side_to_zx(side);
	let mut wheel_id = Entity::from_bits(0);
	// by default cylinder spawns with its flat surface on the ground and we want the round part
	commands
	.entity			(axle)
	.with_children	(|children| {
		wheel_id =
		children.spawn()
		.insert		(body_type)
		.insert		(cfg)
		.insert		(pos)
		.insert		(GlobalTransform::default())
		.insert		(MassProperties::default())
		.with_children(|children| {
			children.spawn()
			.insert	(Transform::from_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)))
			.insert	(Collider::cylinder(cfg.hh, cfg.r))
			.insert	(ColliderMassProperties::Density(cfg.density))
			.insert	(ActiveEvents::COLLISION_EVENTS);
		})
		.insert		(NameComponent{ name: format!("{} Wheel", side_name) })
		.insert		(VehiclePart::Wheel)
		.insert		(sidex)
		.insert		(sidez)
		.id			()
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
	commands		: &mut Commands,
) -> Entity {
	let body_type	= if cfg.fixed { RigidBody::Fixed } else { RigidBody::Dynamic };
	let half_size	= cfg.half_size;
	let density		= cfg.density;

	commands
		.spawn		()
		.insert		(body_type)
		.insert		(cfg)
		.insert		(pos)
		.insert		(GlobalTransform::default())
		.insert		(MassProperties::default())
		.with_children(|children| {
		children.spawn()
			.insert	(Collider::cuboid(half_size.x, half_size.y, half_size.z))
			.insert	(ColliderMassProperties::Density(density)); // joints like it when there is an hierarchy of masses and we want body to be the heaviest
		})	
		.insert		(NameComponent{ name: "Body".to_string() })
		.insert		(VehiclePart::Body)
		.insert		(SideX::Center)
		.insert		(SideZ::Center)
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

fn spawn_world_axis(
	meshes			: &mut ResMut<Assets<Mesh>>,
	materials		: &mut ResMut<Assets<StandardMaterial>>,
	commands		: &mut Commands,
) {
	// X
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(render_shape::Box::new(1.0, 0.1, 0.1))),
		material: materials.add(Color::rgb(0.8, 0.1, 0.1).into()),
		transform: Transform::from_xyz(0.5, 0.0 + 0.05, 0.0),
		..Default::default()
	});
	// Y
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(render_shape::Box::new(0.1, 1.0, 0.1))),
		material: materials.add(Color::rgb(0.1, 0.8, 0.1).into()),
		transform: Transform::from_xyz(0.0, 0.5 + 0.05, 0.0),
		..Default::default()
	});
	// Z
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(render_shape::Box::new(0.1, 0.1, 1.0))),
		material: materials.add(Color::rgb(0.1, 0.1, 0.8).into()),
		transform: Transform::from_xyz(0.0, 0.0 + 0.05, 0.5),
		..Default::default()
	});
}

fn cursor_grab_system(
	mut windows		: ResMut<Windows>,
	_btn			: Res<Input<MouseButton>>,
	key				: Res<Input<KeyCode>>,
) {
	let window = windows.get_primary_mut().unwrap();

	if key.just_pressed(KeyCode::Return) {
		window.set_cursor_lock_mode(true);
		window.set_cursor_visibility(false);
	}

	if key.just_pressed(KeyCode::Escape) {
		let toggle = !window.cursor_visible();
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
		accel_cfg	: Res<AcceleratorConfig>,
		steer_cfg	: Res<SteeringConfig>,
	mut	query		: Query<&mut ImpulseJoint>,
) {
	let fr_axle_joint = game.axles[FRONT_RIGHT];
	let fl_axle_joint = game.axles[FRONT_LEFT];

	let rr_wheel_joint = game.wheels[REAR_RIGHT];
	let rl_wheel_joint = game.wheels[REAR_LEFT];

	if key.just_pressed(KeyCode::W) {
		motor_velocity(accel_cfg.vel_fwd, accel_cfg.damping_fwd, rr_wheel_joint, &mut query);
		motor_velocity(accel_cfg.vel_fwd, accel_cfg.damping_fwd, rl_wheel_joint, &mut query);
	} else if key.just_released(KeyCode::W) {
		motor_velocity(0.0, accel_cfg.damping_stop, rr_wheel_joint, &mut query);
		motor_velocity(0.0, accel_cfg.damping_stop, rl_wheel_joint, &mut query);
	}
	
	if key.just_pressed(KeyCode::S) {
		motor_velocity(-accel_cfg.vel_bwd, accel_cfg.damping_bwd, rr_wheel_joint, &mut query);
		motor_velocity(-accel_cfg.vel_bwd, accel_cfg.damping_bwd, rl_wheel_joint, &mut query);
	} else if key.just_released(KeyCode::S) {
		motor_velocity(0.0, accel_cfg.damping_stop, rr_wheel_joint, &mut query);
		motor_velocity(0.0, accel_cfg.damping_stop, rl_wheel_joint, &mut query);
	}
 
	let steer_angle = steer_cfg.angle;
	let stiffness 	= steer_cfg.stiffness;
	let stiffness_release = steer_cfg.stiffness_release;
	let damping 	= steer_cfg.damping;
	let damping_release = steer_cfg.damping_release;
	if key.just_pressed(KeyCode::D) {
		motor_steer(-steer_angle, stiffness, damping, fr_axle_joint, &mut query);
		motor_steer(-steer_angle, stiffness, damping, fl_axle_joint, &mut query);
	} else if key.just_released(KeyCode::D) {
		motor_steer(0.0, stiffness_release, damping_release, fr_axle_joint, &mut query);
		motor_steer(0.0, stiffness_release, damping_release, fl_axle_joint, &mut query);
	}

 	if key.just_pressed(KeyCode::A) {
		motor_steer(steer_angle, stiffness, damping, fr_axle_joint, &mut query);
		motor_steer(steer_angle, stiffness, damping, fl_axle_joint, &mut query);
	} else if key.just_released(KeyCode::A) {
		motor_steer(0.0, stiffness_release, damping_release, fr_axle_joint, &mut query);
		motor_steer(0.0, stiffness_release, damping_release, fl_axle_joint, &mut query);
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

/* fn draw_density_param_ui(
	ui					: &mut Ui,
	name				: &String,
	range				: [f32; 2],
mut mass_props_co		: &mut Mut<ColliderMassProperties>,
	mass_props_rb		: &MassProperties
) {
	match &mut mass_props_co as &mut ColliderMassProperties {
	ColliderMassProperties::Density(density) => {
		if ui.add(
			Slider::new(&mut *density, range[0] ..= range[1]).text(format!("{} Density (Mass {:.3})", name, mass_props_rb.mass))
		).changed() {
			**mass_props_co = ColliderMassProperties::Density(*density);
		};
	},
	ColliderMassProperties::MassProperties(_) => (),
};
} 

fn draw_cylinder_param_ui(
	ui						: &mut Ui,
	shared_shape			: &mut Mut<Collider>,
) {
	let shape 				= shared_shape.raw.make_mut();
	let cylinder 			= shape.as_cylinder_mut().unwrap();

	egui::CollapsingHeader::new("Wheel sizes")
		.default_open(true)
		.show(ui, |ui| {

	ui.vertical(|ui| {
	
	ui.add(
		Slider::new(&mut cylinder.radius, 0.05 ..= 2.0)
			.text("Radius"),
	);

	ui.add(
		Slider::new(&mut cylinder.half_height, 0.05 ..= 2.0)
			.text("Half Height"),
	);

	}); // ui.vertical
	}); // ui.collapsing
}

fn draw_single_wheel_params_ui(
	ui						: &mut Ui,
	name					: &String,
	range					: [f32; 2],
	collider				: &mut Mut<Collider>,
	density					: &mut f32,
	mass					: f32,
) {
	draw_density_param_ui	(ui, name, range, density, mass);

	match collider.as_cylinder() {
		Some(_cylinder) 	=> draw_cylinder_param_ui(ui, collider),
		_ 					=> (),
	};
} */

fn draw_body_params_ui_collapsing(
	ui						: &mut Ui,
	name					: &String,
	density_range			: [f32; 2],
	half_size				: &mut Vec3,
	wheel_offset			: &mut Vec3,
	density					: &mut f32,
	mass					: f32,
	fixed					: &mut bool,
	section_name			: String
) -> bool {
	let mut changed			= false;			

	ui.collapsing(section_name, |ui| {
		ui.vertical(|ui| {
			changed			|= draw_density_param_ui(ui, name, density_range, density, mass);

			changed 		|= ui.add(
				Slider::new(&mut half_size[0], 0.05 ..= 5.0).text("Half Size X"),
			).changed();
			
			changed 		|= ui.add(
				Slider::new(&mut half_size[1], 0.05 ..= 5.0).text("Half Size Y"),
			).changed();

			changed 		|= ui.add(
				Slider::new(&mut half_size[2], 0.05 ..= 5.0).text("Half Size Z"),
			).changed();

			ui.separator	();

			changed 		|= ui.add(
				Slider::new(&mut wheel_offset[0], 0.05 ..= 5.0).text("Wheel Offset X"),
			).changed();
			
			changed 		|= ui.add(
				Slider::new(&mut wheel_offset[1], 0.05 ..= 5.0).text("Wheel Offset Y"),
			).changed();

			changed 		|= ui.add(
				Slider::new(&mut wheel_offset[2], 0.05 ..= 5.0).text("Wheel Offset Z"),
			).changed();

			changed			|= ui.checkbox(fixed, "Fixed (Debug)").changed();
		}); // ui.vertical
	}); // ui.collapsing

	changed
}

/* fn draw_single_wheel_params_ui_collapsing(
	ui				: &mut Ui,
	density_range	: [f32; 2],
	wheel: Vec<(
		&String,
		Mut<Collider>,
		Mut<ColliderMassProperties>,
		&MassProperties
	)>,
	section_name: String,
) {
	ui.collapsing(section_name, |ui| {
		ui.vertical(|ui| {
			for (name, mut coll_shape, mut mass_props_co, mass_props_rb) in wheel {
				draw_single_wheel_params_ui(
					ui,
					name,
					density_range,
					&mut coll_shape,
					&mut density,
					mass,
				);
			}
		});
	});
} */

fn draw_axle_params(
	ui						: &mut Ui
  , axle_cfg				: &mut AxleConfig
  , section_name			: String
) -> AxleConfigChanged {

	let mut axle_changed	= AxleConfigChanged::default();

	ui.collapsing(section_name, |ui| {
	ui.vertical(|ui| {

	if ui.add(
		Slider::new(&mut axle_cfg.half_size[0], 0.05 ..= 5.0).text("Half Size X"),
	).changed() {
		axle_changed.half_size = true;
	}
	
	if ui.add(
		Slider::new(&mut axle_cfg.half_size[1], 0.05 ..= 5.0).text("Half Size Y"),
	).changed() {
		axle_changed.half_size = true;
	}

	if ui.add(
		Slider::new(&mut axle_cfg.half_size[2], 0.05 ..= 5.0).text("Half Size Z"),
	).changed() {
		axle_changed.half_size = true;
	}

	if ui.add(
		Slider::new(&mut axle_cfg.density, 0.05 ..= 10000.0)
			.text(format!("Axle Density (Mass: {:.3})", axle_cfg.mass)),
	).changed() {
		axle_changed.density = true;
	}

	}); // ui.vertical
	}); // ui.collapsing

	axle_changed
}

fn draw_wheel_params(
	  ui					: &mut Ui
	, wheel_cfg				: &mut WheelConfig
	, section_name			: String
) -> WheelConfigChanged {

	let mut wheel_changed	= WheelConfigChanged::default();

	ui.collapsing(section_name, |ui| {
	ui.vertical(|ui| {

	if ui.add(
		Slider::new(&mut wheel_cfg.r, 0.05 ..= 2.0).text("Radius"),
	).changed() {
		wheel_changed.r		= true;
	}

	if ui.add(
		Slider::new(&mut wheel_cfg.hh, 0.05 ..= 2.0).text("Half Height"),
	).changed() {
		wheel_changed.hh	= true;
	}

	if ui.add(
		Slider::new(&mut wheel_cfg.density, 0.05 ..= 100.0)
			.text(format!("Wheel Density (Mass: {:.3})", wheel_cfg.mass)),
	).changed() {
		wheel_changed.density = true;
	}

	}); // ui.vertical
	}); // ui.collapsing

	wheel_changed
}

fn draw_acceleration_params_ui(
	ui				: &mut Ui,
	accel_cfg		: &mut ResMut<AcceleratorConfig>,
) {
	ui.collapsing("Acceleration".to_string(), |ui| {
	ui.vertical(|ui| {

	ui.add(
		Slider::new(&mut accel_cfg.vel_fwd, 0.05 ..= 400.0).text("Target Speed Forward"),
	);
	ui.add(
		Slider::new(&mut accel_cfg.damping_fwd, 0.05 ..= 1000.0).text("Acceleration Damping Forward"),
	);

	ui.add_space(1.0);
	
	ui.add(
		Slider::new(&mut accel_cfg.vel_bwd, 0.05 ..= 400.0).text("Target Speed Backward"),
	);
	ui.add(
		Slider::new(&mut accel_cfg.damping_bwd, 0.05 ..= 1000.0).text("Acceleration Damping Backward"),
	);

	ui.add_space(1.0);

	ui.add(
		Slider::new(&mut accel_cfg.damping_stop, 0.05 ..= 1000.0).text("Stopping Damping"),
	);

	}); // ui.vertical
	}); // ui.collapsing
}

fn draw_steering_params_ui(
	ui				: &mut Ui,
	steer_cfg		: &mut ResMut<SteeringConfig>,
) {
	ui.collapsing("Steering".to_string(), |ui| {
	ui.vertical(|ui| {

		ui.add(
			Slider::new(&mut steer_cfg.angle, 0.05 ..= 180.0).text("Steering Angle"),
		);
		ui.add(
			Slider::new(&mut steer_cfg.damping, 0.05 ..= 10000.0).text("Steering Damping"),
		);
		ui.add(
			Slider::new(&mut steer_cfg.damping_release, 0.05 ..= 10000.0).text("Steering Release Damping"),
		);
		ui.add(
			Slider::new(&mut steer_cfg.stiffness, 0.05 ..= 100000.0).text("Steering Stiffness"),
		);
		ui.add(
			Slider::new(&mut steer_cfg.stiffness_release, 0.05 ..= 100000.0).text("Steering Release Stiffness"),
		);

	}); // ui.vertical
	}); // ui.collapsing
}

fn update_ui_system(
	mut ui_context	: ResMut<EguiContext>,
	mut	game		: ResMut<Game>,
	mut accel_cfg	: ResMut<AcceleratorConfig>,
	mut steer_cfg	: ResMut<SteeringConfig>,

	mut q_child		: Query<(
		&Parent,
		&mut Collider,
		&mut ColliderMassProperties
	)>,
    	q_parent	: Query<(
		&VehiclePart,
		&SideZ,
		&NameComponent,
		&MassProperties
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
	)>
) {
	let mut body_cfg			= q_body_cfg.get_mut(game.body.unwrap().entity).unwrap();

	let window 					= egui::Window::new("Parameters");
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

		draw_acceleration_params_ui	(ui, &mut accel_cfg);
		draw_steering_params_ui		(ui, &mut steer_cfg);

		// ^^
		let (fl_wheel, _, _)		= q_wheel_cfg.get(game.wheels[FRONT_LEFT].unwrap().entity).unwrap();
		let (rl_wheel, _, _)		= q_wheel_cfg.get(game.wheels[REAR_LEFT].unwrap().entity).unwrap();
		let (fl_axle, _, _)			= q_axle_cfg.get(game.axles[FRONT_LEFT].unwrap().entity).unwrap();
		let (rl_axle, _, _)			= q_axle_cfg.get(game.axles[REAR_LEFT].unwrap().entity).unwrap();

		let mut front_wheel_common 	= fl_wheel.clone();
		let mut rear_wheel_common 	= rl_wheel.clone();
		let mut front_axle_common 	= fl_axle.clone();
		let mut rear_axle_common 	= rl_axle.clone();

		let front_axles_changed 	=
			draw_axle_params		(ui, &mut front_axle_common, String::from("Front Axles"));

		let front_wheels_changed 	=
			draw_wheel_params		(ui, &mut front_wheel_common, String::from("Front Wheels"));

		let rear_axles_changed		=
			draw_axle_params		(ui, &mut rear_axle_common, String::from("Rear Axles"));

		let rear_wheels_changed		=
			draw_wheel_params		(ui, &mut rear_wheel_common, String::from("Rear Wheels"));

		let writeback_axle_cfg = |
			  changed				: AxleConfigChanged
			, cfg_from				: &AxleConfig
			, cfg_to				: &mut AxleConfig
		| {
			if changed.half_size {
				cfg_to.half_size 	= cfg_from.half_size;
			}
			if changed.density {
				cfg_to.density 		= cfg_from.density;
			}
		};

		let writeback_wheel_cfg = |
			  changed				: WheelConfigChanged
			, cfg_from				: &WheelConfig
			, cfg_to				: &mut WheelConfig
		| {
			if changed.hh {
				cfg_to.hh 			= cfg_from.hh;
			}
			if changed.r {
				cfg_to.r 			= cfg_from.r;
			}
			if changed.density {
				cfg_to.density 		= cfg_from.density;
			}
		};

		for (mut wheel_cfg, _sidex, sidez) in q_wheel_cfg.iter_mut() {
			if *sidez == SideZ::Front {
				writeback_wheel_cfg(
					  front_wheels_changed
					, &front_wheel_common
					, &mut wheel_cfg
				);
			} else if *sidez == SideZ::Rear {
				writeback_wheel_cfg(
					  rear_wheels_changed
					, &rear_wheel_common
					, &mut wheel_cfg
				);
			}
		}

		for (mut axle_cfg, _sidex, sidez) in q_axle_cfg.iter_mut() {
			if *sidez == SideZ::Front {
				writeback_axle_cfg(
					front_axles_changed
				  , &front_axle_common
				  , &mut axle_cfg
				);
			}
			if *sidez == SideZ::Rear {
				writeback_axle_cfg(
					rear_axles_changed
				  , &rear_axle_common
				  , &mut axle_cfg
				);
			}
		}

		for (parent, mut collider, mut mass_props_co) in q_child.iter_mut() {
			let (vehicle_part, sidez, name_comp, mass_props_rb) = q_parent.get(parent.0).unwrap();
			let name 				= &name_comp.name;
			let vp 					= *vehicle_part;

			let writeback_axle_collider = |
				  changed	: AxleConfigChanged
				, cfg		: &AxleConfig
				, collider	: &mut Mut<Collider>
				, mass_props_co	: &mut Mut<ColliderMassProperties>,
			| {
				if changed.half_size {
					set_box_half_size(cfg.half_size, collider);
				}
				if changed.density {
					set_density		(cfg.density, mass_props_co);
				}
			};

			let writeback_wheel_collider = |
				  changed	: WheelConfigChanged
				, cfg		: &WheelConfig
				, collider	: &mut Mut<Collider>
				, mass_props_co	: &mut Mut<ColliderMassProperties>,
			| {
				if changed.hh {
					set_cylinder_hh	(cfg.hh, collider);
				}
				if changed.r {
					set_cylinder_r	(cfg.r, collider);
				}
				if changed.density {
					set_density		(cfg.density, mass_props_co);
				}
			};

			let mut body_changed	= false;
			let 	body_cfg_cache 	= body_cfg.clone();

			if vp == VehiclePart::Body {
				// thanks kpreid!
				let mut half_size	= body_cfg.half_size.clone();
				let mut wheel_offset= body_cfg.wheel_offset_abs.clone();
				let mut density		= 1.0;
				match &mut mass_props_co as &mut ColliderMassProperties {
					ColliderMassProperties::Density(d) => density = *d,
					ColliderMassProperties::MassProperties(_) => (),
				};
				let mass			= mass_props_rb.mass;
				let mut fixed		= body_cfg.fixed;

				body_changed 		= draw_body_params_ui_collapsing(ui, name, [0.05, 100.0], &mut half_size, &mut wheel_offset, &mut density, mass, &mut fixed, "Body".to_string());

				body_cfg.half_size	= half_size;
				body_cfg.wheel_offset_abs = wheel_offset;
				body_cfg.density	= density;
				body_cfg.fixed		= fixed;
			} else if vp == VehiclePart::Wheel && *sidez == SideZ::Front {
				writeback_wheel_collider(front_wheels_changed, &front_wheel_common, &mut collider, &mut mass_props_co);
			} else if vp == VehiclePart::Wheel && *sidez == SideZ::Rear {
				writeback_wheel_collider(rear_wheels_changed, &rear_wheel_common, &mut collider, &mut mass_props_co);
			} else if vp == VehiclePart::Axle && *sidez == SideZ::Front {
				writeback_axle_collider(front_axles_changed, &front_axle_common, &mut collider, &mut mass_props_co);
			} else if vp == VehiclePart::Axle && *sidez == SideZ::Rear {
				writeback_axle_collider(rear_axles_changed, &rear_axle_common, &mut collider, &mut mass_props_co);
			}

			if vp == VehiclePart::Wheel {
				let (mut wheel_cfg, _, _) = q_wheel_cfg.get_mut(parent.0).unwrap();
				wheel_cfg.mass		= mass_props_rb.mass;
			}

			if vp == VehiclePart::Axle {
				let (mut axle_cfg, _, _) = q_axle_cfg.get_mut(parent.0).unwrap();
				axle_cfg.mass		= mass_props_rb.mass;
			}

			if body_changed {
				*mass_props_co	 	= ColliderMassProperties::Density(body_cfg.density);

				let cuboid 			= collider.as_cuboid_mut().unwrap();
				cuboid.raw.half_extents = body_cfg.half_size.into();

				let delta			= body_cfg.half_size - body_cfg_cache.half_size;
				body_cfg.wheel_offset_abs += delta;

				for side_ref in WHEEL_SIDES {
					let side 		= *side_ref;
					game.axles[side] = Some(RespawnableEntity{ entity : game.axles[side].unwrap().entity, respawn: true }); // todo: hide the ugly
					game.wheels[side] = Some(RespawnableEntity{ entity : game.wheels[side].unwrap().entity, respawn: true });
				}

				if body_cfg_cache.fixed != body_cfg.fixed {
					game.body 		= Some(RespawnableEntity{ entity : game.body.unwrap().entity, respawn: true })
				}
			}
		}

		ui.separator();

		if ui.button("Respawn Vehicle").clicked() {
			game.body 				= Some(RespawnableEntity{ entity : game.body.unwrap().entity, respawn: true });
		}

		ui.separator();

		ui.horizontal(|ui| {
			let file_name = match &game.opened_file {
			 	Some(_) => file_path_to_string(&game.opened_file),
			 	None 	=> { game.opened_file = Some(PathBuf::from(r"gchki_vehicle.cfg")); file_path_to_string(&game.opened_file) }, //String::from("[DROP FILE HERE]"),
			};

			// if ui.button(file_name).hovered() {
			// 	if let Some(file) = egui.ctx.input().raw.dropped_files.first() {
			// 		game.opened_file = file.path.clone();
			// 	}
			// }

			// if (ui.button("Open")).clicked() {
			// 	let mut dialog = FileDialog::open_file(game.opened_file.clone());
			// 	dialog.open();
			// 	game.open_file_dialog = Some(dialog);
			// }

			// if let Some(dialog) = &mut game.open_file_dialog {
			// 	if dialog.show(&egui.ctx).selected() {
			// 		if let Some(file) = dialog.path() {
			// 			game.opened_file = Some(file);
			// 		}
			// 	}
			// }

			if (ui.button("❌")).clicked() {
				game.opened_file = None;
			}
		});

		// ui.label("Hovering files:");
		// let hovered_files = &egui.ctx.input().raw.hovered_files;
		// if !hovered_files.is_empty() {
		// 	for file in hovered_files.iter() {
		// 		ui.label(format!("File: {}", file_path_to_string(&file.path)));
		// 	}
		// } else {
		// 	ui.label("Nothing");
		// }
		if (ui.button("Save")).clicked() {
			let mut dialog = FileDialog::save_file(game.opened_file.clone());
			dialog.open();
			game.save_file_dialog = Some(dialog);
		}

		if let Some(dialog) = &mut game.save_file_dialog {
			if dialog.show(&ui.ctx()).selected() {
				if let Some(file) = dialog.path() {
					println!("Should save {:?}", file);
				}
			}
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

fn file_path_to_string(buf: &Option<std::path::PathBuf>) -> String {
    match buf {
        Some(path) => path.display().to_string(),
        None => String::from(""),
    }
}

use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::any::type_name;

use ron::ser::{to_string_pretty, PrettyConfig};

use directories :: { BaseDirs, UserDirs, ProjectDirs };

fn save_vehicle_config_system(
	key		: Res	<Input<KeyCode>>,
	game	: Res	<Game>,

	q_body	: Query	<(Entity, &BodyConfig)>,
	q_axle	: Query	<(Entity, &AxleConfig)>,
	q_wheel	: Query	<(Entity, &WheelConfig)>,
) {
//	let x: MyStruct = ron::from_str("(boolean: true, float: 1.23)").unwrap();

	if !key.just_released(KeyCode::K) { return; }

	let mut save_content = String::new();

	match game.body {
		Some(re) => {
			let (_, body_cfg) = q_body.get(re.entity).unwrap();

			let pretty = PrettyConfig::new().depth_limit(5);
			let s = to_string_pretty(&body_cfg, pretty).expect("Serialization failed");

			save_content = [ save_content, type_name::<BodyConfig>().to_string(), s ].join("\n"); 
		},
		_ => (),
	};

	let mut save_name = "gchki_vehicle.cfg".to_owned();
	if let Some(proj_dirs) = ProjectDirs::from("lol", "Gryazevicki Inc",  "Gryazevichki") {
//		save_name = [ proj_dirs.config_dir(), &save_name ].concat();
		// Lin: /home/user/.config/gryazevichki
		// Win: C:\Users\User\AppData\Roaming\Gryazevicki Inc\Gryazevicki\config
		// Mac: /Users/User/Library/Application Support/lol.Gryazevicki-Inc.Gryazevicki
	}

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


}

pub fn respawn_vehicle_system(
	mut	game		: ResMut<Game>,
	mut	q_body		: Query<(
		&	 BodyConfig,
		&mut Transform
	)>,
		q_axle_cfg	: Query<
		&mut AxleConfig
	>,
		q_wheel_cfg	: Query<
		&mut WheelConfig
	>,
	mut q_camera	: Query<
		&mut FlyCamera
	>,
	mut commands	: Commands,
) {
	let (mut body, respawn_body) = match game.body {
		Some(re)		=> (re.entity, re.respawn),
		_				=> return,
	};
	let (body_cfg, mut body_pos) = q_body.get_mut(body).unwrap();

	if true == respawn_body {
		commands.entity(body).despawn_recursive();

		body_pos.translation = Vec3::new(0.0, 5.5, 0.0);
		body 			= spawn_body(*body_pos, *body_cfg, &mut commands);
		game.body 		= Some(RespawnableEntity { entity : body, ..Default::default() });
		// TODO: is there an event we can attach to? 
		let mut camera 	= q_camera.get_mut(game.camera.unwrap()).unwrap();
		camera.target 	= Some(body);
		println!		("camera.target Entity ID {:?}", camera.target);

		println!		("respawned body Entity ID {:?}", body);
	}

	for side_ref in WHEEL_SIDES {
		let side 		= *side_ref;

		let axle_offset = body_cfg.wheel_offset(side);
		let 	re_axle = game.axles[side].unwrap();
		let mut axle	= re_axle.entity;

		if re_axle.respawn || respawn_body {
			let axle_cfg = q_axle_cfg.get(axle).unwrap().clone();

			commands.entity(axle).despawn_recursive();

			axle = spawn_axle_with_joint(
				  side
				, body
				, *body_pos
				, axle_offset
				, axle_cfg
				, &mut commands
			);

			game.axles[side] = Some(RespawnableEntity{ entity : axle, respawn: false });

			println!		("respawned {} axle Entity ID {:?}", side, axle);
		}

		let 	re_wheel	= game.wheels[side].unwrap();
		let mut wheel		= re_wheel.entity;
		if re_wheel.respawn || respawn_body {
			let wheel_cfg 	= q_wheel_cfg.get(wheel).unwrap().clone();

			commands.entity(wheel).despawn_recursive();

			wheel = spawn_wheel_with_joint(
				  side
				, axle
				, *body_pos
				, axle_offset
				, wheel_cfg
				, &mut commands
			);

			game.wheels[side] = Some(RespawnableEntity{ entity : wheel, respawn: false });

			println!		("respawned {} wheel Entity ID {:?}", side, wheel);
		}
	}
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