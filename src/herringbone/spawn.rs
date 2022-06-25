use bevy				:: prelude :: { * };
use bevy_rapier3d		:: prelude :: { * };
use bevy_mod_picking	:: { * };
use bevy_polyline		:: { prelude :: * };
use bevy_prototype_debug_lines :: { DebugLines };

use bevy::render::mesh::shape as render_shape;
use std::f32::consts	:: { * };

use super				:: { * };
use crate				:: { bevy_spline };

pub fn brick_road(
	transform			: &Transform,
	config_in			: &Herringbone2Config,
	debug				: bool,

	polylines			: &mut ResMut<Assets<Polyline>>,
	polyline_materials 	: &mut ResMut<Assets<PolylineMaterial>>,

	mut sargs			: &mut SpawnArguments,
) -> Entity {
	let mut config		= config_in.clone();

	let root_e 			= bevy_spline::spawn::new(
		transform,
		config.limit_z,
		120.0,
		Color::rgb(0.2, 0.2, 0.2),
		polylines,
		polyline_materials,
		sargs
	);
	config.root_entity	= root_e;

	let tile_size		= config.hsize * 2.0;

	config.mesh =
	sargs.meshes.add(
	Mesh::from(
		render_shape::Box::new(
			tile_size.x, tile_size.y, tile_size.z
			//0.06, 0.06, 0.08
		)
	));

	config.material	=
	sargs.materials.add(
	StandardMaterial { 
		base_color : Color::ALICE_BLUE,
		..default()
	});

	sargs.commands.entity(root_e)
		.insert			(config)
		.insert			(HerringboneControl::default())
		.insert			(TileState::default())
		;

	root_e
}

pub fn brick_road_iter(
	mut state			: &mut TileState,
	mut	config			: &mut Herringbone2Config,
		spline			: &Spline,
		transform		: &GlobalTransform,
		_ass			: &Res<AssetServer>,
		sargs			: &mut SpawnArguments,
		debug			: u32,
		visual_debug	: bool,
		verbose			: bool,
		dry_run			: bool,

	mut debug_lines		: &mut ResMut<DebugLines>,
) {
	let total_length 	= spline.total_length();

	if verbose {
		println!("[{}] total_length: {:.3}", state.iter, total_length);
	}

	let seam			= config.seam * 2.0;

	let hlenz			= config.hsize.z;
	let lenz			= hlenz * 2.0;

	let hlenx			= config.hsize.x;
	let lenx			= hlenx * 2.0;

	let iter0			= (state.iter + 0) as f32;
	let iter1			= (state.iter + 1) as f32;

	// t == z in spherical vacuum

	let dbg_iter = if debug != 1 { 4 } else { 100000000 };

	let keys = spline.keys();
	let key_id = state.key;
	let key0 = keys[key_id + 0];
	let key1 = keys[key_id + 1]; // assume we never have invalid key_id

	let linear_length = (key1.value - key0.value).length();
	let spline_segment_length = spline.calculate_segment_length(key_id + 1);

	let linear2spline_ratio = linear_length / spline_segment_length;

	let herrrot = 
	if state.iter % 2 == 0 {
		Quat::from_rotation_y(FRAC_PI_4)
	} else {
		Quat::from_rotation_y(-FRAC_PI_4)
	};

	let calc_next_pos = |prev_p : Vec3, iter : u32| -> Vec3 {
		let ver = Vec3::Y * 0.5;

		if iter == 0 {
			return ver;
		}

		let herrpos = prev_p +
		if iter % 2 == 0 {
			let offset0_scalar = hlenz - hlenx;
			let offset0 = Quat::from_rotation_y(-FRAC_PI_4).mul_vec3(Vec3::Z * offset0_scalar);

			let offset1_scalar = hlenx + seam + hlenz;
			let offset1 = Quat::from_rotation_y(FRAC_PI_4).mul_vec3(Vec3::Z * offset1_scalar);

			if verbose {
				println!("[{}] 0 calc_next_pos: [{:.3} {:.3}] prev: [{:.3} {:.3}]", iter, (prev_p + offset0 + offset1).x, (prev_p + offset0 + offset1).z, prev_p.x, prev_p.z);
			}

			offset0 + offset1
		} else {
			let offset0_scalar = hlenz - hlenx;
			let offset0 = Quat::from_rotation_y(FRAC_PI_4).mul_vec3(Vec3::Z * offset0_scalar);

			let offset1_scalar = hlenx + seam + hlenz;
			let offset1 = Quat::from_rotation_y(-FRAC_PI_4).mul_vec3(Vec3::Z * offset1_scalar);

			if verbose {
				println!("[{}] 1 calc_next_pos: [{:.3} {:.3}] prev: [{:.3} {:.3}]", iter, (prev_p + offset0 + offset1).x, (prev_p + offset0 + offset1).z, prev_p.x, prev_p.z);
			}

			offset0 + offset1
		};

		herrpos
	};

	let calc_offset_from_spline = |iter : u32, spline_rotation_ref : &Quat, spline_offset_scalar : f32| {
		let angle =
		if iter % 2 == 0 {
			-FRAC_PI_2
		} else {
			FRAC_PI_2
		};

		let init_rot = Quat::from_rotation_y(angle);
		let offset = (*spline_rotation_ref * init_rot).mul_vec3(Vec3::Z * spline_offset_scalar);
		offset
	};

	let calc_spline_rotation = |t : f32, spline_p : Vec3| -> Quat {
		let next_t = 
		if t + 0.01 < total_length {
			t.max(0.0) + 0.01
		} else {
			total_length - 0.01
		};
		let next_spline_p	= match spline.clamped_sample(next_t) {
			Some(p)			=> p,
			None			=> panic!("secondary spline.clamped_sample failed!"),
		};

		let spline_dir		= (next_spline_p - spline_p).normalize();
		Quat::from_rotation_arc(Vec3::Z, spline_dir)
	};

	// Sample with given t and adjust it until distance between tiles is close to target
	let mut calc_t_on_spline = |iter : u32, state_t : f32, tile_pos_delta : Vec3, prev_p : Vec3, tile_dist_target : f32| -> (f32, Vec3) {
		let ver 		= Vec3::Y * 1.0;

		let spline_offset_scalar = if state.iter % 2 != 0 { tile_pos_delta.x } else { 0.0 };
		let init_offset = tile_pos_delta.z;

		let mut spline_p = Vec3::ZERO;
		let mut t 		= state_t + init_offset;
		let mut i 		= 0;
		let mut corrections : Vec<f32> = Vec::new();

		if tile_dist_target <= 0.0 {
			return 		(t, spline_p);
		}

		loop {
			spline_p = match spline.clamped_sample(t) {
				Some(p)	=> p,
				None	=> panic!("calc_t_corrected spline.clamped_sample failed!"),
			};

			let q = calc_spline_rotation(t, spline_p);
			let spline_offset = calc_offset_from_spline(iter, &q, spline_offset_scalar);

			if visual_debug {
				debug_lines.line_colored(spline_p + ver, spline_p + spline_offset + ver, 0.01, Color::rgb(0.8, 0.2, 0.8));
			}

			let new_p = spline_p + spline_offset;

			let tile_dist_actual = (new_p - prev_p).length();

			let correction = tile_dist_target / tile_dist_actual;
			corrections.push(correction);
			let mut corrected_offset = init_offset;
			for c in corrections.iter() {
				corrected_offset *= c;
			}
			t = state_t + corrected_offset;

			if verbose {
				println!("[{} {}] tile_dist_actual : {:.3} = new_p({:.3} {:.3} {:.3}) - prev_p({:.3} {:.3} {:.3}) (spline_p: ({:.3} {:.3} {:.3}) spline_offset_scalar: {:.3})", state.iter, i, tile_dist_actual, new_p.x, new_p.y, new_p.z, prev_p.x, prev_p.y, prev_p.z, spline_p.x, spline_p.y, spline_p.z, spline_offset_scalar);
				println!("[{} {}] t : {:.3} correction : tile_dist_target({:.3}) / tile_dist_actual({:.3}) = correction({:.3})", state.iter, i, t, tile_dist_target, tile_dist_actual, correction);
			}

			i += 1;

			if (1.0 - correction).abs() <= 0.01  || i >= 5 {
				break;
			}
		}

		if t.is_infinite() || t.is_nan() {
			panic!("calc_t_on_spline: t is invalid!");
		}

		(t, spline_p)
	};

	let calc_width_offset = |neg : bool, iter : f32, spline_rotation : Quat| -> (f32, Vec3) {
		let single_offset = lenx * 1.5 + seam;
		let offset_x	= iter * single_offset;
		let mut www		= Vec3::new(offset_x, 0.0, 0.0);
		if neg 			{ www.x = -www.x; }
		www				= spline_rotation.mul_vec3(www);
		(offset_x, www)
	};

	if verbose {
		println!("[{}] getting next pos on straight line (along +z with little +-x) prev pos: [{:.3} {:.3}]", state.iter, state.pos.x, state.pos.z);
	}

	let next_pos 			= calc_next_pos(state.pos, state.iter);
	let tile_pos_delta 		= next_pos - state.pos;
	let t 					= state.t + tile_pos_delta.z;
	let tile_dist_target 	= tile_pos_delta.length();

	if t >= total_length {
		if verbose {
			println!("[{}] total_length limit reached! Next tile pos: [{:.3} {:.3}(<-)] total spline length: {:.3}", state.iter, next_pos.x, next_pos.z, total_length);
		}

		let (scalar, _) = calc_width_offset(false, (state.iter_width + 1) as f32, Quat::IDENTITY);

		if scalar * 2.0 < config.width {
			state.t = 0.0;
			state.iter = 0;
			state.pos = Vec3::Y * 0.5; // VERTICALITY
			state.iter_width += 1;

			if verbose {
				println!("[{}] width limit not reached({:.3}/{:.3}), inc iter_width({} -> {})", state.iter, scalar * 2.0, config.width, state.iter_width - 1, state.iter_width);
			}
		} else {
			state.finished = true;

			if verbose {
				println!("[{}] width limit reached! finished!", state.iter);
			}
		}
		
		return;
	}

	if verbose {
		println!("[{}] t: {:.3} next_pos:[{:.3} {:.3}] prev_pos: [{:.3} {:.3}] tile_pos_delta: [{:.3} {:.3}]", state.iter, t, next_pos.x, next_pos.z, state.pos.x, state.pos.z, tile_pos_delta.x, tile_pos_delta.z);
	}

	let mut prev_p = state.pos;
	prev_p.y = 0.5; // VERTICALITY

	let (t, spline_p)	= calc_t_on_spline(state.iter, state.t, tile_pos_delta, prev_p, tile_dist_target);

	if t > total_length {
		state.finished = true;
		return;
	}

	if verbose {
		println!("[{}] final spline_p [{:.3} {:.3}] for t : {:.3}", state.iter, spline_p.x, spline_p.z, t);
	}

	let spline_rotation = calc_spline_rotation(t, spline_p);

	let mut pose 		= Transform::identity();
 
	// tile offset/rotation
	//
	//

	pose.translation.x 	= spline_p.x;
	pose.translation.z 	= spline_p.z;

	// pattern is built around spline as a center position and every odd number we have an offset
	if state.iter % 2 != 0 {
		pose.translation += calc_offset_from_spline(state.iter, &spline_rotation, tile_pos_delta.x);
	}

	// rows of tiles
	let (_, width_offset) = calc_width_offset(false, state.iter_width as f32, spline_rotation);
	pose.translation 	+= width_offset;

	pose.rotation		*= herrrot * spline_rotation; // 

	if verbose {
		println!("[{}] final pose: [{:.3} {:.3}] tile_pos_delta.x: {:.3}", state.iter, pose.translation.x, pose.translation.z, tile_pos_delta.x);
	}

	if state.iter > 0 && visual_debug {
		let ver = Vec3::Y * 1.5;
		let iter = state.iter;
		let prev_p = state.pos;

		if iter % 2 == 0 {
			let offset0_scalar = hlenz - hlenx;
			let init_rot = Quat::from_rotation_y(-FRAC_PI_4);
			let offset0 = (spline_rotation * init_rot).mul_vec3(Vec3::Z * offset0_scalar);
	
			debug_lines.line_colored(prev_p + ver, prev_p + offset0 + ver, 0.01, Color::rgb(0.8, 0.2, 0.2));
	
			let offset1_scalar = hlenx + seam + hlenz;
			let init_rot = Quat::from_rotation_y(FRAC_PI_4);
			let offset1 = (spline_rotation * init_rot).mul_vec3(Vec3::Z * offset1_scalar);

			debug_lines.line_colored(prev_p + offset0 + ver, prev_p + offset0 + offset1 + ver, 0.01, Color::rgb(0.8, 0.2, 0.2));
			if verbose {
				println!("[{}] 0 dbg next_pos: [{:.3} {:.3}]", iter, (prev_p + offset0 + offset1).x, (prev_p + offset0 + offset1).z);
			}
	
			offset0 + offset1
		} else {
			let offset0_scalar = hlenz - hlenx;
			let init_rot = Quat::from_rotation_y(FRAC_PI_4);
			let offset0 = (spline_rotation * init_rot).mul_vec3(Vec3::Z * offset0_scalar);
	
			debug_lines.line_colored(prev_p + ver, prev_p + offset0 + ver, 0.01, Color::rgb(0.2, 0.2, 0.8));
	
			let offset1_scalar = hlenx + seam + hlenz;
			let init_rot = Quat::from_rotation_y(-FRAC_PI_4);
			let offset1 = (spline_rotation * init_rot).mul_vec3(Vec3::Z * offset1_scalar);
	
			debug_lines.line_colored(prev_p + offset0 + ver, prev_p + offset0 + offset1 + ver, 0.01, Color::rgb(0.2, 0.2, 0.8));
			if verbose {	
				println!("[{}] 1 dbg next_pos: [{:.3} {:.3}]", iter, (prev_p + offset0 + offset1).x, (prev_p + offset0 + offset1).z);
			}
	
			offset0 + offset1
		};
	}

	// spawning
	//
	//

	if !dry_run {
		// spawn first brick with a strong reference to keep reference count > 0 and keep mesh/material from dying when out of scope
		let (mut me, mut ma) = (config.mesh.clone_weak(), config.material.clone_weak());
		if state.iter == 0 {
			(me, ma) = (config.mesh.clone(), config.material.clone());
		}

		// this can be done without macro, but i need it for a reference
		macro_rules! insert_tile_components {
			($a:expr) => {
				$a	
				.insert			(config.body_type)
				.insert			(pose)
				.insert			(GlobalTransform::default())
				.insert			(Collider::cuboid(config.hsize.x, config.hsize.y, config.hsize.z))
//				.insert			(Friction{ coefficient : friction, combine_rule : CoefficientCombineRule::Average });
				.insert_bundle	(PickableBundle::default())
//				.insert			(Draggable::default())
				.insert			(Herringbone2)
				.insert			(Tile)
				.insert			(state.clone())
			}
		}

		let bundle = PbrBundle{ mesh: me, material: ma, ..default() };

		sargs.commands.entity(config.root_entity).with_children(|parent| {
			insert_tile_components!(parent.spawn_bundle(bundle));
		});
	}

	state.iter	+= 1;
	state.t		 = t;
	state.pos	 = pose.translation - width_offset;

	if verbose {
		println!("----------------------------");
	}
}

pub fn brick_road_iter2(
	mut state			: &mut TileState,
	mut	config			: &mut Herringbone2Config,
		spline			: &Spline,
		debug			: bool,
		_ass			: &Res<AssetServer>,
		sargs			: &mut SpawnArguments,
) {
	let init_rotation	= match state.orientation {
	Orientation2D::Horizontal 	=> Quat::from_rotation_y(FRAC_PI_2),
	Orientation2D::Vertical 	=> Quat::IDENTITY,
	};

	let seam			= config.seam;

	let hlenz			= config.hsize.z;
	let lenz			= hlenz * 2.0;

	let hlenx			= config.hsize.x;
	let lenx			= hlenx * 2.0;

	// main tile center calculation without seams
	//
	//

	let iter0			= (state.iter + 0) as f32;
	let iter1			= (state.iter + 1) as f32;

	let calc_offset_x = |x : f32, iter : f32, orientation : Orientation2D| -> f32 {
		match orientation {
		Orientation2D::Horizontal 	=> (iter + 1.0) * hlenz 				+ (x * (lenz * 2.0)),
		Orientation2D::Vertical 	=> (iter + 0.0) * hlenz + (hlenx * 1.0)	+ (x * (lenz * 2.0)),
		}
	};

	let calc_offset_z = |z : f32, iter : f32, orientation : Orientation2D| -> f32 {
		match orientation {
		Orientation2D::Horizontal 	=> (iter + 0.0) * hlenz + (hlenx * 1.0)	+ (z * (lenz * 2.0)),
		Orientation2D::Vertical 	=> (iter + 0.0) * hlenz + (hlenz * 2.0) + (z * (lenz * 2.0)),
		}
	};

	let offset_x 		= calc_offset_x(state.x as f32, iter0, state.orientation);
	let offset_z 		= calc_offset_z(state.z as f32, iter0, state.orientation);

	// now seams are tricky
	//
	//

	let calc_seam_offset_x = |x : f32, z : f32, iter : f32, orientation : Orientation2D, seam: f32| -> f32 {
		let mut offset_x = (iter * seam) + (x * seam * 3.0);

		if Orientation2D::Horizontal == orientation && z > 0.0 {
			offset_x 	+= seam * 0.5;
		}

		offset_x
	};

	let calc_seam_offset_z = |z : f32, iter : f32, orientation : Orientation2D, seam: f32| -> f32 {
		let mut offset_z = (iter * seam) + (z * seam * 3.0);

		if Orientation2D::Vertical == orientation {
			offset_z 	+= seam * 1.5;
		}
		offset_z 		+= z * seam * 0.5;

		offset_z
	};

	let seam_offset_x 	= calc_seam_offset_x(state.x as f32, state.z as f32, iter0, state.orientation, seam);
	let seam_offset_z 	= calc_seam_offset_z(state.z as f32, iter0, state.orientation, seam);

	// Start constructing tile pose
	//
	//

	let mut pose 		= Transform::identity();

	// half width shift to appear in the middle
	pose.translation.x	-= config.width / 2.0;

	// external offset for convenience
	//pose.translation.z	+= config.limit_mz;

	// tile offset/rotation
	pose.translation.x	+= offset_x + seam_offset_x;
	pose.translation.z	+= offset_z + seam_offset_z;
	pose.rotation		*= init_rotation;

	// if only io.limit is given set limits in cordinates anyway because otherwise we don't know where to stop not on diagonal
	if state.iter == config.limit_iter {
		if config.width == 0.0 {
			config.width = pose.translation.x;
		}

		if config.limit_z == 0.0 {
			config.limit_z = pose.translation.z;
		}
	}

	// up until now it's pure straight line of tiles

	// now let me interject for a moment with a spline (Hi Freya!)
	//
	//

	// t as in 'time' we sample on spline. equals to how much we moved from start to finish of the road
	// spline is in the same space as each brick is
	let t				= pose.translation.z; //Vec3::new(pose.translation.x, 0.0, pose.translation.z).length();

	// find pair of points (control points or tangents) in spline that are in front and behind our current t value
	let mut pt0			= Vec3::ZERO;
	let mut pt1			= Vec3::ZERO;
	let mut key0		= spline.keys()[0].clone();
	for key1 in spline.keys() {
		let t0 			= key0.t;
		let t1 			= key1.t;
		// println!("t0: {:.3} t: {:.3} t1: {:.3}", t0, t, t1);

		if t0 < t && t < t1 {

			pt0 = key0.value;
			pt1 = key1.value;

			// println!("took pt0: {:.3} {:.3} pt1: {:.3} {:.3}", pt0.x, pt0.z, pt1.x, pt1.z);

			break;
		}

		key0			= key1.clone();
	}

	// make a transform matrix from them
	let base_spline_dir = (pt1 - pt0).normalize();
	let base_spline_rotation = Quat::from_rotation_arc(Vec3::Z, base_spline_dir);
	let base_spline_pose = Transform { translation : pt0, rotation : base_spline_rotation, ..default() };

	

	// println!("[{:.3}[{} {}] {:?}] t: {:.3}, ox: {:.3} oz: {:.3}",
	// 	state.iter,
	// 	state.x,
	// 	state.z,
	// 	state.orientation,
	// 	t,
	// 	pose.translation.x,
	// 	pose.translation.z,
	// );
	

	// if there is no previous point we try to just move t forward or backward if possible and sample there
	if state.prev_spline_p.is_none() {
		//let t			= if (t + lenx) >= config.limit_z { t - lenx * 2.0 } else { t + lenx };
		state.prev_spline_p = spline.clamped_sample(t);
	}

	let spline_p		= match spline.sample(t) {
		// ok, sample was a success, get the point from it
		Some(p)			=> p,
		// sample wasnt a succes, try previous point on spline
		None			=> {
		match state.prev_spline_p {
			Some(p)		=> p,
			None		=> Vec3::ZERO,
		}
		},
	};
	// spline_pose.translation = spline_p;

	let detail_spline_rotation	= match state.prev_spline_p {
		Some(prev_spline_p) => {
			let spline_dir	= (spline_p - prev_spline_p).normalize();
			Quat::from_rotation_arc(Vec3::Z, spline_dir)
		},
		None => Quat::IDENTITY,
	};

	pose.translation.x += spline_p.x;// - pose.translation.x;
	pose.translation.z  = spline_p.z;
	pose.rotation		= detail_spline_rotation * pose.rotation;

	// // println!("rotation diff {:?} base_spline_rotation: {:?}, detail_spline_rotation: {:?}", (detail_spline_rotation - base_spline_rotation), base_spline_rotation, detail_spline_rotation);
	// let detail_spline_rotation = (base_spline_rotation.conjugate() * detail_spline_rotation);

	// //let base_spline_pose = Transform { translation : spline_p, rotation : detail_spline_rotation, ..default() };
	// let detail_spline_pose = Transform { rotation : detail_spline_rotation, ..default() };

	// pose = base_spline_pose * detail_spline_pose * pose;

	// spline_pose.rotation = spline_r;

	// state.prev_spline_p = Some(spline_p);

	// applying rotation calculated from spline direction
	//pose.rotation		*= spline_r;

	// let cache_x = pose.translation.x;
	// let cache_z = pose.translation.z;

	// let mut coef = 1.0;
	if debug {
		// let x = spline_p.x;
		// let z = pose.translation.z;
		// let s = (x * x + z * z).sqrt();
		// coef = t / s;

		// pose.translation.z *= coef;
		// let Z = pose.translation.z;
		// // println!("[{:.3}] x: {:.3} z: {:.3} s: {:.3} t: {:.3} Z: {:.3}", state.iter, state.x, state.z, s, t, Z);

		// pose.translation.x += spline.clamped_sample(Z).unwrap().x;

		//pose.translation.x += spline_p.x;
		//pose.translation.z = spline_p.z;
	} else {
		// applying offset by x sampled from spline
		//pose.translation.x += spline_p.x;
	}

	// pose = pose * spline_pose;

	// println!("[{:.3}[{} {}] {:?}] t: {:.3}, ox: {:.3} oz: {:.3} cx: {:.3} cz: {:.3}",
	// 	state.iter,
	// 	state.x,
	// 	state.z,
	// 	state.orientation,
	// 	t,
	// 	pose.translation.x,
	// 	pose.translation.z,
	// 	cache_x,
	// 	cache_z,
	// 	// spline_p.x,
	// 	// spline_p.z
	// );

	// spawning
	//
	//

	// spawn first brick with a strong reference to keep reference count > 0 and mesh/material from dying when out of scope
	let (mut me, mut ma) = (config.mesh.clone_weak(), config.material.clone_weak());
	match (state.x, state.z) {
		(0, 0) => {
			(me, ma)	= (config.mesh.clone(), config.material.clone());
		}
		_ => (),
	}

	{
		// this can be done without macro, but i need it for a reference
		macro_rules! insert_tile_components {
			($a:expr) => {
				$a	
				.insert			(config.body_type)
				.insert			(pose)
				.insert			(GlobalTransform::default())
				.insert			(Collider::cuboid(config.hsize.x, config.hsize.y, config.hsize.z))
//				.insert			(Friction{ coefficient : friction, combine_rule : CoefficientCombineRule::Average });
				.insert_bundle	(PickableBundle::default())
//				.insert			(Draggable::default())
				.insert			(Herringbone2)
				.insert			(Tile)
				.insert			(state.clone())
			}
		}

		let bundle = PbrBundle{ mesh: me, material: ma, ..default() };

		sargs.commands.entity(config.root_entity).with_children(|parent| {
			insert_tile_components!(parent.spawn_bundle(bundle));
		});
	}

	state.iter			+= 1;

	// check for end conditions
	//
	//

	let newoffx	= calc_offset_x		(state.x as f32, iter1, state.orientation) 
				+ calc_seam_offset_x(state.x as f32, state.z as f32, iter1, state.orientation, seam);

	let newoffz	= calc_offset_z		(state.z as f32, iter1, state.orientation)
				+ calc_seam_offset_z(state.z as f32, iter1, state.orientation, seam)
				+ config.limit_mz;

	let limit_length = spline.total_length();

	if ((newoffx >= config.width) && (config.width != 0.0))
	|| (newoffz >= limit_length)
	|| (state.iter >= config.limit_iter && config.limit_iter != 0)
	{
		let prev_orientation = state.orientation.clone();

		state.iter			= 0;
		state.orientation.flip();

		println!("FLIP! width: {:.3} limit_length: {:.3} newoffx: {:.3} newoffz: {:.3}",
			config.width,
			limit_length,
			newoffx,
			newoffz
		);

		state.prev_spline_p = None;

		if prev_orientation == Orientation2D::Vertical {
			let newoffx	= calc_offset_x		((state.x + 1) as f32, state.iter as f32, state.orientation) 
						+ calc_seam_offset_x((state.x + 1) as f32, state.z as f32, state.iter as f32, state.orientation, seam);

			let newoffz	= calc_offset_z		((state.z + 1) as f32, state.iter as f32, state.orientation)
						+ calc_seam_offset_z((state.z + 1) as f32, state.iter as f32, state.orientation, seam)
						+ config.limit_mz;

			if newoffx < config.width && !state.finished_hor {
				state.x	+= 1;
				println!("x += 1");
			} else if newoffz < limit_length {
				state.x	= 0;
				state.z	+= 1;
				state.finished_hor = true;
				println!("z += 1, x = 0");
			} else {
				state.finished = true;
				println!("finished!");
			}
		}
	}
}