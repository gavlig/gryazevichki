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

	pub fn set_interpolation_from_id(&mut self, id : usize, interpolation : Interpolation) {
		*self.0.get_mut(id).unwrap().interpolation = interpolation;
	}

	pub fn get_interpolation(&self, t : f32) -> &Interpolation {
		let id = self.get_key_id(t).unwrap();
		&self.0.get(id).unwrap().interpolation
	}

	pub fn set_control_point(&mut self, t : f32, controlp_pos : Vec3) {
		let id = self.get_key_id(t).unwrap();
		self.0.replace(id, |k : &Key| { Key::new(t, controlp_pos, k.interpolation) });
	}

	pub fn set_control_point_by_id(&mut self, id : usize, t : f32, controlp_pos : Vec3) {
		self.0.replace(id, |k : &Key| { Key::new(t, controlp_pos, k.interpolation) });
	}

	pub fn set_control_point_t(&mut self, id : usize, t : f32) {
		self.0.replace(id, |k : &Key| { Key::new(t, k.value, k.interpolation) });
	}

	pub fn set_control_point_pos(&mut self, id : usize, controlp_pos : Vec3) {
		*self.0.get_mut(id).unwrap().value = controlp_pos;
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

		let mut key_id = 1;
		let mut total_length = 0.0;

		let new_t =
		loop {
			let key0 = keys[key_id - 1];
			let key1 = keys[key_id];
			let pos0 = key0.value;
			let pos1 = key1.value;

			let segment_len_sq = pos0.distance_squared(pos1);
			let pos_in_segment_sq = pos0.distance_squared(pos);
			let epsilon = 0.05;
			println!("[{}] key0 t: {:.3} v: [{:.3} {:.3} {:.3}] key1 t: {:.3} v: [{:.3} {:.3} {:.3}]", key_id, key0.t, key0.value.x, key0.value.y, key0.value.z, key1.t, key1.value.x, key1.value.y, key1.value.z);
			println!("pos_in_segment_sq: {} segment_len_sq: {}", pos_in_segment_sq, segment_len_sq);
			if pos_in_segment_sq <= epsilon {
				println!("key0.t");
				break key0.t;
			}

			if (pos_in_segment_sq - segment_len_sq).abs() <= epsilon {
				println!("key1.t");
				break key1.t;
			}

			let new_pos_delta = pos - pos0;

			if pos_in_segment_sq < segment_len_sq {
				println!("mid {}", total_length + new_pos_delta.length());
				break total_length + new_pos_delta.length();
			}

			if key_id + 1 == total_keys {
				println!("i + 1 == total_keys so output is {}", total_length + new_pos_delta.length());
				break total_length + new_pos_delta.length();
			} else {
				println!("i += 1");
				key_id += 1;
			}

			total_length += self.calculate_segment_length(key_id);
			println!("total_length: {}", total_length);
		};

		new_t
	}

	// calculate length of segment [key_id - 1, key_id]
	pub fn calculate_segment_length(&self, key_id : usize) -> f32 {
		if key_id < 1 {
			return 0.0;
		}

		let keys = self.keys();
		let key0 = keys[key_id - 1];
		let key1 = keys[key_id - 0]; // assume we never have invalid key_id

		let t0 = key0.t;
		let t1 = key1.t;
		let t_range = t1 - t0;
		let t_delta = t_range / 10.0; // 10 samples per meter

		let mut segment_length = 0.0;
		let mut prev_p = key0.value;
		let mut t = t0 + t_delta;
		println!("calculate_segment_length t0 {:.3} t1 {:.3} t_delta {:.3}", t0, t1, t_delta);
		while t < t1 {
			let new_p = self.sample(t).unwrap();
			segment_length += (new_p - prev_p).length();
			prev_p = new_p;
			println!("[t: {:.3}] segment_length {:.3} new_p [{:.3} {:.3} {:.3}]", t, segment_length, new_p.x, new_p.y, new_p.z);
			t += t_delta;
		}

		segment_length
	}

	pub fn get_key_id(&self, t_in : f32) -> Option<usize> {
		let keys = self.0.keys();
		keys.iter().position(|&key| { (key.t - t_in).abs() <= f32::EPSILON })
	}

	pub fn get_key(&self, key_id : usize) -> &Key {
		let keys = self.0.keys();
		&keys[key_id]
	}

	pub fn get_key_id_from_pos(&self, pos : Vec3) -> Option<usize> {
		let keys = self.0.keys();
		let keys_cnt : usize = keys.len();
		let mut key_id = 0;
		let mut found = false;
		for i in 0 .. keys_cnt {
			let k = keys[i];
			if !k.value.cmpeq(pos).all() {
				continue;
			}

			key_id = i;
			found = true;
			break;
		}

		if found { Some(key_id) } else { None }
	}

	pub fn get_key_from_pos(&self, pos : Vec3) -> Option<&Key> {
		let option = self.get_key_id_from_pos(pos);
		if option.is_none() {
			return None;
		}

		let key_id = option.unwrap();
		let keys = self.0.keys();

		Some(&keys[key_id])
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
	POS(Vec3)
}

#[derive(Component)]
pub struct SplinePolyline;

#[derive(Component)]
pub struct ControlPointPolyline;

#[derive(Component, Clone, Copy)]
pub enum RoadWidth {
	W(f32)
}

#[derive(Component, Default)]
pub struct SplineControl {
	pub recalc_length : bool,
	pub new_point : bool,
}

pub struct BevySplinePlugin;

impl Plugin for BevySplinePlugin {
	fn build(&self, app: &mut App) {
        app
			.add_system(road_draw)
			.add_system(road_system)
			.add_system_to_stage(CoreStage::PostUpdate, on_tangent_moved)
			.add_system_to_stage(CoreStage::PostUpdate, on_control_point_moved)
 			;
	}
}

