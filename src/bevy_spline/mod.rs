use bevy				:: { prelude :: * };
use bevy_mod_picking	:: { * };
use iyes_loopless		:: { prelude :: * };

use super::game 		:: { * };

pub mod spawn;
mod systems;
use systems				:: *;

pub use bevy_debug_text_overlay	:: { screen_print };


pub type Raw 			= splines::Spline<f32, Vec3>;
pub type Interpolation 	= splines::Interpolation<f32, Vec3>;
pub type Key 			= splines::Key<f32, Vec3>;

// wrapper for SplineRaw to have it as a Bevy Component
#[derive(Component)]
pub struct Spline(pub Raw);

impl Spline {
	pub fn set_interpolation(&mut self, id : usize, interpolation : Interpolation) {
		*self.0.get_mut(id).unwrap().interpolation = interpolation;
	}

	pub fn get_interpolation(&self, id : usize) -> &Interpolation {
		&self.0.get(id).unwrap().interpolation
	}
	
	pub fn set_control_point(&mut self, id : usize, controlp_pos : Vec3) {
		let t = controlp_pos.length();
		self.0.replace(id, |k : &Key| { Key::new(t, controlp_pos, k.interpolation) });
	}

	pub fn total_length(&self) -> f32 {
		let keys = self.keys();
		let total_keys = keys.len();
		if total_keys < 2 {
			return 0.0; // return error instead?
		}

		let mut i = 1;
		let mut total_length = 0.0;
		loop {
			total_length += (keys[i].value - keys[i - 1].value).length();
			if i + 1 == total_keys {
				break;
			} else {
				i += 1;
			}
		};

		total_length
	}

	// wrapper
	pub fn from_vec(keys : Vec<Key>) -> Self {
		Self {
			0 : Raw::from_vec(keys),
		}
	}

	// wrapper
	pub fn sample(&self, t : f32) -> Option<Vec3> {
		self.0.sample(t)
	}

	// wrapper
	pub fn clamped_sample(&self, t : f32) -> Option<Vec3> {
		self.0.clamped_sample(t)
	}

	// wrapper
	pub fn add(&mut self, key : Key) {
		self.0.add(key);
	}

	// wrapper
	pub fn len(&self) -> usize {
		self.0.len()
	}

	// wrapper
	pub fn get_key_id(&self, t_in : f32) -> usize {
		let keys = self.0.keys();
		keys.iter().position(|&key| key.t == t_in).unwrap()
	}

	// wrapper
	pub fn keys(&self) -> &[Key] {
		self.0.keys()
	}
}

#[derive(Component)]
pub struct Tangent {
	pub global_id 	: usize,
	pub local_id 	: usize,
}

#[derive(Component)]
pub enum ControlPoint {
	ID(usize)
}

#[derive(Component)]
pub struct ControlPointPolyline;

pub struct BevySplinePlugin;

impl Plugin for BevySplinePlugin {
	fn build(&self, app: &mut App) {
		let clear_color = ClearColor(
			Color::rgb(
				0xF9 as f32 / 255.0,
				0xF9 as f32 / 255.0,
				0xFF as f32 / 255.0,
			));

        app	
			.add_system_to_stage(CoreStage::PostUpdate, on_tangent_moved)
			.add_system_to_stage(CoreStage::PostUpdate, on_control_point_moved)
 			;
	}
}

