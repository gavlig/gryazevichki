use bevy				::	prelude :: { * };
use bevy_rapier3d		::	prelude :: { * };
use bevy_mod_picking	::	{ * };

use bevy::render::mesh::shape as render_shape;
use std::f32::consts	::	{ * };

use super				::	{ Herringbone :: * };

use crate :: Game 		as Game;

pub fn brick_road(
		transform		: Transform,
		config_in		: &Herringbone2Config,
	mut sargs			: &mut SpawnArguments,
) -> Entity {
	let mut config		= config_in.clone();
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

	let offset			= transform.translation;
	
	// spline requires at least 4 points: 2 control points(Key) and 2 tangents
	//
	//
	let road_len		= config.limit_z - config.offset_z;
	let tan_offset		= road_len / 4.0;
	config.init_tangent_offset = tan_offset;

	let t0				= config.offset_z;
	let t1				= config.limit_z;
	
	// limit_z and offset_z are used both for final tile coordinates and for final value of t to have road length tied to spline length and vice versa
	let key0_pos		= Vec3::new(offset.x, offset.y, config.offset_z);
	
	// StrokeBezier allows having two tangent points and we're going to use that
	let tangent00		= Vec3::new(offset.x, offset.y, -tan_offset);
	let tangent01		= Vec3::new(offset.x, offset.y, tan_offset);
	let tangent10		= Vec3::new(offset.x, offset.y, road_len - tan_offset);
	let tangent11		= Vec3::new(offset.x, offset.y, road_len + tan_offset);

	let key1_pos		= Vec3::new(offset.x, offset.y, config.limit_z);

	let key0			= SplineKey::new(t0, key0_pos, SplineInterpolation::StrokeBezier(tangent00, tangent01));
	let key1			= SplineKey::new(t1, key1_pos, SplineInterpolation::StrokeBezier(tangent10, tangent11));
	let spline			= Spline::from_vec(vec![key0, key1]);

	let root_e			= Game::spawn::root_handle(transform, &mut sargs);
	config.parent 		= root_e;

	let key0_e 			= Game::spawn::spline_control_point(0, &key0, root_e, true, &mut sargs);
	let key1_e 			= Game::spawn::spline_control_point(1, &key1, root_e, true, &mut sargs);

	sargs.commands.entity(root_e)
		.insert			(config)
		.insert			(spline)
		.insert			(Control::default())
		.insert			(TileState::default())
		.add_child		(key0_e)
		.add_child		(key1_e)
		;

	root_e
}

pub fn brick_road_iter(
	mut state			: &mut TileState,
	mut	config			: &mut Herringbone2Config,
		spline			: &Spline,
		_ass			: &Res<AssetServer>,
		commands		: &mut Commands
) {
	let init_rotation	= match state.orientation {
	Orientation2D::Horizontal 	=> Quat::from_rotation_y(FRAC_PI_2),
	Orientation2D::Vertical 	=> Quat::IDENTITY,
	};

	let seam			= config.seam;
	let length			= config.limit_z - config.offset_z;

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
	pose.translation.z	+= config.offset_z;

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

	// now let me interject for a moment with a spline (Hi Freya!)
	//
	//

	// spline is in the same local space as each brick is
	let t				= pose.translation.z;
	let spline_p		= match spline.sample(t) {
		// ok, sample was a success, get the point from it
		Some(p)			=> p,
		// sample wasnt a succes, try previuos point on spline
		None			=> {
		match state.prev_spline_p {
			Some(p)		=> p,
			None		=> Vec3::ZERO,
		}
		},
	};

	let spline_r		= match state.prev_spline_p {
		Some(prev_spline_p) => {
			let spline_dir	= (spline_p - prev_spline_p).normalize();
			Quat::from_rotation_arc(Vec3::Z, spline_dir)
		},
		// if there is no previous point we try to just move t forward or backward if possible and sample there
		None => {
			let t		= if t >= lenx { t - lenx } else { t + lenx };
			match spline.sample(t) {
				Some(prev_spline_p) => {
					let spline_dir = (spline_p - prev_spline_p).normalize();
					Quat::from_rotation_arc(Vec3::Z, spline_dir)
				},
				None	=> Quat::IDENTITY,
			}
		}
	};

	state.prev_spline_p = Some(spline_p);

	// applying offset by x sampled from spline
	pose.translation.x	+= spline_p.x;
	// spline is sampled by z so it doesnt bring any offset on z

	// applying rotation calculated from spline direction
	pose.rotation		*= spline_r;

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

		commands.entity(config.parent).with_children(|parent| {
			insert_tile_components!(parent.spawn_bundle(bundle));
		});
	}

	// check for end conditions
	//
	//

	let newoffx	= calc_offset_x		(state.x as f32, iter1, state.orientation) 
				+ calc_seam_offset_x(state.x as f32, state.z as f32, iter1, state.orientation, seam);

	let newoffz	= calc_offset_z		(state.z as f32, iter1, state.orientation)
				+ calc_seam_offset_z(state.z as f32, iter1, state.orientation, seam);

	if ((newoffx >= config.width) && (config.width != 0.0))
	|| ((newoffz >= length) && (length != 0.0))
	|| (state.iter >= config.limit_iter && config.limit_iter != 0)
	{
		let prev_orientation = state.orientation.clone();

		state.iter			= 0;
		state.orientation.flip();

		state.prev_spline_p = None;

		// println!		("Flipped orientation x_limit: {} z_limit: {} limit: {}", io.x_limit, io.z_limit, io.limit);

		if prev_orientation == Orientation2D::Vertical {
			let newoffx	= calc_offset_x		((state.x + 1) as f32, state.iter as f32, state.orientation) 
						+ calc_seam_offset_x((state.x + 1) as f32, state.z as f32, state.iter as f32, state.orientation, seam);

			let newoffz	= calc_offset_z		((state.z + 1) as f32, state.iter as f32, state.orientation)
						+ calc_seam_offset_z((state.z + 1) as f32, state.iter as f32, state.orientation, seam);

			if newoffx < config.width && !state.finished_hor {
				state.x		+= 1;
				// println!("x =+ 1 new offx {:.3}", newoffx);
			} else if newoffz < length {
				state.x		= 0;
				state.z		+= 1;
				state.finished_hor = true;
				// println!("x = 0, z += 1 new offz {:.3}", newoffz);
			} else {
				state.finished = true;
				// println!("herringbone_brick_road_iter finished!");
			}
		}
	}

	state.iter				+= 1;
}