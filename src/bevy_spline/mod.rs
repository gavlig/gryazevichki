use bevy				:: { prelude :: * };

use super::game 		:: { * };

pub mod spawn;
mod systems;
use systems				:: *;

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
	
	pub fn set_control_point(&mut self, t : f32, controlp_pos : Vec3) {
		let keys = self.keys();
		let id = keys.iter().position(|&k| k.t.to_bits() == k.t.to_bits()).unwrap();
		self.0.replace(id, |k : &Key| { Key::new(t, controlp_pos, k.interpolation) });
	}

	pub fn set_control_point_by_id(&mut self, id : usize, t : f32, controlp_pos : Vec3) {
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

	pub fn calculate_t_for_pos(&self, pos : Vec3) -> f32 {
		let keys = self.keys();
		let total_keys = keys.len();
		if total_keys == 0 {
			return pos.length();
		}

		let mut i = 1;
		let mut total_length = 0.0;

		let new_t =
		loop {
			let pos0 = keys[i - 1].value;
			let pos1 = keys[i].value;
			let delta = pos1 - pos0;
			let dir = delta.normalize();

			let dir_x = dir.x > dir.y && dir.x > dir.z;
			let dir_y = dir.y > dir.z && dir.y > dir.x;
			let dir_z = dir.z > dir.x && dir.z > dir.y;

			let mid_x = pos0.x < pos.x && pos.x < pos1.x;
			let mid_y = pos0.y < pos.y && pos.y < pos1.y;
			let mid_z = pos0.z < pos.z && pos.z < pos1.z;
			
			let new_pos_delta = pos - pos0;

			if dir_x && mid_x 
			|| dir_y && mid_y
			|| dir_z && mid_z {
				break total_length + new_pos_delta.length();
			}

			total_length += delta.length();
			if i + 1 == total_keys {
				let new_pos_delta = pos - pos1;
				break total_length + new_pos_delta.length();
			} else {
				i += 1;
			}
		};

		new_t
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
	T(f32)
}

#[derive(Component)]
pub struct SplinePolyline;

#[derive(Component)]
pub struct ControlPointPolyline;

pub struct BevySplinePlugin;

impl Plugin for BevySplinePlugin {
	fn build(&self, app: &mut App) {
        app	
			.add_system_to_stage(CoreStage::PostUpdate, on_tangent_moved)
			.add_system_to_stage(CoreStage::PostUpdate, on_control_point_moved)
 			;
	}
}

