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
	pub dir				: Direction2D,
	pub iter			: usize,
	pub pattern_iter	: usize,
	pub t				: f32,
	pub pos				: Vec3,
	pub finished		: bool,
}

impl Default for BrickRoadProgressState {
	fn default() -> Self {
		Self {
			dir			: Direction2D::Up,
			iter		: 0,
			pattern_iter: 0,
			t			: 0.0,
			pos			: Vec3::Y * 0.5, // VERTICALITY
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
		self.iter == 0
	}

	pub fn set_next_direction(&mut self, init_dir: Direction2D) {
		self.dir.set_next_direction(init_dir);
		// if self.dir == Direction2D::Up || self.dir == Direction2D::Down {
		// 	self.pattern_iter = if self.pattern_iter == 0 { 1 } else { 0 };
		// }
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
	pub length			: f32,

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
			
			width		: 2.0,
			length		: 6.0,

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
			length		: self.length,

			root_entity	: self.root_entity,

			mesh		: self.mesh.clone_weak(),
			material	: self.material.clone_weak(),
			material_dbg: self.material_dbg.clone_weak(),
		}
	}
}

#[derive(Component)]
pub struct Herringbone2TileFilterInfo {
	pub spline_p : Vec3,
	pub road_halfwidth_rotated : Vec3,
	pub left_border : Vec3,
	pub right_border : Vec3,
	pub pos : Vec3,
	pub t : f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction2D {
	Up,
	Right,
	Down,
	Left
}

impl Direction2D {
	pub fn next_direction(&self, init_dir: Self) -> Self {
		match self {
			Direction2D::Right => Direction2D::Up,
			Direction2D::Left  => Direction2D::Up,
			Direction2D::Up    => { if init_dir != Direction2D::Left { Direction2D::Left } else { Direction2D::Right } },
			Direction2D::Down  => Direction2D::Down, // unused for now
		}
	}

	pub fn set_next_direction(&mut self, init_dir: Self) {
		*self = self.next_direction(init_dir)
	}

	pub fn is_horizontal(&self) -> bool {
		*self == Direction2D::Left || *self == Direction2D::Right
	}

	pub fn is_vertical(&self) -> bool {
		!self.is_horizontal()
	}
}

pub struct HerringbonePlugin;

impl Plugin for HerringbonePlugin {
    fn build(&self, app: &mut App) {
        app	.add_system	(brick_road_system)
			.add_system	(brick_road_system_debug)
			.add_system_to_stage(CoreStage::PostUpdate, on_spline_tangent_moved)
			.add_system_to_stage(CoreStage::PostUpdate, on_spline_control_point_moved)
            ;
    }
}