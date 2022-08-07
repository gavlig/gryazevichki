use bevy				:: prelude :: { * };
use iyes_loopless		:: prelude :: { * };

use super::game 		:: { * };

pub mod spawn;
mod systems;
use systems				:: *;

pub type Raw 			= splines::Spline<f32, Vec3>;
pub type Interpolation 	= splines::Interpolation<f32, Vec3>;
pub type Key 			= splines::Key<f32, Vec3>;

pub enum ParameterType {
	SplineLength,
	Timeline
}

// wrapper for SplineRaw to have it as a Bevy Component
#[derive(Component)]
pub struct Spline {
	pub raw : Raw,
	pub clamp_x : Option<f32>,
	pub clamp_y : Option<f32>,
	pub clamp_z : Option<f32>,
	pub parameter_type : ParameterType
}

impl Default for Spline {
	fn default() -> Self {
		Self {
			raw : Raw::from_vec(Vec::<Key>::new()),
			clamp_x : None,
			clamp_y : None,
			clamp_z : None,
			parameter_type : ParameterType::SplineLength,
		}
	}
}

impl Spline {
	pub fn set_interpolation(&mut self, t : f32, interpolation : Interpolation) {
		let id = self.get_key_id(t).unwrap();
		*self.raw.get_mut(id).unwrap().interpolation = interpolation;
	}

	pub fn set_interpolation_from_id(&mut self, id : usize, interpolation : Interpolation) {
		*self.raw.get_mut(id).unwrap().interpolation = interpolation;
	}

	pub fn get_interpolation(&self, t : f32) -> &Interpolation {
		let id = self.get_key_id(t).unwrap();
		&self.raw.get(id).unwrap().interpolation
	}

	pub fn set_control_point(&mut self, t : f32, controlp_pos : Vec3) {
		let id = self.get_key_id(t).unwrap();
		self.raw.replace(id, |k : &Key| { Key::new(t, controlp_pos, k.interpolation) });
	}

	pub fn set_control_point_by_id(&mut self, id : usize, t : f32, controlp_pos : Vec3) {
		self.raw.replace(id, |k : &Key| { Key::new(t, controlp_pos, k.interpolation) });
	}

	pub fn set_control_point_t(&mut self, id : usize, t : f32) {
		self.raw.replace(id, |k : &Key| { Key::new(t, k.value, k.interpolation) });
	}

	pub fn set_control_point_pos(&mut self, id : usize, controlp_pos : Vec3) {
		*self.raw.get_mut(id).unwrap().value = controlp_pos;
	}

	pub fn total_length(&self) -> f32 {
		let keys = self.keys();
		let total_keys = keys.len();
		if total_keys < 2 {
			return 0.0; // return error instead?
		}

		keys.last().unwrap().t
	}

	// distance between two points in space is used as t parameter
	fn calculate_t_for_pos_spline_length(&self, pos : Vec3) -> f32 {
		let mut key_id = 1;
		let mut total_length = 0.0;
		let keys = self.keys();
		let total_keys = keys.len();

		let new_t =
		loop {
			let key0 = keys[key_id - 1];
			let key1 = keys[key_id];
			let pos0 = key0.value;
			let pos1 = key1.value;

			let segment_len_sq = pos0.distance_squared(pos1);
			let pos_in_segment_sq = pos0.distance_squared(pos);
			let epsilon = 0.05;
			if pos_in_segment_sq <= epsilon {
				break key0.t;
			}

			if (pos_in_segment_sq - segment_len_sq).abs() <= epsilon {
				break key1.t;
			}

			let new_pos_delta = pos - pos0;

			if pos_in_segment_sq < segment_len_sq {
				break total_length + new_pos_delta.length();
			}

			key_id += 1;
			if key_id == total_keys {
				break total_length + new_pos_delta.length();
			}

			total_length += keys[key_id].t;
		};

		new_t
	}

	// z coordinate is used as a 't' parameter
	fn calculate_t_for_pos_timeline(&self, pos : Vec3) -> f32 {
		let mut key_id = 1;
		let mut total_length = 0.0;
		let keys = self.keys();
		let total_keys = keys.len();

		let new_t =
		loop {
			let key0 = keys[key_id - 1];
			let key1 = keys[key_id];
			let pos0 = key0.value;
			let pos1 = key1.value;

			let segment_len = (pos0.z - pos1.z).abs();
			let pos_in_segment = (pos0.z - pos.z).abs();
			let epsilon = 0.05;
			if pos_in_segment <= epsilon {
				break key0.t;
			}

			if (pos_in_segment - segment_len).abs() <= epsilon {
				break key1.t;
			}

			let new_pos_delta = pos - pos0;

			if pos_in_segment < segment_len {
				break total_length + new_pos_delta.z.abs();
			}

			key_id += 1;
			if key_id == total_keys {
				break total_length + new_pos_delta.z.abs();
			}

			total_length += keys[key_id].t;
		};

		new_t
	}

	pub fn calculate_t_for_pos(&self, pos : Vec3) -> f32 {
		let keys = self.keys();
		let total_keys = keys.len();
		if total_keys < 2 {
			return pos.length();
		}

		let new_t = match self.parameter_type {
			ParameterType::SplineLength => self.calculate_t_for_pos_spline_length(pos),
			ParameterType::Timeline		=> self.calculate_t_for_pos_timeline(pos)
		};

		new_t
	}

	// calculate t of segment [key_id - 1, key_id]
	pub fn calculate_segment_t(&self, key_id : usize) -> f32 {
		if key_id < 1 {
			return 0.0;
		}
		
		let keys = self.keys();
		let key0 = keys[key_id - 1];
		let key1 = keys[key_id];

		let t0 = key0.t;
		let t1 = key1.t;
		let t_range = t1 - t0;

		let segment_t = match self.parameter_type {
		ParameterType::SplineLength => {
			let t_delta = t_range / 10.0; // 10 samples per meter

			let mut segment_length = 0.0;
			let mut prev_p = key0.value;
			let mut t = t0 + t_delta;
			// println!("calculate_segment_length t0 {:.3} t1 {:.3} t_delta {:.3}", t0, t1, t_delta);
			while t < (t1 + 0.00001) {
				let new_p = self.clamped_sample(t).unwrap();
				segment_length += (new_p - prev_p).length();
				prev_p = new_p;
				// println!("[t: {:.3}] tn: {} segment_length {:.3} new_p [{:.3} {:.3} {:.3}]", t, t + t_delta, segment_length, new_p.x, new_p.y, new_p.z);
				t += t_delta;
			}

			segment_length
		},
		ParameterType::Timeline => {
			key1.value.z - key0.value.z
		}
		};

		segment_t
	}

	pub fn calc_init_position(&self) -> Vec3 {
		let keys			= self.keys();
		if keys.len() <= 0 {
			assert!			(false, "calc_init_position was called on an empty spline! (keys.len() <= 0)");
			return 			Vec3::ZERO;
		}

		let t				= keys[0].t;
		self.calc_position	(t)
	}

	pub fn calc_init_rotation(&self) -> Quat {
		let keys			= self.keys();
		if keys.len() <= 0 {
			assert!			(false, "calc_init_rotation was called on an empty spline! (keys.len() <= 0)");
			return 			Quat::IDENTITY;
		}

		let t				= keys[0].t;
		self.calc_rotation	(t)
	}

	pub fn calc_position(&self, t : f32) -> Vec3 {
		let mut spline_p	= match self.clamped_sample(t) {
			Some(p)			=> p,
			None			=> panic!("calc_position: spline.clamped_sample failed! t: {:.3}", t),
		};

		let clamp = |val : f32, clamp_xxx : Option<f32>| -> f32 {
			if clamp_xxx.is_none() {
				return		val;
			}
			let clxxx		= clamp_xxx.unwrap();
			val.clamp(-clxxx, clxxx)
		};

		spline_p.x = clamp(spline_p.x, self.clamp_x);
		spline_p.y = clamp(spline_p.y, self.clamp_y);
		spline_p.z = clamp(spline_p.z, self.clamp_z);
		
		spline_p
	}

	pub fn calc_rotation(&self, t : f32) -> Quat {
		let spline_p		= self.calc_position(t);

		self.calc_rotation_wpos(t, spline_p)
	}

	pub fn calc_rotation_wpos(&self, t : f32, spline_p : Vec3) -> Quat {
		let total_length 	= self.total_length();
		let eps				= 0.00001;
		let (next_t, reverse) = 
		if t + eps < total_length {
			(t.max(0.0) + eps, false)
		} else {
			(total_length - eps, true)
		};

		let next_spline_p	= self.calc_position(next_t);

		let spline_dir		= (if !reverse { next_spline_p - spline_p } else { spline_p - next_spline_p }).normalize();
		Quat::from_rotation_arc(Vec3::Z, spline_dir)
	}

	pub fn get_key_id(&self, t_in : f32) -> Option<usize> {
		let keys = self.raw.keys();
		keys.iter().position(|&key| { (key.t - t_in).abs() <= f32::EPSILON })
	}

	pub fn get_key(&self, key_id : usize) -> &Key {
		let keys = self.raw.keys();
		&keys[key_id]
	}

	pub fn get_key_id_from_pos(&self, pos : Vec3) -> Option<usize> {
		let keys = self.raw.keys();
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
		let keys = self.raw.keys();

		Some(&keys[key_id])
	}

	pub fn clone_with_offset(&self, offset_in : Vec3) -> Spline {
		let mut keys_with_offset : Vec<Key> = Vec::new();
		for k in self.keys() {
			let spline_rotation = self.calc_rotation(k.t);
			let offset = spline_rotation.mul_vec3(offset_in);

			let new_interpolation = match k.interpolation {
				Interpolation::StrokeBezier(V0, V1) => {
					Interpolation::StrokeBezier(V0 + offset, V1 + offset)
				},
				_ => panic!("unsupported interpolation!"),
			};

			let mut new_k = k.clone();

			new_k.value += offset;
			new_k.interpolation = new_interpolation;

			keys_with_offset.push(new_k);
		}

		Spline::from_vec(keys_with_offset)
	}

	// TODO: "reset" spline to default values
	// pub fn fff(length : f32) {
	// 	let offset_y		= 0.5; // VERTICALITY
	
	// 	// spline requires at least 4 points: 2 control points(Key) and 2 tangents
	// 	//
	// 	//
	// 	let tan_offset		= length / 3.0;

	// 	// limit_z and offset_z are used both for final tile coordinates and for final value of t to have road length tied to spline length and vice versa
	// 	let key0_pos		= Vec3::new(0.0, offset_y, 0.0);
		
	// 	// StrokeBezier allows having two tangent points and we're going to use that
	// 	let tangent00		= Vec3::new(0.0, offset_y, 0.0 - tan_offset);
	// 	let tangent01		= Vec3::new(0.0, offset_y, 0.0 + tan_offset);

	// 	let tangent10		= Vec3::new(0.0, offset_y, length - tan_offset);
	// 	let tangent11		= Vec3::new(0.0, offset_y, length + tan_offset);

	// 	let key1_pos		= Vec3::new(0.0, offset_y, length);

	// 	let t0				= 0.0;
	// 	let t1				= (key1_pos - key0_pos).length();
	// }

	// wrapper
	pub fn from_vec(keys : Vec<Key>) -> Self {
		Self {
			raw : Raw::from_vec(keys),
			..default()
		}
	}

	// wrapper
	pub fn sample(&self, t : f32) -> Option<Vec3> {
		self.raw.sample(t)
	}

	// wrapper
	pub fn clamped_sample(&self, t : f32) -> Option<Vec3> {
		self.raw.clamped_sample(t)
	}

	// wrapper
	pub fn add(&mut self, key : Key) {
		self.raw.add(key);
	}

	// wrapper
	pub fn len(&self) -> usize {
		self.raw.len()
	}

	// wrapper
	pub fn keys(&self) -> &[Key] {
		self.raw.keys()
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
	pub recalc_t : bool,
	pub new_point : bool,
	pub reset : bool,
}

pub struct BevySplinePlugin;

impl Plugin for BevySplinePlugin {
	fn build(&self, app: &mut App) {
        app
			.add_system(road_draw)
			.add_system_to_stage(
				CoreStage::PostUpdate,
				on_tangent_moved
					.label("bevy_spline::on_tangent_moved")
			)
			.add_system_to_stage(
				CoreStage::PostUpdate,
				on_control_point_moved
					.after("bevy_spline::on_tangent_moved")
					.label("bevy_spline::on_control_point_moved")
			)
			.add_system_to_stage(
				CoreStage::PostUpdate,
				road_system
					.after("bevy_spline::on_control_point_moved")
					.label("bevy_spline::road_system")
			)
 			;
	}
}

