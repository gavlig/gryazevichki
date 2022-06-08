mod motor;
mod spawn;
mod save_load;
mod systems;

pub use save_load::save_vehicle_config_system;
pub use save_load::load_vehicle_config_system;
pub use save_load::load_vehicle_config;

pub use spawn::vehicle as spawn;
pub use systems::respawn_vehicle_system as respawn_vehicle_system;

use bevy		::	prelude :: *;
use bevy_rapier3d::	prelude :: *;
use serde		::	{ Deserialize, Serialize };

use super		::	Game :: { PhysicsConfig, SideX, SideZ };
pub use super	::	Game :: { GameState, RespawnableEntity, NameComponent };

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
	  accel		: Option<AcceleratorConfig>
	, steer		: Option<SteeringConfig>
	, body 		: Option<BodyConfig>
	, bophys	: Option<PhysicsConfig>
	, axles		: [Option<AxleConfig>;      WHEELS_MAX as usize]
	, axphys	: [Option<PhysicsConfig>;   WHEELS_MAX as usize]
	, wheels	: [Option<WheelConfig>;     WHEELS_MAX as usize]
	, whphys	: [Option<PhysicsConfig>;   WHEELS_MAX as usize]
}

impl Config {
	fn version() -> u32 { return 0; }
}

#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub enum Part {
	Wheel,
	Axle,
	Body,
	_WheelJoint, // these are not exposed to engine anymore
	_AxleJoint,
}

// TODO: all this looks like a bad design, most likely i need a different approach
use usize as WheelSideType;
pub const FRONT_RIGHT		: WheelSideType = 0;
pub const FRONT_LEFT		: WheelSideType = 1;

pub const REAR_RIGHT		: WheelSideType = 2;
pub const REAR_LEFT			: WheelSideType = 3;

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

#[allow(dead_code)]
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

#[allow(dead_code)]
fn wheel_side_rotation(side: WheelSideType) -> Quat {
	let default_rotation	= Quat::IDENTITY;
	let flip_side			= Quat::from_rotation_y(std::f32::consts::PI);

	match side {
		FRONT_RIGHT			=> default_rotation,
		FRONT_LEFT			=> flip_side,
		REAR_RIGHT			=> default_rotation,
		REAR_LEFT 			=> flip_side,
		WHEELS_MAX			=> panic!("Max shouldn't be used as a wheel side!"),
		_					=> panic!("Only 4 sides are supported currently: 0 - 3 or FrontRight FrontLeft RearRight RearLeft"),
	}
}

pub const WHEEL_SIDES: &'static [WheelSideType] = &[
	  FRONT_RIGHT
	, FRONT_LEFT
	, REAR_LEFT
	, REAR_RIGHT
];

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WheelConfig {
	  pub hh					: f32
	, pub r						: f32
//	, model_path				: Option<&'static str>
}

impl Default for WheelConfig {
	fn default() -> Self {
		Self {
			  hh				: 0.5
			, r					: 0.8
//			, model_path		: Some("corvette/wheel/corvette_wheel.gltf#Scene0")
		}
	}
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AxleConfig {
	  pub half_size				: Vec3
	, pub wheel_offset			: Vec3
	, pub auto_offset			: bool
}

impl Default for AxleConfig {
	fn default() -> Self {
		Self {
			  half_size			: Vec3::new(0.1, 0.2, 0.1)
			, wheel_offset		: Vec3::new(0.8, 0.0, 0.0)
			, auto_offset		: true
		}
	}
}

impl AxleConfig {
	fn wheel_offset(self, side: WheelSideType) -> Vec3 {
		wheel_side_offset		(side, self.wheel_offset)
	}
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BodyConfig {
	  pub half_size				: Vec3
	, pub axle_offset			: Vec3
	, pub auto_offset			: bool
}

impl Default for BodyConfig {
	fn default() -> Self {
		Self {
			  half_size			: Vec3::new(0.5, 0.5, 1.0)
			, axle_offset		: Vec3::new(0.8, 0.8, 1.4)
			, auto_offset		: true
		}
	}
}

impl BodyConfig {
	pub fn axle_offset(self, side: WheelSideType) -> Vec3 {
		wheel_side_offset		(side, self.axle_offset)
	}
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AcceleratorConfig {
	  pub vel_fwd				: f32
	, pub vel_bwd				: f32
	, pub damping_fwd			: f32
	, pub damping_bwd			: f32
	, pub damping_stop			: f32
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
	  pub stiffness				: f32
	, pub stiffness_release		: f32
	, pub damping				: f32
	, pub damping_release		: f32
	, pub angle					: f32
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

// TODO: probably split input processing from gamelogic here
pub fn vehicle_controls_system(
		key			: Res<Input<KeyCode>>,
		game		: ResMut<GameState>,
		q_accel_cfg	: Query<&AcceleratorConfig>,
		q_steer_cfg	: Query<&SteeringConfig>,
	mut	query		: Query<&mut ImpulseJoint>,
) {
	let fr_axle_joint	= game.axles[FRONT_RIGHT];
	let fl_axle_joint	= game.axles[FRONT_LEFT];

	let rr_wheel_joint 	= game.wheels[REAR_RIGHT];
	let rl_wheel_joint 	= game.wheels[REAR_LEFT];

	let body = match game.body {
		Some(b) => b,
		None => return,
	};

	let accel_cfg = match q_accel_cfg.get(body.entity) {
		Ok(c) => c,
		Err(_) => return,
	};

	let steer_cfg = match q_steer_cfg.get(body.entity) {
		Ok(c) => c,
		Err(_) => return,
	};

	if key.just_pressed(KeyCode::W) {
		motor::velocity	(accel_cfg.vel_fwd, accel_cfg.damping_fwd, rr_wheel_joint, &mut query);
		motor::velocity	(accel_cfg.vel_fwd, accel_cfg.damping_fwd, rl_wheel_joint, &mut query);
	} else if key.just_released(KeyCode::W) {
		motor::velocity	(0.0, accel_cfg.damping_stop, rr_wheel_joint, &mut query);
		motor::velocity	(0.0, accel_cfg.damping_stop, rl_wheel_joint, &mut query);
	}
	
	if key.just_pressed(KeyCode::S) {
		motor::velocity	(-accel_cfg.vel_bwd, accel_cfg.damping_bwd, rr_wheel_joint, &mut query);
		motor::velocity	(-accel_cfg.vel_bwd, accel_cfg.damping_bwd, rl_wheel_joint, &mut query);
	} else if key.just_released(KeyCode::S) {
		motor::velocity	(0.0, accel_cfg.damping_stop, rr_wheel_joint, &mut query);
		motor::velocity	(0.0, accel_cfg.damping_stop, rl_wheel_joint, &mut query);
	}
 
	let steer_angle 	= steer_cfg.angle;
	let stiffness 		= steer_cfg.stiffness;
	let stiffness_release = steer_cfg.stiffness_release;
	let damping 		= steer_cfg.damping;
	let damping_release = steer_cfg.damping_release;
	if key.just_pressed(KeyCode::D) {
		motor::steer	(-steer_angle, stiffness, damping, fr_axle_joint, &mut query);
		motor::steer	(-steer_angle, stiffness, damping, fl_axle_joint, &mut query);
	} else if key.just_released(KeyCode::D) {
		motor::steer	(0.0, stiffness_release, damping_release, fr_axle_joint, &mut query);
		motor::steer	(0.0, stiffness_release, damping_release, fl_axle_joint, &mut query);
	}

 	if key.just_pressed(KeyCode::A) {
		motor::steer	(steer_angle, stiffness, damping, fr_axle_joint, &mut query);
		motor::steer	(steer_angle, stiffness, damping, fl_axle_joint, &mut query);
	} else if key.just_released(KeyCode::A) {
		motor::steer	(0.0, stiffness_release, damping_release, fr_axle_joint, &mut query);
		motor::steer	(0.0, stiffness_release, damping_release, fl_axle_joint, &mut query);
	}
}

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app	.add_system_to_stage(CoreStage::PostUpdate, respawn_vehicle_system)

			.add_system_to_stage(CoreStage::Last, save_vehicle_config_system)
 			.add_system_to_stage(CoreStage::Last, load_vehicle_config_system)
			;
    }
}