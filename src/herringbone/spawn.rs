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
		debug			: bool,
		_ass			: &Res<AssetServer>,
		sargs			: &mut SpawnArguments,

	mut debug_lines		: &mut ResMut<DebugLines>,
) {
	let init_rotation	= match state.orientation {
		Orientation2D::Horizontal 	=> Quat::from_rotation_y(FRAC_PI_2),
		Orientation2D::Vertical 	=> Quat::IDENTITY,
		};

	let total_length 	= spline.total_length();

	let seam			= config.seam * 2.0;

	let hlenz			= config.hsize.z;
	let lenz			= hlenz * 2.0;

	let hlenx			= config.hsize.x;
	let lenx			= hlenx * 2.0;

	let iter0			= (state.iter + 0) as f32;
	let iter1			= (state.iter + 1) as f32;

	// t == z in spherical vacuum

	let keys = spline.keys();
	let key_id = state.key;
	let key0 = keys[key_id + 0];
	let key1 = keys[key_id + 1]; // assume we never have invalid key_id

	let interpolation0 = key0.interpolation;
	let interpolation1 = key1.interpolation;

	// TODO: move this to method
	let tangent01 = match interpolation0 {
		splines::Interpolation::StrokeBezier(_V0, V1) => V1,
		_ => panic!("unsupported interpolation!"),
	};

	let tangent10 = match interpolation1 {
		splines::Interpolation::StrokeBezier(V0, _V1) => V0,
		_ => panic!("unsupported interpolation!"),
	};

	let tangent01_local = tangent01 - key0.value;

	let seglen0 = (tangent01 - key0.value).length();
	let seglen1 = (tangent10 - tangent01).length();
	let seglen2 = (key1.value - tangent10).length();

	let linear_length = (key1.value - key0.value).length();
	let spline_segment_length = spline.calculate_segment_length(key_id + 1);

	let ratio = linear_length / spline_segment_length;

	let iter_offset = lenz + seam;

	// let calc_offset_z = |iter : f32| -> f32 {
	// 	//0.2 * iter
	// 	hlenz + iter * iter_offset
		
	// };

	// let mut herrpos = Vec3::ZERO;
	let mut herrrot = Quat::IDENTITY;

	if state.iter % 2 == 0 {
		herrrot = Quat::from_rotation_y(FRAC_PI_4);
	} else {
		herrrot = Quat::from_rotation_y(-FRAC_PI_4);
	};

	{
		let mut herrpos = Vec3::ZERO;

		let offset = iter_offset * ratio;
		let p = Vec3::new(0.0, 1.5, state.t + offset);

		if state.iter % 2 == 0 {
			herrpos = p + Quat::from_rotation_y(-FRAC_PI_4).mul_vec3(Vec3::Z) * hlenx;
			debug_lines.line_colored(p, herrpos, 0.01, Color::rgb(0.8, 0.2, 0.2));
		} else {
			herrpos = p + Quat::from_rotation_y(-FRAC_PI_4).mul_vec3(-Vec3::Z) * hlenx;
			debug_lines.line_colored(p, herrpos, 0.01, Color::rgb(0.2, 0.2, 0.8));
		};
	}

	let mut calc_t = |state_t : f32, iter : u32, correction0 : f32, correction1 : f32| -> (f32, f32) {
		let offset = iter_offset * ratio * correction0 * correction1;
		let p = Vec3::new(0.0, 0.0, offset);
		let mut herrpos = p;
		if iter % 2 == 0 {
			herrpos = p + Quat::from_rotation_y(-FRAC_PI_4).mul_vec3(Vec3::Z) * hlenx;
			// herrpos = spline_p + Quat::from_rotation_y(-FRAC_PI_4).mul_vec3(Vec3::Z) * 1.0;
	
			// debug_lines.line_colored(sp, herrpos + Vec3::Y, 0.01, Color::rgb(0.8, 0.2, 0.8));
		} else {
			herrpos = p + Quat::from_rotation_y(-FRAC_PI_4).mul_vec3(-Vec3::Z) * hlenx;
			// herrpos = spline_p + Quat::from_rotation_y(-FRAC_PI_4).mul_vec3(-Vec3::Z) * 1.0;
			// herrpos = spline_p + -Vec3::Z;
	
			// debug_lines.line_colored(sp, herrpos + Vec3::Y, 0.01, Color::rgb(0.2, 0.8, 0.8));
		};

		(state_t + herrpos.z, herrpos.x)
	};

	let (t, _) = calc_t(state.t, state.iter, 1.0, 1.0);

	// let t = state.t + iter_offset * ratio;

	// println!("0 t: {:.3} offset {:.3} ratio: {:.3} linear: {:.3} spline: {:.3}", t, calc_offset_z(iter0), ratio, linear_length, spline_segment_length);

	if state.prev_spline_p.is_none() {
		state.prev_spline_p = spline.clamped_sample(key0.t);
	}
	let prev_spline_p 	= state.prev_spline_p.unwrap();

	let spline_p		= match spline.clamped_sample(t) {
	 	Some(p)			=> p,
	 	None			=> panic!("first spline.sample failed!"),
	};
	let spline_p_cache = spline_p.clone();

	let tile_dist = (spline_p - prev_spline_p).length();
	let correction0 = iter_offset / tile_dist;

	// let t = state.t + iter_offset * ratio * correction0;
	let (t, _) = calc_t(state.t, state.iter, correction0, 1.0);

	let spline_p		= match spline.clamped_sample(t) {
		Some(p)			=> p,
		None			=> panic!("first spline.sample failed!"),
   	};

	let tile_dist = (spline_p - prev_spline_p).length();
	let correction1 = iter_offset / tile_dist;

	// let t = state.t + (iter_offset * ratio * correction0) * correction1;
	let (t, herrx) = calc_t(state.t, state.iter, correction0, correction1);

	if t > total_length {
		state.finished = true;
		return;
	}

	// let ratio = (tangent01.length_squared() - (tangent01.x * tangent01.x)).sqrt();

	// let t_cache = t;
	// let t_delta = t - state.t;
	// let t = f32::min(state.t + (t_delta * ratio), total_length - 0.001);
	state.t = t;

	// println!("0 t: {:.3} t_cache : {:.3} spline_p: {:.3} {:.3} {:.3} tile_dist: {:.3} iter_offset: {:.3} ratio {:.3}", t, t_cache, spline_p.x, spline_p.y, spline_p.z, tile_dist, iter_offset, ratio);

	let spline_p		= match spline.sample(t) {
		Some(p)			=> p,
		None			=> panic!("main spline.sample failed! t: {} keys: {:?}", t, keys),
	};
	state.prev_spline_p	= Some(spline_p);

	let next_t = 
	if t + 0.01 < total_length {
		t + 0.01
	} else {
		t - 0.01
	};
	let next_spline_p	= match spline.clamped_sample(next_t) {
		Some(p)			=> p,
		None			=> panic!("secondary spline.clamped_sample failed!"),
	};

	let tile_dist = (spline_p - prev_spline_p).length();

	// println!("1 spline_p: {:.3} {:.3} {:.3} tile_dist: {:.3} iter_offset: {:.3}", spline_p.x, spline_p.y, spline_p.z, tile_dist, iter_offset);

	// pick next position, see how much space left between current and last tile, if more than seem, then either repick spline or trigonometry!

	let detail_spline_rotation = {
			let spline_dir	= (next_spline_p - spline_p).normalize();
			Quat::from_rotation_arc(Vec3::Z, spline_dir)
	};

	let mut pose 		= Transform::identity();

	// tile offset/rotation
	pose.translation.x 	= spline_p.x;// + herrx;
	pose.translation.z 	= spline_p.z;
	pose.rotation		*= init_rotation * herrrot * detail_spline_rotation; // 

	// spawning
	//
	//

	// spawn first brick with a strong reference to keep reference count > 0 and mesh/material from dying when out of scope
	let (mut me, mut ma) = (config.mesh.clone_weak(), config.material.clone_weak());
	if state.iter == 0 {
		(me, ma) = (config.mesh.clone(), config.material.clone());
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

	let (newoffz, _) = calc_t(state.t, state.iter, 1.0, 1.0);

	if newoffz >= total_length {
		state.finished = true;
	}

	// if state.iter == 100 {
	// 	state.finished = true;
	// }
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