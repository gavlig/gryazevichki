use bevy				:: prelude :: { * };
use bevy_rapier3d		:: prelude :: { * };
use bevy_mod_picking	:: { * };
use bevy_polyline		:: { prelude :: * };

use bevy::render::mesh::shape as render_shape;
use std::f32::consts	:: { * };

use super				:: { * };
use crate				:: { game };
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
		.insert			(Control::default())
		.insert			(TileState::default())
		;

	root_e
}

pub fn brick_road_iter(
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
		Orientation2D::Horizontal 	=> (x * lenz),
		Orientation2D::Vertical 	=> 0.0,
		}
	};

	let calc_offset_z = |z : f32, iter : f32, orientation : Orientation2D| -> f32 {
		match orientation {
		Orientation2D::Horizontal 	=> z * lenx,
		Orientation2D::Vertical 	=> 0.0,
		}
	};

	let offset_x 		= calc_offset_x(state.x as f32, iter0, state.orientation);
	let offset_z 		= calc_offset_z(state.z as f32, iter0, state.orientation);

	// now seams are tricky
	//
	//

	let calc_seam_offset_x = |x : f32, z : f32, iter : f32, orientation : Orientation2D, seam: f32| -> f32 {
		let mut offset_x = (x * seam * 2.0);

		offset_x
	};

	let calc_seam_offset_z = |z : f32, iter : f32, orientation : Orientation2D, seam: f32| -> f32 {
		let mut offset_z = (z * seam * 2.0);

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

		if t0 < t && t < t1 {
			pt0 = key0.value;
			pt1 = key1.value;

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
		let t			= if (t + lenx) >= config.limit_z { t - lenx * 2.0 } else { t + lenx };
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

	pose = base_spline_pose * pose;

	let base_spline_pose_inv = base_spline_pose.compute_matrix().inverse();
	let spline_p_local = base_spline_pose_inv.transform_point3(spline_p);

	pose.translation.x += spline_p_local.x;
	pose.translation.z += spline_p_local.z - pose.translation.z;
	//pose.translation.z += if !debug { 0.0 } else { spline_p.z - pose.translation.z };
	// pose.translation.z  = if debug { spline_p.z } else { pose.translation.z };
	
	// pose.rotation		= detail_spline_rotation * pose.rotation;

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

	let newoffx	= calc_offset_x		((state.x + 1) as f32, state.iter as f32, state.orientation) 
				+ calc_seam_offset_x((state.x + 1) as f32, state.z as f32, state.iter as f32, state.orientation, seam);

	let newoffz	= calc_offset_z		((state.z + 1) as f32, state.iter as f32, state.orientation)
				+ calc_seam_offset_z((state.z + 1) as f32, state.iter as f32, state.orientation, seam);

	let total_length = spline.total_length();

	if newoffx < config.width {
		state.x += 1;
	} else if newoffz < total_length {
		state.x  = 0;
		state.z += 1;
	} else {
		state.finished = true;
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