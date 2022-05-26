use bevy			::	{ prelude :: * };
use bevy			::	{ app::AppExit };
use bevy_rapier3d	::	{ prelude :: * };
use bevy_fly_camera	::	{ FlyCamera };

use std				:: 	{ path::PathBuf };
use serde			::	{ Deserialize, Serialize };

pub mod systems;
pub use systems		:: *;
pub mod Vehicle;
pub use Vehicle		:: *;
pub mod Ui;
pub use Ui			:: *;			

mod spawn;
pub use spawn::HerringboneStepRequest;

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