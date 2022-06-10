use bevy				:: { prelude :: *, window :: PresentMode };
use bevy_mod_picking	:: { * };
use bevy_atmosphere		:: { * };
use iyes_loopless		:: prelude :: { * };

use std					:: { path::PathBuf };
use serde				:: { Deserialize, Serialize };

use super::vehicle		:: { * };
use super::ui			:: { * };
use super::draggable	:: { * };
use super::bevy_spline	:: { BevySplinePlugin };
use super::herringbone	:: { HerringbonePlugin };

pub mod spawn;
mod systems;
use systems				:: *;


pub use super::draggable :: { Draggable, DraggableActive };

pub type PickingObject = bevy_mod_picking::RayCastSource<PickingRaycastSet>;

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

pub struct SpawnArguments<'a0, 'a1, 'b0, 'b1, 'c, 'd, 'e> {
	pub meshes				: &'a0 mut ResMut<'a1, Assets<Mesh>>,
	pub materials			: &'b0 mut ResMut<'b1, Assets<StandardMaterial>>,
	pub commands			: &'c mut Commands<'d, 'e>
}

#[derive(Debug, Clone, Copy)]
pub struct RespawnableEntity {
	pub entity	: Entity,
	pub respawn	: bool
}

impl Default for RespawnableEntity {
	fn default() -> Self {
		Self {
			  entity		: Entity::from_bits(0)
			, respawn		: false
		}
	}
}

#[derive(Default)]
pub struct DespawnResource {
	pub entities: Vec<Entity>,
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PhysicsConfig {
	  pub fixed				: bool
	, pub density			: f32
	, pub mass				: f32
	, pub friction			: f32
	, pub restitution		: f32
	, pub lin_damping		: f32
	, pub ang_damping		: f32
}

impl Default for PhysicsConfig {
	fn default() -> Self {
		Self {
			  fixed			: false
			, density		: 1.0
			, mass			: 0.0 // calculated at runtime
			, friction		: 0.5
			, restitution	: 0.0
			, lin_damping	: 0.0
			, ang_damping	: 0.0
		}
	}
}

#[derive(Component)]
pub struct Gizmo;

#[derive(Component)]
pub struct Tile;

#[derive(Default)]
pub struct GryazevichkiPickingHighlight;
impl Highlightable for GryazevichkiPickingHighlight {
    type HighlightAsset = StandardMaterial;

    fn highlight_defaults(
        mut materials: Mut<Assets<Self::HighlightAsset>>,
    ) -> DefaultHighlighting<Self> {
        DefaultHighlighting {
            hovered: materials.add(StandardMaterial{ base_color: Color::rgb(0.35, 0.35, 0.35).into(), unlit: true, ..default() }),
            pressed: materials.add(StandardMaterial{ base_color: Color::rgb(0.90, 0.90, 0.90).into(), unlit: true, ..default() }),
            selected: materials.add(StandardMaterial{ base_color: Color::rgb(0.35, 0.35, 0.75).into(), unlit: true, ..default() }),
        }
    }
}

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

			.insert_resource(WindowDescriptor { present_mode : PresentMode::Mailbox, ..default() })
			
		
 			.add_plugin		(HerringbonePlugin)
            .add_plugin		(UiPlugin)
            .add_plugin		(VehiclePlugin)
			.add_plugin		(BevySplinePlugin)
			.add_plugin		(DraggablePlugin)

 			.add_plugin		(PickingPlugin)
         	.add_plugin		(InteractablePickingPlugin)
 			.add_plugin		(CustomHighlightPlugin(GryazevichkiPickingHighlight))

 			.add_startup_system(setup_cursor_visibility_system)
 			.add_startup_system(setup_lighting_system)
 			.add_startup_system(setup_world_system)
 			.add_startup_system_to_stage(StartupStage::PostStartup, setup_camera_system)

 			// input
 			.add_system		(cursor_visibility_system)
			.add_system		(input_misc_system)
			.add_system		(vehicle_controls_system)

			.add_system_to_stage(CoreStage::PostUpdate, despawn_system)
 			;
	}
}

