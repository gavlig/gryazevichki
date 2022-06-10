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

#[derive(Component)]
pub struct Control {
	pub next			: bool,
	pub animate			: bool,
	pub reset			: bool,
	pub instant			: bool,
	pub new_spline_point: bool,
	pub debug			: bool,
	pub last_update		: f64,
	pub anim_delay_sec	: f64,
}

impl Default for Control {
	fn default() -> Self {
		Self {
			next		: false,
			animate		: false,
			reset		: false,
			instant		: false,
			new_spline_point : false,
			debug		: false,
			last_update	: 0.0,
			anim_delay_sec: 0.001,
		}
	}
}

#[derive(Component)]
pub struct TileState {
	pub x 				: u32,
	pub z 				: u32,
	pub iter			: u32,
	pub orientation		: Orientation2D,
	pub finished_hor	: bool,
	pub finished		: bool,
	pub prev_spline_p	: Option<Vec3>,
}

impl Default for TileState {
	fn default() -> Self {
		Self {
			x 			: 0,
			z 			: 0,
			iter		: 0,
			orientation	: Orientation2D::Horizontal,
			finished_hor: false,
			finished	: false,
			prev_spline_p : None,
		}
	}
}

impl TileState {
	#[allow(dead_code)]
	pub fn set_default(&mut self) {
		*self			= Self::default();
	}

	pub fn reset_changed(&mut self) {
		self.iter 		= 0;
		self.x 			= 0;
		self.z 			= 0;
		self.finished 	= false;
		self.finished_hor = false;
	}

	pub fn clone(&self) -> Self {
		Self {
			x 			: self.x,
			z 			: self.z,
			iter		: self.iter,
			orientation	: self.orientation,
			finished_hor: self.finished_hor,
			finished	: self.finished,
			prev_spline_p : self.prev_spline_p.clone(),
		}
	}
}

#[derive(Component)]
pub struct Herringbone2Config {
	pub body_type 		: RigidBody,
	pub hsize 			: Vec3,
	pub seam			: f32,
	
	pub width			: f32,
	pub limit_z			: f32,
	pub limit_mz		: f32,
	pub limit_iter		: u32,
	pub init_tangent_offset : f32,

	pub root_entity		: Entity,

	// cant copy
	pub mesh			: Handle<Mesh>,
	pub material		: Handle<StandardMaterial>,
}

impl Default for Herringbone2Config {
	fn default() -> Self {
		Self {
			body_type 	: RigidBody::Fixed,
			hsize 		: Vec3::new(0.2 / 2.0, 0.05 / 2.0, 0.4 / 2.0),
			seam		: 0.01,
			
			width		: 4.0,
			limit_z		: 8.0,
			limit_mz 	: 0.0,
			limit_iter	: 100,
			init_tangent_offset : 1.0,

			root_entity	: Entity::from_raw(0),

			mesh		: Handle::<Mesh>::default(),
			material	: Handle::<StandardMaterial>::default(),
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
			seam		: self.seam,
			
			width		: self.width,
			limit_z		: self.limit_z,
			limit_mz 	: self.limit_mz,
			limit_iter	: self.limit_iter,
			init_tangent_offset : self.init_tangent_offset,

			root_entity	: self.root_entity,

			mesh		: self.mesh.clone_weak(),
			material	: self.material.clone_weak(),
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