use bevy::prelude::*;
use bevy_rapier3d::{prelude::*};
use rapier3d::dynamics::{JointAxis};//, JointLimits, JointMotor, MotorModel};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy::app::AppExit;

use bevy::render::mesh::shape as render_shape;

#[derive(Component)]
pub struct NameComponent {
	pub name : String
}

#[derive(Component, Debug)]
pub enum Tag {
	Wheel,
	Axle,
	Body,
	WheelJoint,
	AxleJoint,
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

const WHEEL_SIDES: &'static [WheelSideType] = &[
	  FRONT_RIGHT
	, FRONT_LEFT
	, REAR_LEFT
	, REAR_RIGHT
];

#[derive(Default, Debug)]
pub struct WheelEntity {
	wheel			: Option<Entity>
  , wheel_joint		: Option<Entity>
  , axle			: Option<Entity>
  , axle_joint		: Option<Entity>
}

#[derive(Default)]
pub struct Game {
	  camera		: Option<Entity>
	, body 			: Option<Entity>

	, wheels		: [WheelEntity; WHEELS_MAX as usize]
}

#[derive(Debug, Clone, Copy)]
pub struct WheelConfig {
	  hh						: f32
	, r							: f32
	, density					: f32
}

impl Default for WheelConfig {
	fn default() -> Self {
		Self {
			  hh				: 0.5
			, r					: 0.8
			, density			: 1.0
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct AxleConfig {
	  half_size					: Vec3
	, density					: f32
}

impl Default for AxleConfig {
	fn default() -> Self {
		Self {
			  half_size			: Vec3::new(0.1, 0.2, 0.1)
			, density			: 1.0
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct VehicleConfig {
	  body_half_size			: Vec3
	, body_density				: f32
	, wheel_offset_abs			: Vec3
	, wheel_cfg					: [WheelConfig;	WHEELS_MAX as usize]
	, axle_cfg					: [AxleConfig;	WHEELS_MAX as usize]
}

impl Default for VehicleConfig {
	fn default() -> Self {
		Self {
			  body_half_size	: Vec3::new(0.5, 0.5, 1.0)
			, body_density		: 1.0
			, wheel_offset_abs	: Vec3::new(0.8, 0.8, 1.4)
			, wheel_cfg			: [WheelConfig::default();	WHEELS_MAX as usize]
			, axle_cfg			: [AxleConfig::default();	WHEELS_MAX as usize]
		}
	}
}

#[derive(Debug, Clone, Copy)]
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
			, damping_fwd		: 1.0
			, damping_bwd		: 1.0
			, damping_stop		: 2.0
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct SteeringConfig {
	  stiffness					: f32
	, damping					: f32
	, angle						: f32
}

impl Default for SteeringConfig {
	fn default() -> Self {
		Self {
			  stiffness			: 1.0 // was 5
			, damping			: 1.0 	// was 3
			, angle				: 20.0
		}
	}
}

impl VehicleConfig {
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

fn main() {
	App::new()
		.insert_resource(ClearColor(Color::rgb(
			0xF9 as f32 / 255.0,
			0xF9 as f32 / 255.0,
			0xFF as f32 / 255.0,
		)))
		.insert_resource(Msaa::default())
		.init_resource::<Game>()
		.insert_resource(VehicleConfig::default())
		.insert_resource(AcceleratorConfig::default())
		.insert_resource(SteeringConfig::default())
		.add_plugins(DefaultPlugins)
		.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
		.add_plugin(FlyCameraPlugin)
		.add_plugin(bevy_egui::EguiPlugin)
		.add_startup_system(setup_graphics_system)
		.add_startup_system(setup_physics_system)
		.add_startup_system(setup_grab_system)
		.add_startup_system_to_stage(StartupStage::PostStartup, setup_camera_system)
		.add_system(cursor_grab_system)
		.add_system(toggle_button_system)
		.add_system(accelerate_system)
		.add_system(update_ui)
		.add_system_to_stage(CoreStage::PostUpdate, display_events_system)
		.run();
}

fn setup_grab_system(mut windows: ResMut<Windows>) {
	let window = windows.get_primary_mut().unwrap();

	window.set_cursor_lock_mode(true);
	window.set_cursor_visibility(false);
}

fn setup_graphics_system(
	mut	meshes: ResMut<Assets<Mesh>>,
	mut	materials: ResMut<Assets<StandardMaterial>>,
	mut game: ResMut<Game>,
	mut commands: Commands,
) {
	const HALF_SIZE: f32 = 100.0;

	commands.spawn_bundle(DirectionalLightBundle {
		directional_light: DirectionalLight {
			illuminance: 10000.0,
			// Configure the projection to better fit the scene
			shadow_projection: OrthographicProjection {
				left: -HALF_SIZE,
				right: HALF_SIZE,
				bottom: -HALF_SIZE,
				top: HALF_SIZE,
				near: -10.0 * HALF_SIZE,
				far: 100.0 * HALF_SIZE,
				..Default::default()
			},
			shadows_enabled: true,
			..Default::default()
		},
		transform: Transform {
			translation: Vec3::new(10.0, 2.0, 10.0),
			rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
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
		vehicle_cfg		: Res<VehicleConfig>,
	mut commands		: Commands
) {
//	configuration.timestep_mode = TimestepMode::VariableTimestep;

	spawn_ground		(&game, &mut meshes, &mut materials, &mut commands);

	if false {
		spawn_cubes		(&mut commands);
	}

	spawn_vehicle		(&mut game, &mut meshes, &mut materials, &vehicle_cfg, &mut commands);
}

fn setup_camera_system(
		 game			: ResMut<Game>,
	mut query			: Query<&mut FlyCamera>
) {
	// initialize camera with target to look at
	if game.camera.is_some() && game.body.is_some() {
		let mut camera 	= query.get_mut(game.camera.unwrap()).unwrap();
		camera.target 	= Some(game.body.unwrap());
		println!		("{:?} camera.target", camera.target);
	}
}

fn spawn_ground(
	_game				: &ResMut<Game>,
	mut meshes			: &mut ResMut<Assets<Mesh>>,
	mut materials		: &mut ResMut<Assets<StandardMaterial>>,
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
		vehicle_cfg		: &Res<VehicleConfig>,
	mut commands		: &mut Commands
) {
	let body_pos 		= Vec3::new(0.0, 5.5, 0.0);
	let body 			= spawn_body(body_pos, vehicle_cfg.body_half_size, RigidBody::Dynamic, vehicle_cfg.body_density, &mut commands);
	game.body 			= Some(body);
	println!			("body Entity ID {:?}", body);

	// 0..1 {
	for side_ref in WHEEL_SIDES {
		let side 		= *side_ref;
		let offset 		= &vehicle_cfg.wheel_offset(side);
		let wheel_cfg	= &vehicle_cfg.wheel_cfg[side];
		let axle_cfg	= &vehicle_cfg.axle_cfg[side];
		game.wheels[side] =
			spawn_attached_wheel(side, body, body_pos, offset.clone(), wheel_cfg, axle_cfg, &mut commands);
			
		println!		("{} Wheel spawned! {:?}", wheel_side_name(side), game.wheels[side]);
	}
}

fn spawn_attached_wheel(
	side			: WheelSideType,
	body			: Entity,
	body_pos		: Vec3,
	offset			: Vec3,
	wheel_cfg		: &WheelConfig,
	axle_cfg		: &AxleConfig,
	mut	commands	: &mut Commands
) -> WheelEntity {
	let side_name	= wheel_side_name(side);

	let axle_size	= axle_cfg.half_size;
	let axle_pos	= body_pos + offset;
	let axle		= spawn_axle(side_name, body, axle_pos, axle_size, RigidBody::Dynamic, axle_cfg.density, &mut commands);

	let mut anchor1	= offset;
	let mut anchor2 = Vec3::ZERO;
	let axle_joint 	= spawn_axle_joint(body, axle, anchor1, anchor2, &mut commands);

	let x_sign		= offset.x * (1.0 / offset.x.abs());
	let wheel_offset= Vec3::X * 0.8 * x_sign; // 0.2 offset by x axis
	let wheel_pos 	= axle_pos + wheel_offset;
	let wheel 		= spawn_wheel(side_name, axle, wheel_pos, wheel_cfg.hh, wheel_cfg.r, RigidBody::Dynamic, wheel_cfg.density, &mut commands);

	anchor1			= wheel_offset;
	anchor2 		= Vec3::ZERO;
	let wheel_joint = spawn_wheel_joint(axle, wheel, anchor1, anchor2, &mut commands);

	WheelEntity {
		wheel		: Some(wheel),
		wheel_joint	: Some(wheel_joint),
		axle		: Some(axle),
		axle_joint	: Some(axle_joint),
		..Default::default()
	}
}

fn spawn_axle(
	prefix			: &str,
	body			: Entity,
	pos				: Vec3,
	half_size		: Vec3,
	body_type		: RigidBody,
	density			: f32,
	commands		: &mut Commands,
) -> Entity {
	let mut axle_id = Entity::from_bits(0);
	commands
		.entity		(body)
		.with_children(|children| {
		axle_id = children
		.spawn		()
		.insert		(body_type)
		.insert		(Transform::from_translation(pos))
		.insert		(GlobalTransform::default())
		.insert		(MassProperties::default())
		.with_children(|children| {
			children
				.spawn()
				.insert(Transform::from_translation(Vec3::new(0.0, 0.3, 0.0)))
				.insert(GlobalTransform::default())
				.insert(Collider::cuboid(half_size.x, half_size.y, half_size.z))
				.insert(ColliderMassProperties::Density(density));
		})
		.insert		(NameComponent{ name: format!("{} Axle", prefix) })
		.insert		(Tag::Axle)
		.id			()
	});

	axle_id
}

fn spawn_wheel(
	prefix			: &str,
	body			: Entity,
	pos				: Vec3,
	half_height		: f32,
	radius			: f32,
	body_type		: RigidBody,
	density			: f32,
	commands		: &mut Commands,
) -> Entity {
	let mut wheel_id = Entity::from_bits(0);
	// by default cylinder spawns with its flat surface on the ground and we want the round part
	commands
		.entity			(body)
		.with_children	(|children| {
			wheel_id =
			children.spawn()
			.insert		(body_type)
			.insert		(Transform::from_translation(pos))
			.insert		(GlobalTransform::default())
			.insert		(MassProperties::default())
			.with_children(|children| {
				children.spawn()
				.insert	(Transform::from_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)))
				.insert	(Collider::cylinder(half_height, radius))
				.insert	(ColliderMassProperties::Density(density))
				.insert	(ActiveEvents::COLLISION_EVENTS);
			})
			.insert		(NameComponent{ name: format!("{} Wheel", prefix) })
			.insert		(Tag::Wheel)
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
) -> Entity {
	let axle_joint = RevoluteJointBuilder::new(Vec3::Y)
		.local_anchor1(anchor1)
		.local_anchor2(anchor2)
		.limits([0., 0.]);

	commands
		.entity		(entity2)
		.insert		(ImpulseJoint::new(entity1, axle_joint))
//		.insert		(NameComponent{ name: "Axle Joint".to_string() })
//		.insert		(Tag::AxleJoint)
		.id()
}

fn spawn_wheel_joint(
	entity1			: Entity,
	entity2			: Entity,
	anchor1			: Vec3,
	anchor2			: Vec3,
	commands		: &mut Commands,
) -> Entity {
	let wheel_joint = RevoluteJointBuilder::new(Vec3::X)
		.local_anchor1(anchor1)
		.local_anchor2(anchor2);

	commands
		.entity		(entity2)
		.insert		(ImpulseJoint::new(entity1, wheel_joint))
//		.insert		(NameComponent{ name: "Wheel Joint".to_string() })
//		.insert		(Tag::WheelJoint)
		.id()
}

fn spawn_body(
	pos				: Vec3,
	half_size		: Vec3,
	body_type		: RigidBody,
	density			: f32,
	commands		: &mut Commands,
) -> Entity {
	commands
		.spawn		()
		.insert		(body_type)
		.insert		(Transform::from_translation(pos))
		.insert		(GlobalTransform::default())
		.insert		(MassProperties::default())
		.with_children(|children| {
		children.spawn()
			.insert	(Collider::cuboid(half_size.x, half_size.y, half_size.z))
			.insert	(ColliderMassProperties::Density(density)); // joints like it when there is an hierarchy of masses and we want body to be the heaviest
		})	
		.insert		(NameComponent{ name: "Body".to_string() })
		.insert		(Tag::Body)
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
	mut meshes		: &mut ResMut<Assets<Mesh>>,
	mut materials	: &mut ResMut<Assets<StandardMaterial>>,
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
	joint_e			: Option<Entity>,
	query			: &mut Query<&mut ImpulseJoint>
) {
	match joint_e {
		Some(entity) => {
			let mut	joint	= query.get_mut(entity).unwrap();
			joint.data.set_motor_velocity(JointAxis::AngX, velocity, factor);
		}
		_ => ()
	}
}

fn motor_steer(angle: f32, stiffness: f32, damping: f32, joint_e: Option<Entity>, query: &mut Query<&mut ImpulseJoint>) {
	match joint_e {
		Some(entity) => {
			let mut joint 	= query.get_mut(entity).unwrap();
			let	angle_rad	= angle.to_radians();
			joint.data
			.set_motor_position(JointAxis::AngX, angle_rad, stiffness, damping)
			;//.set_limits(JointAxis::AngX, [-angle_rad, angle_rad]);
			
		}
		_ => ()
	}
//	println!("motor steer {:?}", joint);

//	println!("motor steer {} limit axes {:?}", angle, joint.data.limit_axes);
//	if angle.abs() > 0.0001 {
//			joint.data 	= joint.data.unlimit_axis(JointAxis::AngZ);
//	} else {
//			joint.data 	= joint.data.limit_axis(JointAxis::AngZ, [-0.0001, 0.0001]);
//	}
}

fn accelerate_system(
		key			: Res<Input<KeyCode>>,
		game		: ResMut<Game>,
		accel_cfg	: Res<AcceleratorConfig>,
		steer_cfg	: Res<SteeringConfig>,
	mut	query		: Query<&mut ImpulseJoint>,
) {
	let fr_axle_joint = game.wheels[FRONT_RIGHT].axle_joint;
	let fl_axle_joint = game.wheels[FRONT_LEFT].axle_joint;

	let rr_wheel_joint = game.wheels[REAR_RIGHT].wheel_joint;
	let rl_wheel_joint = game.wheels[REAR_LEFT].wheel_joint;

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
	let damping 	= steer_cfg.damping;
	if key.just_pressed(KeyCode::D) {
		motor_steer(-steer_angle, stiffness, damping, fr_axle_joint, &mut query);
		motor_steer(-steer_angle, stiffness, damping, fl_axle_joint, &mut query);
	} else if key.just_released(KeyCode::D) {
		motor_steer(0.0, stiffness, damping, fr_axle_joint, &mut query);
		motor_steer(0.0, stiffness, damping, fl_axle_joint, &mut query);
	}

 	if key.just_pressed(KeyCode::A) {
		motor_steer(steer_angle, stiffness, damping, fr_axle_joint, &mut query);
		motor_steer(steer_angle, stiffness, damping, fl_axle_joint, &mut query);
	} else if key.just_released(KeyCode::A) {
		motor_steer(0.0, stiffness, damping, fr_axle_joint, &mut query);
		motor_steer(0.0, stiffness, damping, fl_axle_joint, &mut query);
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

fn set_density(
		density_in			: f32,
	mut mass_props_co		: &mut Mut<ColliderMassProperties>,
) {
	match &mut mass_props_co as &mut ColliderMassProperties {
		ColliderMassProperties::Density(density) => {
			**mass_props_co = ColliderMassProperties::Density(*density);
		},
		ColliderMassProperties::MassProperties(_) => (),
	};
}

fn draw_density_param_ui(
		ui					: &mut Ui,
		name				: &String,
	mut mass_props_co		: &mut Mut<ColliderMassProperties>,
		mass_props_rb		: &MassProperties
) {
	match &mut mass_props_co as &mut ColliderMassProperties {
		ColliderMassProperties::Density(density) => {
			if ui.add(
				Slider::new(&mut *density, 0.01 ..= 1000.0).text(format!("{} Density (Mass {:.3})", name, mass_props_rb.mass))
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
		Slider::new(&mut cylinder.radius, 0.05 ..= 1.0)
			.text("Radius"),
	);

	ui.add(
		Slider::new(&mut cylinder.half_height, 0.05 ..= 1.0)
			.text("Half Height"),
	);

	}); // ui.vertical
	}); // ui.collapsing
}

fn draw_single_wheel_params_ui(
	ui						: &mut Ui,
	name					: &String,
	collider				: &mut Mut<Collider>,
	mass_props_co			: &mut Mut<ColliderMassProperties>,
	mass_props_rb			: &MassProperties,
) {
	draw_density_param_ui	(ui, name, mass_props_co, mass_props_rb);

	match collider.as_cylinder() {
		Some(_cylinder) 	=> draw_cylinder_param_ui(ui, collider),
		_ 					=> (),
	};
}

fn draw_body_params_ui_collapsing(
	ui						: &mut Ui,
	name					: &String,
	collider				: &mut Mut<Collider>,
	mass_props_co			: &mut Mut<ColliderMassProperties>,
	mass_props_rb			: &MassProperties,
	section_name			: String
) {
	ui.collapsing(section_name, |ui| {
		ui.vertical(|ui| {
			draw_density_param_ui(ui, name, mass_props_co, mass_props_rb);

			let cuboid = collider.as_cuboid_mut().unwrap();

			if ui.add(
				Slider::new(&mut cuboid.raw.half_extents[0], 0.05 ..= 5.0)
					.text("Half Height X"),
			).changed() {
				println!("{}", cuboid.raw.half_extents[0]);
			}
			
			ui.add(
				Slider::new(&mut cuboid.raw.half_extents[1], 0.05 ..= 5.0)
					.text("Half Height Y"),
			);

			ui.add(
				Slider::new(&mut cuboid.raw.half_extents[2], 0.05 ..= 5.0)
					.text("Half Height Z"),
			);
		}); // ui.vertical
	}); // ui.collapsing
}

fn draw_single_wheel_params_ui_collapsing(
	ui: &mut Ui,
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
			for (name_in, mut coll_shape, mut mass_props_co, mass_props_rb) in wheel {
				draw_single_wheel_params_ui(
					ui,
					name_in,
					&mut coll_shape,
					&mut mass_props_co,
					mass_props_rb,
				);
			}
		});
	});
}

fn update_ui(
	mut ui_context	: ResMut<EguiContext>,
		_game		: Res	<Game>,
	mut veh_cfg		: ResMut<VehicleConfig>,
	mut accel_cfg	: ResMut<AcceleratorConfig>,
	mut steer_cfg	: ResMut<SteeringConfig>,

	mut q_child		: Query<(
		&Parent,
		&mut Collider,
		&mut ColliderMassProperties
	)>,
    	q_parent	: Query<(
		&NameComponent,
		&Tag,
		&MassProperties
	)>,
) {
	let window 					= egui::Window::new("Parameters");
	//let out = 
	window.show(ui_context.ctx_mut(), |ui| {
		// get front and rear minimal wheel size
		let (mut front_wheels, mut rear_wheels)	= veh_cfg.wheel_cfg.split_at_mut(FRONT_SPLIT);
		let mut front_wheel_common	= front_wheels.first().unwrap().clone();
		let mut rear_wheel_common	= rear_wheels.first().unwrap().clone();

		let (mut front_axles, mut rear_axles)	= veh_cfg.axle_cfg.split_at_mut(FRONT_SPLIT);
		let mut front_axle_common	= front_axles.first().unwrap().clone();
		let mut rear_axle_common	= rear_axles.first().unwrap().clone();

		let set_wheels_hh = |hh: f32, wheels: &mut [WheelConfig]| {
			for wh in wheels {
				wh.hh				= hh;
			}
		};

		let set_wheels_r = |r: f32, wheels: &mut [WheelConfig]| {
			for wh in wheels {
				wh.r				= r;
			}
		};

		let set_wheels_density = |density: f32, wheels: &mut [WheelConfig]| {
			for wh in wheels {
				wh.density			= density;
			}
		};

		let set_axles_density = |density: f32, axles: &mut [AxleConfig]| {
			for ax in axles {
				ax.density			= density;
			}
		};

		//
		//

		ui.collapsing("Acceleration".to_string(), |ui| {
		ui.vertical(|ui| {

		ui.add(
			Slider::new(&mut accel_cfg.vel_fwd, 0.05 ..= 400.0)
				.text("Target Speed Forward"),
		);
		ui.add(
			Slider::new(&mut accel_cfg.damping_fwd, 0.05 ..= 1000.0)
				.text("Acceleration Damping Forward"),
		);

		ui.add_space(1.0);
		
		ui.add(
			Slider::new(&mut accel_cfg.vel_bwd, 0.05 ..= 400.0)
				.text("Target Speed Backward"),
		);
		ui.add(
			Slider::new(&mut accel_cfg.damping_bwd, 0.05 ..= 1000.0)
				.text("Acceleration Damping Backward"),
		);

		ui.add_space(1.0);

		ui.add(
			Slider::new(&mut accel_cfg.damping_stop, 0.05 ..= 1000.0)
				.text("Stopping Damping"),
		);

		}); // ui.vertical
		}); // ui.collapsing

		ui.collapsing("Steering".to_string(), |ui| {
		ui.vertical(|ui| {
	
			ui.add(
				Slider::new(&mut steer_cfg.angle, 0.05 ..=180.0)
					.text("Steering Angle"),
			);
			ui.add(
				Slider::new(&mut steer_cfg.damping, 0.05 ..= 1000.0)
					.text("Steering Damping"),
			);
	
			ui.add(
				Slider::new(&mut steer_cfg.stiffness, 0.05 ..= 1000.0)
					.text("Steering Stiffness"),
			);
	
		}); // ui.vertical
		}); // ui.collapsing

		let render_wheel_params = |
			  ui					: &mut Ui
			, wheel_cfg				: &mut WheelConfig
			, axle_cfg				: &mut AxleConfig
			, section_name			: String
		| -> (bool, bool, bool, bool) {
			let mut hh_changed		= false;
			let mut r_changed		= false;
			let mut wh_density_changed = false;
			let mut axle_density_changed = false;

			ui.collapsing(section_name, |ui| {
			ui.vertical(|ui| {
	
			if ui.add(
				Slider::new(&mut wheel_cfg.hh, 0.05 ..= 1.0)
					.text("Half Height"),
			).changed() {
				hh_changed 			= true;
			}
	
			if ui.add(
				Slider::new(&mut wheel_cfg.r, 0.05 ..= 1.0)
					.text("Radius"),
			).changed() {
				r_changed 			= true;
			}

			if ui.add(
				Slider::new(&mut wheel_cfg.density, 0.05 ..= 100.0)
					.text("Wheel Density"),
			).changed() {
				wh_density_changed	= true;
			}

			if ui.add(
				Slider::new(&mut axle_cfg.density, 0.05 ..= 100.0)
					.text("Axle Density"),
			).changed() {
				axle_density_changed= true;
			}
	
			}); // ui.vertical
			}); // ui.collapsing

			(hh_changed, r_changed, wh_density_changed, axle_density_changed)
		};

		let (front_wh_hh_changed, front_wh_r_changed, front_wh_density_changed, front_axle_density_changed) =
			render_wheel_params(ui, &mut front_wheel_common, &mut front_axle_common, String::from("Front Wheels"));

		let (rear_wh_hh_changed, rear_wh_r_changed, rear_wh_density_changed, rear_axle_density_changed) =
			render_wheel_params(ui, &mut rear_wheel_common, &mut rear_axle_common, String::from("Rear Wheels"));

		let mut FR = vec![];
		let mut FL = vec![];
		let mut RR = vec![];
		let mut RL = vec![];

		for (parent, mut collider, mut mass_props_co) in q_child.iter_mut() {
			let (name_comp, tag, mass_props_rb) = q_parent.get(parent.0).unwrap();
			let name = &name_comp.name;

			let mut writeback_wheel = |
				  hh_changed
				, r_changed
				, density_changed
				, wheel_cfg	: &WheelConfig
				, wheels	: &mut [WheelConfig]
			| {
				if hh_changed {
					set_cylinder_hh(wheel_cfg.hh, &mut collider);
					set_wheels_hh(wheel_cfg.hh, wheels);
				}
				if r_changed {
					set_cylinder_r(wheel_cfg.r, &mut collider);
					set_wheels_r(wheel_cfg.r, wheels);
				}
				if density_changed {
					set_density(wheel_cfg.density, &mut mass_props_co);
					set_wheels_density(wheel_cfg.density, wheels);
				}
			};

			let mut writeback_axle = |
				  density_changed
				, axle_cfg	: &AxleConfig
				, axles		: &mut [AxleConfig]
			| {
				if density_changed {
					set_density(axle_cfg.density, &mut mass_props_co);
					set_axles_density(axle_cfg.density, axles);
				}
			};

			match tag {
				Tag::Wheel if name.starts_with("Front") => {
					writeback_wheel(
						  front_wh_hh_changed
						, front_wh_r_changed
						, front_wh_density_changed
						, &front_wheel_common
						, &mut front_wheels
					);
				},
				Tag::Axle if name.starts_with("Front") => {
					writeback_axle(
						  front_axle_density_changed
						, &front_axle_common
						, &mut front_axles
					);
				},
				Tag::Wheel if name.starts_with("Rear") => {
					writeback_wheel(
						  rear_wh_hh_changed
						, rear_wh_r_changed
						, rear_wh_density_changed
						, &rear_wheel_common
						, &mut rear_wheels
					);
				},
				Tag::Axle if name.starts_with("Rear") => {
					writeback_axle(
						  rear_axle_density_changed
						, &rear_axle_common
						, &mut rear_axles
					);
				},
				_ => (),
			}

			let to_push = (name, collider, mass_props_co, mass_props_rb);
			if name.starts_with(wheel_side_name(FRONT_RIGHT)) {
				FR.push(to_push);
			} else if name.starts_with(wheel_side_name(FRONT_LEFT)) {
				FL.push(to_push);
			} else if name.starts_with(wheel_side_name(REAR_RIGHT)) {
				RR.push(to_push);
			} else if name.starts_with(wheel_side_name(REAR_LEFT)) {
				RL.push(to_push);
			} else if name.eq("Body") {
				// thanks kpreid!
				let (name, mut collider, mut mass_props_co, mass_props_rb) = to_push;
				draw_body_params_ui_collapsing(ui, name, &mut collider, &mut mass_props_co, mass_props_rb, "Body".to_string());
			}
		}

		draw_single_wheel_params_ui_collapsing(ui, FR, wheel_side_name(FRONT_RIGHT).to_string());
		draw_single_wheel_params_ui_collapsing(ui, FL, wheel_side_name(FRONT_LEFT).to_string());
		draw_single_wheel_params_ui_collapsing(ui, RR, wheel_side_name(REAR_RIGHT).to_string());
		draw_single_wheel_params_ui_collapsing(ui, RL, wheel_side_name(REAR_LEFT).to_string());
	});

// uncomment when we need to catch a closed window
//	match out {
//		Some(response) => {
//			if response.inner == None { println!("PEWPEWPEWPEW") }; 
//		}
//		_ => ()
//	}
}

// contact info + modification. I'd rather add more info to event
/* fn display_contact_info(narrow_phase: Res<NarrowPhase>) {
	let entity1 = ...; // The first entity with a collider bundle attached.
	let entity2 = ...; // The second entity with a collider bundle attached.
	
	/* Find the contact pair, if it exists, between two colliders. */
	if let Some(contact_pair) = narrow_phase.contact_pair(entity1.handle(), entity2.handle()) {
		// The contact pair exists meaning that the broad-phase identified a potential contact.
		if contact_pair.has_any_active_contact {
			// The contact pair has active contacts, meaning that it
			// contains contacts for which contact forces were computed.
		}

		// We may also read the contact manifolds to access the contact geometry.
		for manifold in &contact_pair.manifolds {
			println!("Local-space contact normal: {}", manifold.local_n1);
			println!("Local-space contact normal: {}", manifold.local_n2);
			println!("World-space contact normal: {}", manifold.data.normal);

			// Read the geometric contacts.
			for contact_point in &manifold.points {
				// Keep in mind that all the geometric contact data are expressed in the local-space of the colliders.
				println!("Found local contact point 1: {:?}", contact_point.local_p1);
				println!("Found contact distance: {:?}", contact_point.dist); // Negative if there is a penetration.
				println!("Found contact impulse: {}", contact_point.data.impulse);
				println!("Found friction impulse: {}", contact_point.data.tangent_impulse);
			}

			// Read the solver contacts.
			for solver_contact in &manifold.data.solver_contacts {
				// Keep in mind that all the solver contact data are expressed in world-space.
				println!("Found solver contact point: {:?}", solver_contact.point);
				println!("Found solver contact distance: {:?}", solver_contact.dist); // Negative if there is a penetration.
			}
		}
	}
} */
