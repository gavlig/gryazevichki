use bevy			:: { prelude :: * };
use bevy_rapier3d	:: { prelude :: * };

use super           :: { game :: * };
use super           :: { bevy_spline :: * };

mod systems;
use systems			:: { * };

pub mod spawn;

#[derive(Component)]
pub struct Herringbone2;

/* coming...
#[derive(Component)]
pub struct Herringbone3;
*/

#[derive(Component, Clone, Copy)]
pub struct HerringboneControl {
	pub spawn_tile		: bool,
	pub animate			: bool,
	pub clean_tiles		: bool,
	pub instant			: bool,
	pub debug			: u32,
	pub visual_debug	: bool,
	pub verbose			: bool,
	pub very_verbose	: bool,
	pub dry_run			: bool,
	pub looped			: bool,
	pub last_update		: f64,
	pub anim_delay_sec	: f64,
}

impl Default for HerringboneControl {
	fn default() -> Self {
		Self {
			spawn_tile	: false,
			animate		: false,
			clean_tiles	: false,
			instant		: false,
			debug		: 0,
			visual_debug: true,
			verbose		: false,
			very_verbose: false,
			dry_run		: false,
			looped		: false,
			last_update	: 0.0,
			anim_delay_sec: 0.001,
		}
	}
}

impl HerringboneControl {
	pub fn respawn_all_tiles_instantly(&mut self) {
		self.clean_tiles = true;
		self.spawn_tile = true;
		self.instant 	= true;
	}
}

#[derive(Component, Clone, Copy)]
pub struct BrickRoadProgressState {
	pub row_id			: usize,
	pub column_id		: usize,
	pub t				: f32,
	pub pos				: Vec3,
	pub max_spline_offset : f32,
	pub min_spline_offset : f32,
	pub finished		: bool,
}

impl Default for BrickRoadProgressState {
	fn default() -> Self {
		Self {
			row_id		: 0,
			column_id	: 0,
			t			: 0.0,
			pos			: Vec3::Y * 0.5, // VERTICALITY
			max_spline_offset : 0.0,
			min_spline_offset : 0.0,
			finished	: false,
		}
	}
}

impl BrickRoadProgressState {
	#[allow(dead_code)]
	pub fn set_default(&mut self) {
		*self			= Self::default();
	}

	pub fn hasnt_started(&self) -> bool {
		self.row_id == 0 && self.column_id == 0
	}
}

#[derive(Default, Clone, Copy)]
pub struct TileRowIterState {
	pub t 				: f32,
	pub tile_p 			: Vec3,
	pub tile_r			: Quat,
	pub spline_p 		: Vec3,
	pub spline_r 		: Quat,
}

#[derive(Component)]
pub struct Herringbone2Config {
	pub body_type 		: RigidBody,
	pub hsize 			: Vec3,
	pub hseam			: f32,
	
	pub width			: f32,
	pub t_max			: f32,
	pub t_min			: f32,
	pub iter_max		: u32,
	pub init_tangent_offset : f32,

	pub root_entity		: Entity,

	// cant copy
	pub mesh			: Handle<Mesh>,
	pub material		: Handle<StandardMaterial>,
	pub material_dbg	: Handle<StandardMaterial>,
}

impl Default for Herringbone2Config {
	fn default() -> Self {
		Self {
			body_type 	: RigidBody::Fixed,
			hsize 		: Vec3::new(0.2 / 2.0, 0.05 / 2.0, 0.4 / 2.0),
			hseam		: 0.01,
			
			width		: 4.0,
			t_max		: 6.0,
			t_min 		: 0.0,
			iter_max	: 100,
			init_tangent_offset : 1.0,

			root_entity	: Entity::from_raw(0),

			mesh		: Handle::<Mesh>::default(),
			material	: Handle::<StandardMaterial>::default(),
			material_dbg: Handle::<StandardMaterial>::default(),
		}
	}
}

impl Herringbone2Config {
	#[allow(dead_code)]
	pub fn set_default(&mut self) {
		*self			= Self::default();
	}

	pub fn clone(&self) -> Self {
		Self {
			body_type 	: self.body_type,
			hsize 		: self.hsize,
			hseam		: self.hseam,
			
			width		: self.width,
			t_max		: self.t_max,
			t_min 		: self.t_min,
			iter_max	: self.iter_max,
			init_tangent_offset : self.init_tangent_offset,

			root_entity	: self.root_entity,

			mesh		: self.mesh.clone_weak(),
			material	: self.material.clone_weak(),
			material_dbg: self.material_dbg.clone_weak(),
		}
	}
}

pub struct HerringbonePlugin;

// This plugin is responsible to control the game audio
impl Plugin for HerringbonePlugin {
    fn build(&self, app: &mut App) {
        app	.add_system	(brick_road_system)
			.add_system_to_stage(CoreStage::PostUpdate, on_spline_tangent_moved)
			.add_system_to_stage(CoreStage::PostUpdate, on_spline_control_point_moved)
            ;
    }
}