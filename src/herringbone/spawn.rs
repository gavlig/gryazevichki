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
		.insert			(BrickRoadProgressState::default())
		;

	root_e
}

pub fn brick_road_iter(
	mut state			: &mut BrickRoadProgressState,
	mut	config			: &mut Herringbone2Config,
		// spline : &Spline,
		spline_in		: &Spline,
		tiles_row_prev	: &mut Vec<TileRowIterState>,
		tiles_row_cur	: &mut Vec<TileRowIterState>,
		transform		: &GlobalTransform,
		_ass			: &Res<AssetServer>,
		sargs			: &mut SpawnArguments,
		control			: &HerringboneControl,

	mut debug_lines		: &mut ResMut<DebugLines>,
) {
	let hseam			= config.seam;
	let seam			= hseam * 2.0; // 0.01 * 2.0

	let hlenz			= config.hsize.z;
	let lenz			= hlenz * 2.0;

	let hlenx			= config.hsize.x;
	let lenx			= hlenx * 2.0;

	let iter0			= (state.iter + 0) as f32;
	let iter1			= (state.iter + 1) as f32;

	let herrrot = 
	if state.iter % 2 == 0 {
		Quat::from_rotation_y(FRAC_PI_4)
	} else {
		Quat::from_rotation_y(-FRAC_PI_4)
	};

	let calc_width_offset = |positive : bool, iter_width : f32| -> (f32, Vec3) {
		let single_offset = ((hlenz + seam).powf(2.0) + (hlenx + seam + hlenx).powf(2.0)).sqrt();
		// let single_offset = config.width / 8.0;
		let width		= iter_width * single_offset;
		let mut offset_x = Vec3::new(width, 0.0, 0.0);
		if !positive	{ offset_x.x = -offset_x.x; }
		(width, offset_x)
	};

	let calc_width_offset_wrotation = |positive : bool, iter_width : f32, spline_rotation : Quat| -> (f32, Vec3) {
		let (width, mut offset_x) = calc_width_offset(positive, iter_width);
		offset_x				= spline_rotation.mul_vec3(offset_x);
		(width, offset_x)
	};

	let (row_offset, row_offset_x) = calc_width_offset(true, state.iter_width as f32);
	let spline			= spline_in.clone_with_offset(row_offset_x);
	let total_length 	= spline.total_length();

	if control.verbose {
		println!("[{} {}] t: {:.3} spline.total_length: {:.3} iter_width: {} row_offset(width): {:.3}", state.iter_width, state.iter, state.t, total_length, state.iter_width, row_offset);
	}

	// t == z in spherical vacuum

	let calc_next_pos = |prev_p : Vec3, iter : usize| -> Vec3 {
		let ver = Vec3::Y * 0.5; // VERTICALITY

		if iter == 0 {
			return ver;
		}

		let herrpos = prev_p +
		if iter % 2 == 0 {
			let offset0_scalar = hlenz - hlenx;
			let offset0 = Quat::from_rotation_y(-FRAC_PI_4).mul_vec3(Vec3::Z * offset0_scalar);

			let offset1_scalar = hlenx + seam + hlenz;
			let offset1 = Quat::from_rotation_y(FRAC_PI_4).mul_vec3(Vec3::Z * offset1_scalar);

			if control.verbose {
				let offset = prev_p + offset0 + offset1;
				println!("[{}] (even) calc_next_pos: [{:.3} {:.3} {:.3}]", iter, offset.x, offset.y, offset.z);
			}

			offset0 + offset1
		} else {
			let offset0_scalar = hlenz - hlenx;
			let offset0 = Quat::from_rotation_y(FRAC_PI_4).mul_vec3(Vec3::Z * offset0_scalar);

			let offset1_scalar = hlenx + seam + hlenz;
			let offset1 = Quat::from_rotation_y(-FRAC_PI_4).mul_vec3(Vec3::Z * offset1_scalar);

			if control.verbose {
				let offset = prev_p + offset0 + offset1;
				println!("[{}] (odd) calc_next_pos: [{:.3} {:.3} {:.3}]", iter, offset.x, offset.y, offset.z);
			}

			offset0 + offset1
		};

		if control.verbose {
			println!("[{}] prev: [{:.3} {:.3} {:.3}]", iter, prev_p.x, prev_p.y, prev_p.z);
		}

		herrpos
	};

	// pattern is built around spline as a center position and every odd number we have an offset
	let calc_offset_from_spline = |iter : usize, spline_rotation : Quat, spline_offset_scalar : f32| {
		let angle =	if iter % 2 == 0 { -FRAC_PI_2 } else { FRAC_PI_2 };
		let init_rot = Quat::from_rotation_y(angle);

		let offset = (spline_rotation * init_rot).mul_vec3(Vec3::Z * spline_offset_scalar);
		offset
	};

	let mut fit_tile_on_spline = |iter : usize, state_t : f32, prev_p : Vec3, init_offset : f32, spline_offset_scalar : f32| -> (f32, Vec3, Quat, Vec3, Quat) {
		let mut t 		= state_t + init_offset;

		let ver 		= Vec3::Y * 1.0; // VERTICALITY

		let tile_dist_target = init_offset;
		let mut i 		= 0;
		let mut corrections : Vec<f32> = Vec::new();

		let (new_p, new_r, spline_p, spline_r) =
		loop {
			let spline_p = match spline.clamped_sample(t) {
				Some(p)	=> p,
				None	=> panic!("calc_t_corrected spline.clamped_sample failed!"),
			};

			let spline_r = spline.calc_rotation_wpos(t, spline_p);
			let spline_offset = calc_offset_from_spline(iter, spline_r, spline_offset_scalar);

			let (_width_offset_scalar, width_offset) = calc_width_offset_wrotation(true, state.iter_width as f32, spline_r);

			if control.visual_debug {
				// debug_lines.line_colored(spline_p + width_offset + ver, spline_p + spline_offset + width_offset + ver, 0.01, Color::rgb(0.8, 0.2, 0.8));
				debug_lines.line_colored(spline_p + ver, spline_p + spline_offset + ver, 0.01, Color::rgb(0.8, 0.2, 0.8));
			}

			let new_p = spline_p + spline_offset;// + width_offset;
			let new_r = herrrot * spline_r;

			let tile_dist_actual = (new_p - prev_p).length();

			let correction = tile_dist_target / tile_dist_actual;
			corrections.push(correction);
			let mut corrected_offset = init_offset;
			for c in corrections.iter() {
				corrected_offset *= c;
			}
			t = state_t + corrected_offset;

			if control.verbose {
				// println!("[{} {}] tile_dist_actual : {:.3} = new_p({:.3} {:.3} {:.3}) - prev_p({:.3} {:.3} {:.3}) (spline_p: ({:.3} {:.3} {:.3}) spline_offset_scalar: {:.3} width_offset: ({:.3} {:.3} {:.3}))", state.iter, i, tile_dist_actual, new_p.x, new_p.y, new_p.z, prev_p.x, prev_p.y, prev_p.z, spline_p.x, spline_p.y, spline_p.z, spline_offset_scalar, width_offset.x, width_offset.y, width_offset.z);
				println!("[{} {}] tile_dist_actual : {:.3} = new_p({:.3} {:.3} {:.3}) - prev_p({:.3} {:.3} {:.3}) (spline_p: ({:.3} {:.3} {:.3}) spline_offset_scalar: {:.3})", state.iter, i, tile_dist_actual, new_p.x, new_p.y, new_p.z, prev_p.x, prev_p.y, prev_p.z, spline_p.x, spline_p.y, spline_p.z, spline_offset_scalar);
				println!("[{} {}] t : {:.3} correction : tile_dist_target({:.3}) / tile_dist_actual({:.3}) = correction({:.3})", state.iter, i, t, tile_dist_target, tile_dist_actual, correction);
			}

			i += 1;

			if (1.0 - correction).abs() <= 0.01  || i >= 5 {
				break (new_p, new_r, spline_p, spline_r);
			}
		};

		if t.is_infinite() || t.is_nan() {
			panic!("calc_t_on_spline: t is invalid!");
		}

		(t, new_p, new_r, spline_p, spline_r)
	};

	if control.verbose {
		println!("[{}] getting next pos on straight line (along +z with little +-x) prev pos: [{:.3} {:.3} {:.3}]", state.iter, state.pos.x, state.pos.y, state.pos.z);
	}

	let next_pos 		= calc_next_pos(state.pos, state.iter);
	let tile_pos_delta 	= next_pos - state.pos;
	let spline_offset_scalar = if state.iter % 2 != 0 { tile_pos_delta.x } else { 0.0 };
	let t 				= state.t + tile_pos_delta.z;
	let tile_dist_target = tile_pos_delta.length();

	if t >= total_length || state.iter > 4 {
		if control.verbose {
			println!("[{}] total_length limit reached! Next tile pos: [{:.3} {:.3} {:.3}(<-)] total spline length: {:.3}", state.iter, next_pos.x, next_pos.y, next_pos.z, total_length);
		}

		let (hwidth, _) = calc_width_offset(false, (state.iter_width + 1) as f32);

		if control.verbose {
			println!("[{}] new width_offset scalar(hwidth): {:.3}", state.iter, hwidth);
		}

		if hwidth * 2.0 < config.width {
			// let spline_rotation = calc_spline_rotation(state.t, spline.clamped_sample(state.t).unwrap());
			// let (_, width_offset) = calc_width_offset(false, state.iter_width as f32, spline_rotation);
			state.pos = Vec3::Y * 0.5;// + width_offset; // VERTICALITY

			// we only keep cached positions of previous row, everything else gets cleaned up
			if state.iter_width > 0 {
				tiles_row_prev.resize_with(tiles_row_cur.len(), default);
				tiles_row_prev.copy_from_slice(tiles_row_cur.as_slice());
				tiles_row_cur.clear();
			}

			state.t = 0.0;
			state.iter = 0;

			state.iter_width += 1;

			if control.verbose {
				println!("[{}] width limit not reached({:.3}/{:.3}), inc iter_width({} -> {})", state.iter, hwidth * 2.0, config.width, state.iter_width - 1, state.iter_width);
			}
		} else {
			state.finished = true;

			if control.verbose {
				println!("[{}] width limit reached! finished!", state.iter);
			}
		}
		
		if control.verbose {
			println!("----------------------------");
		}
		return;
	}

	if control.verbose {
		println!("[{}] t: {:.3} next_pos:[{:.3} {:.3}] prev_pos: [{:.3} {:.3}] tile_pos_delta: [{:.3} {:.3}]", state.iter, t, next_pos.x, next_pos.z, state.pos.x, state.pos.z, tile_pos_delta.x, tile_pos_delta.z);
	}

	let mut prev_p = state.pos;
	prev_p.y = 0.5; // VERTICALITY

	let (t, tile_p, tile_r, spline_p, spline_r) =
	if state.iter != 0 {
		fit_tile_on_spline(state.iter, state.t, prev_p, tile_dist_target, spline_offset_scalar)
	} else {
		(t, next_pos, Quat::IDENTITY, next_pos, Quat::IDENTITY)
	};
	
	// if true || state.iter_width == 0 || state.iter == 0 || state.iter + 1 >= tiles_row_prev.len() {
		// fit_tile_on_spline(state.iter, state.t, prev_p, tile_dist_target, spline_offset_scalar)
	// } else {
	// 	// let tile_dist_target = ((hlenz + seam).powf(2.0) + (hlenx + seam + hlenx).powf(2.0)).sqrt(); // same iter prev row
	// 	// let tile_dist_target = ((hlenx).powf(2.0) + (hlenx + seam + hlenz).powf(2.0)).sqrt(); // distance to next tile in row
		
	// 	let iter_state	= tiles_row_prev[state.iter];
	// 	let (tile_dist_target, iter_state_target) = 
	// 	if state.t < total_length / 2.0 {
	// 		let tile_dist_target = ((hlenx).powf(2.0) + (hlenx + seam + hlenz).powf(2.0)).sqrt(); // distance to next tile in row
	// 		(tile_dist_target, tiles_row_cur[state.iter - 1])
	// 	} else {
	// 		let tile_dist_target = ((hlenz + seam).powf(2.0) + (hlenx + seam + hlenx).powf(2.0)).sqrt(); // same iter prev row
	// 		(tile_dist_target, iter_state)
	// 	};
	// 	calc_t_on_spline_wwidth(state.iter, state.t, spline_offset_scalar, tile_dist_target, &iter_state, &iter_state_target)
	// }

	// if state.iter == 4 && state.iter_width == 1 {
	// 	state.finished = true;
	// 	return;
	// }

	if t > total_length {
		state.finished = true;
		return;
	}

	if control.verbose {
		println!("[{}] final tile_p [{:.3} {:.3}] for t : {:.3}", state.iter, tile_p.x, tile_p.z, t);
	}

	let mut pose 		= Transform::identity();
 
	// tile offset/rotation
	//
	//

	let spline_rotation = spline.calc_rotation_wpos(t, spline_p);

	pose.translation 	= tile_p;
	pose.rotation		= herrrot * spline_rotation;

	if control.verbose {
		println!("[{}] final pose: [{:.3} {:.3} {:.3}] tile_pos_delta.x: {:.3}", state.iter, pose.translation.x, pose.translation.y, pose.translation.z, tile_pos_delta.x);
	}

	if state.iter > 0 && control.visual_debug {
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
			if control.verbose {
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
			if control.verbose {	
				println!("[{}] 1 dbg next_pos: [{:.3} {:.3}]", iter, (prev_p + offset0 + offset1).x, (prev_p + offset0 + offset1).z);
			}
	
			offset0 + offset1
		};
	}

	// spawning
	//
	//

	if !control.dry_run {
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
	state.pos	 = pose.translation;

	let iter_state = TileRowIterState{ t: t, tile_p: state.pos, spline_p: spline_p, spline_r: spline_r };
	if state.iter_width == 0 {
		tiles_row_prev.push(iter_state);
	} else {
		tiles_row_cur.push(iter_state);
	}

	if control.verbose {
		println!("----------------------------");
	}
}