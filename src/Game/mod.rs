use bevy			::	{ prelude :: * };

use std				:: 	{ path::PathBuf };
use serde			::	{ Deserialize, Serialize };

pub mod systems;
pub use systems		:: *;
pub mod Vehicle;
pub use Vehicle		:: *;
pub mod Ui;
pub use Ui			:: *;			

mod spawn;
pub use spawn::Tile;
pub use spawn::HerringboneStepRequest;
pub use spawn::HerringboneIO;
pub use spawn::Herringbone;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameMode {
    Editor,
    InGame,
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

#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub enum Orientation2D {
	Horizontal,
	Vertical
}

impl Orientation2D {
	pub fn flip(&mut self) {
		let flipped = if *self == Orientation2D::Vertical { Orientation2D::Horizontal } else { Orientation2D::Vertical };
		*self = flipped;
	}
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

#[derive(Component)]
pub struct ObjectRoot;

#[derive(Component)]
pub enum SplineTangent {
	ID(usize)
}

#[derive(Component)]
pub enum SplineControlPoint {
	ID(usize)
}

#[derive(Component)]
pub struct Gizmo;