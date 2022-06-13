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
	pub fn set_interpolation(&mut self, t : f32, interpolation : Interpolation) {
		let id = self.get_key_id(t).unwrap();
		*self.0.get_mut(id).unwrap().interpolation = interpolation;
	}

	pub fn get_interpolation(&self, t : f32) -> &Interpolation {
		let id = self.get_key_id(t).unwrap();
		&self.0.get(id).unwrap().interpolation
	}
	
	pub fn set_control_point(&mut self, t : f32, controlp_pos : Vec3) {
		let keys = self.keys();
		let id = self.get_key_id(t).unwrap();
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
		if total_keys < 2 {
			return pos.length();
		}

		let mut i = 1;
		let mut total_length = 0.0;

		let new_t =
		loop {
			let key0 = keys[i - 1];
			let key1 = keys[i];
			let pos0 = key0.value;
			let pos1 = key1.value;

			let segment_len_sq = pos0.distance_squared(pos1);
			let pos_in_segment_sq = pos0.distance_squared(pos);
			let epsilon = 0.05;
			println!("[{}] key0 t: {:.3} v: [{:.3} {:.3} {:.3}] key1 t: {:.3} v: [{:.3} {:.3} {:.3}]", i, key0.t, key0.value.x, key0.value.y, key0.value.z, key1.t, key1.value.x, key1.value.y, key1.value.z);
			println!("pos_in_segment_sq: {} segment_len_sq: {}", pos_in_segment_sq, segment_len_sq);
			if pos_in_segment_sq <= epsilon {
				println!("key0.t");
				break key0.t;
			}

			if (pos_in_segment_sq - segment_len_sq).abs() <= epsilon {
				println!("key1.t");
				break key1.t;
			}

			let delta = pos1 - pos0;
			let new_pos_delta = pos - pos0;

			if pos_in_segment_sq < segment_len_sq {
				println!("mid {}", total_length + new_pos_delta.length());
				break total_length + new_pos_delta.length();
			}

			if i + 1 == total_keys {
				println!("i + 1 == total_keys so output is {}", total_length + new_pos_delta.length());
				break total_length + new_pos_delta.length();
			} else {
				println!("i += 1");
				i += 1;
			}

			total_length += delta.length();
			println!("total_length: {}", total_length);
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
	pub fn get_key_id(&self, t_in : f32) -> Option<usize> {
		let keys = self.0.keys();
		keys.iter().position(|&key| { println!("key.t: {:.3} t: {:.3}", key.t, t_in); (key.t - t_in).abs() <= f32::EPSILON })
	}

	// wrapper
	pub fn keys(&self) -> &[Key] {
		self.0.keys()
	}
}

#[derive(Component)]
pub struct Tangent {
	pub id : usize,
}

#[derive(Component)]
pub enum ControlPoint {
	T(f32)
}

#[derive(Component)]
pub struct SplinePolyline;

#[derive(Component)]
pub struct ControlPointPolyline;

#[derive(Component, Clone, Copy)]
pub enum RoadWidth {
	W(f32)
}

pub struct BevySplinePlugin;

impl Plugin for BevySplinePlugin {
	fn build(&self, app: &mut App) {
        app	
			.add_system(draw_road)
			.add_system_to_stage(CoreStage::PostUpdate, on_tangent_moved)
			.add_system_to_stage(CoreStage::PostUpdate, on_control_point_moved)
 			;
	}
}

