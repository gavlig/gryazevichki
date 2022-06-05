use bevy			::	{ prelude :: * };
use bevy_mod_picking::	{ PickingRaycastSet };
use bevy_atmosphere	::	{ * };
use iyes_loopless	::	prelude :: { * };

use std				:: 	{ path::PathBuf };
use serde			::	{ Deserialize, Serialize };

mod Vehicle;
use Vehicle			:: *;
mod Ui;
use Ui				:: *;
mod Herringbone;
use Herringbone		:: *;

mod spawn;
mod systems;
use systems			:: *;
mod draggable;
use draggable		:: *;

pub type PickingObject = bevy_mod_raycast::RayCastSource<PickingRaycastSet>;

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

#[allow(dead_code)]
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
	entity	: Entity,
	respawn	: bool
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

pub struct SpawnArguments<'a0, 'a1, 'b0, 'b1, 'c, 'd, 'e> {
	pub meshes					: &'a0 mut ResMut<'a1, Assets<Mesh>>,
	pub materials				: &'b0 mut ResMut<'b1, Assets<StandardMaterial>>,
	pub commands				: &'c mut Commands<'d, 'e>
}

pub type SplineRaw 				= splines::Spline<f32, Vec3>;
pub type SplineInterpolation 	= splines::Interpolation<f32, Vec3>;
pub type SplineKey 				= splines::Key<f32, Vec3>;

// wrapper for SplineRaw to have it as a Bevy Component
#[derive(Component)]
pub struct Spline(pub SplineRaw);

impl Spline {
	pub fn set_interpolation(&mut self, id : usize, interpolation : SplineInterpolation) {
		*self.0.get_mut(id).unwrap().interpolation = interpolation;
	}
	
	pub fn set_control_point(&mut self, id : usize, controlp_pos : Vec3) {
		let t = controlp_pos.z;
		self.0.replace(id, |k : &SplineKey| { SplineKey::new(t, controlp_pos, k.interpolation) });
	}

	// wrapper
	pub fn from_vec(keys: Vec<SplineKey>) -> Self {
		Self {
			0 : SplineRaw::from_vec(keys),
		}
	}

	// wrapper
	pub fn sample(&self, t: f32) -> Option<Vec3> {
		self.0.sample(t)
	}

	// wrapper
	pub fn add(&mut self, key: SplineKey) {
		self.0.add(key);
	}

	// wrapper
	pub fn len(&self) -> usize {
	  self.0.len()
	}
}

#[derive(Component)]
pub struct RootHandle;

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

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
		let clear_color = ClearColor(
			Color::rgb(
				0xF9 as f32 / 255.0,
				0xF9 as f32 / 255.0,
				0xFF as f32 / 255.0,
			));

        app	.add_loopless_state(GameMode::InGame)

			.insert_resource(clear_color)
			.insert_resource(Msaa			::default())
			.insert_resource(AtmosphereMat	::default()) // Default Earth sky

			.insert_resource(GameState		::default())
			.insert_resource(DespawnResource::default())
			
		
			.add_plugin		(HerringbonePlugin)
            .add_plugin		(UiPlugin)
            .add_plugin		(VehiclePlugin)

			.add_startup_system(setup_cursor_visibility_system)
			.add_startup_system(setup_lighting_system)
			.add_startup_system(setup_world_system)
			.add_startup_system_to_stage(StartupStage::PostStartup, setup_camera_system)

			// input
			.add_system		(cursor_visibility_system)
			.add_system		(input_misc_system)
			.add_system		(vehicle_controls_system)

			.add_system_to_stage(CoreStage::PostUpdate, despawn_system)

			.add_system_to_stage(CoreStage::PostUpdate, dragging_start_system)
			.add_system_to_stage(CoreStage::PostUpdate, dragging_system)
			.add_system_to_stage(CoreStage::PostUpdate, dragging_stop_system)
			;
    }
}

