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
	let mut config = config_in.clone();

	let root_e = bevy_spline::spawn::new(
		transform,
		config.t_max,
		120.0,
		Color::rgb(0.2, 0.2, 0.2),
		polylines,
		polyline_materials,
		sargs
	);

	config.root_entity	= root_e;

	let tile_size		= config.hsize * 2.0;

	config.mesh = sargs.meshes.add(
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
		.insert			(HerringboneControl::default())
		.insert			(BrickRoadProgressState::default())
		;

	root_e
}
fn calc_row_offset(
	iter_row_in : usize,
	config	 	: &Herringbone2Config,
) -> f32 {
	let iter_row 		= iter_row_in as f32;
	let hlenx			= config.hsize.x;
	let hlenz			= config.hsize.z;
	let seam			= config.hseam * 2.0;

	let single_offset 	= ((hlenz + seam).powf(2.0) + (hlenx + seam + hlenx).powf(2.0)).sqrt();
	let offset 			= iter_row * single_offset;
	offset
}

fn calc_row_offset_wrotation(
	positive : bool,
	iter_row : usize,
	config	 : &Herringbone2Config,
	rotation : Quat
) -> Vec3 {
	let offset 			= calc_row_offset(iter_row, config);

	let mut offset_x 	= Vec3::new(offset, 0.0, 0.0);
	if !positive 		{ offset_x.x = -offset_x.x; }
	offset_x 			= rotation.mul_vec3(offset_x);
	offset_x
}

// with given tile parameters and iteration number calculate tile position, pattern: herringbone2
fn calc_tile_pos(
	prev_p	: Vec3,
	iter 	: usize,
	config	: &Herringbone2Config,
	control : &HerringboneControl,
	log		: impl Fn(String)
) -> Vec3 {
	let ver = Vec3::Y * 0.5; // VERTICALITY

	if iter == 0 {
		return prev_p;
	}

	let hlenx			= config.hsize.x;
	let hlenz			= config.hsize.z;
	let seam			= config.hseam * 2.0;

	let offset0_scalar = hlenz - hlenx;
	let offset1_scalar = hlenx + seam + hlenz;
	let (rotation0, rotation1) = 
	if iter % 2 == 0 {
		(-FRAC_PI_4, FRAC_PI_4)
	} else {
		(FRAC_PI_4, -FRAC_PI_4)
	};
	let offset0 = Quat::from_rotation_y(rotation0).mul_vec3(Vec3::Z * offset0_scalar);
	let offset1 = Quat::from_rotation_y(rotation1).mul_vec3(Vec3::Z * offset1_scalar);
	let herrpos = prev_p + offset0 + offset1;

	if control.verbose {
		log(format!("calc_tile_pos: [{:.3} {:.3} {:.3}]", herrpos.x, herrpos.y, herrpos.z));
		log(format!("         prev: [{:.3} {:.3} {:.3}]", prev_p.x, prev_p.y, prev_p.z));
	}

	herrpos
}

// pattern is built around points on spline as a center position and every odd number we have an offset
fn calc_spatial_offset_from_spline(iter : usize, spline_rotation : Quat, spline_offset_scalar : f32) -> Vec3 {
	let angle =	if iter % 2 == 0 { -FRAC_PI_2 } else { FRAC_PI_2 };
	let init_rot = Quat::from_rotation_y(angle);

	// let offset = 
	(spline_rotation * init_rot).mul_vec3(Vec3::Z * spline_offset_scalar)
}

fn fit_tile_on_spline(
	state 			: &BrickRoadProgressState,
	init_t_delta 	: f32,
	tile_dist_target: f32,
	row_offset_scalar : f32,
	spline_offset_scalar : f32,
	pattern_rotation : Quat,
	spline 			: &Spline,
	control			: &HerringboneControl,
	log				: impl Fn(String)
) -> (f32, Vec3, Quat, Vec3, Quat) {
	let iter		= state.iter;
	let state_t		= state.t;
	let prev_p		= state.pos;

	let mut t 		= state_t + init_t_delta;
	let row_offset	= Vec3::new(row_offset_scalar, 0.0, 0.0);

	let ver 		= Vec3::Y * 1.0; // VERTICALITY

	let mut i 		= 0;
	let mut corrections : Vec<f32> = Vec::new();

	let (new_p, new_r, spline_p, spline_r) =
	loop {
		let spline_p = match spline.clamped_sample(t) {
			Some(p)	=> p,
			None	=> panic!("fit_tile_on_spline spline.clamped_sample failed!"),
		};

		let spline_r = spline.calc_rotation_wpos(t, spline_p);
		let spline_offset = calc_spatial_offset_from_spline(iter, spline_r, spline_offset_scalar);

		// if control.visual_debug {
		// 	debug_lines.line_colored(spline_p + ver, spline_p + spline_offset + ver, 0.01, Color::rgb(0.8, 0.2, 0.8));
		// }

		let new_p = spline_p + spline_offset;
		let new_r = pattern_rotation * spline_r;

		let tile_dist_actual = (new_p - prev_p).length();

		let correction = tile_dist_target / tile_dist_actual;
		corrections.push(correction);
		let mut corrected_offset = init_t_delta;
		for c in corrections.iter() {
			corrected_offset *= c;
		}
		t = state_t + corrected_offset;

		if control.verbose {
			log(format!("[{}] tile_dist_target : {:.3} tile_dist_actual : {:.3} = new_p({:.3} {:.3} {:.3}) - prev_p({:.3} {:.3} {:.3}) (spline_p: ({:.3} {:.3} {:.3}) spline_offset_scalar: {:.3})", i, tile_dist_target, tile_dist_actual, new_p.x, new_p.y, new_p.z, prev_p.x, prev_p.y, prev_p.z, spline_p.x, spline_p.y, spline_p.z, spline_offset_scalar));
			log(format!("[{}] t : {:.3} correction : tile_dist_target({:.3}) / tile_dist_actual({:.3}) = correction({:.3})", i, t, tile_dist_target, tile_dist_actual, correction));
		}

		i += 1;

		if (1.0 - correction).abs() <= 0.01  || i >= 5 {
			let row_offset_rotated = spline_r.mul_vec3(row_offset);
			let new_p = new_p + row_offset;//_rotated; // TODO: check what works better

			break (new_p, new_r, spline_p, spline_r);
		}
	};

	if t.is_infinite() || t.is_nan() {
		panic!("fit_tile_on_spline: t is invalid! t: {:.3}", t);
	}

	(t, new_p, new_r, spline_p, spline_r)
}

fn end_conditions_met(
		t            	: f32,
		spline 			: &Spline,
		config			: &Herringbone2Config,
		control      	: &HerringboneControl,
	mut state 			: &mut BrickRoadProgressState,
		log				: impl Fn(String)
) -> bool {
	let total_length	= spline.total_length();

	if t < total_length { // || state.iter > 2 {
		if control.verbose {
			log(format!("end condition (t >= total_length) not met! t: {:.3} total_length: {:.3} state.iter: {:.3}", t, total_length, state.iter));
		}
		return false;
	}

	let row_offset_scalar = calc_row_offset(state.iter_row + 1, config);
	
	if control.verbose {
		log(format!("total_length limit reached! t: {:.3} total spline length: {:.3} row_offset_scalar: {:.3}", t, total_length, row_offset_scalar));
	}

	if row_offset_scalar * 2.0 < config.width {
		let init_spline_r = spline.calc_init_rotation();
		let row_offset 	= init_spline_r.mul_vec3(Vec3::new(row_offset_scalar, 0.0, 0.0));
//		if !positive 	{ row_offset.x = -row_offset.x; }

		state.pos = Vec3::Y * 0.5 + row_offset; // VERTICALITY

		// we only keep cached positions of previous row, everything else gets cleaned up
		// if state.iter_row > 0 {
		// 	tiles_row_prev.resize_with(tiles_row_cur.len(), default);
		// 	tiles_row_prev.copy_from_slice(tiles_row_cur.as_slice());
		// 	tiles_row_cur.clear();
		// }

		state.t = 0.0;
		state.iter = 0;
		state.iter_row += 1;

		if control.verbose {
			log(format!("width limit not reached({:.3}/{:.3}), inc iter_row({} -> {})", row_offset_scalar * 2.0, config.width, state.iter_row - 1, state.iter_row));
		}
	} else {
		state.set_default();
		state.finished = true;

		if control.verbose {
			log(format!("width limit reached! finished!"));
		}
	}
	
	if control.verbose {
		log(format!("----------------------------"));
	}

	return true;
}

fn spawn_tile(
	pose	: Transform,
	config	: &mut Herringbone2Config,
	state	: &mut BrickRoadProgressState,
	sargs	: &mut SpawnArguments
) {
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
			.insert_bundle	(PickableBundle::default())
//			.insert			(Draggable::default())
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

pub fn brick_road_iter(
	mut state			: &mut BrickRoadProgressState,
	mut	config			: &mut Herringbone2Config,
		spline 			: &Spline,
		tiles_row_prev	: &mut Vec<TileRowIterState>,
		tiles_row_cur	: &mut Vec<TileRowIterState>,
		transform		: &GlobalTransform,
		_ass			: &Res<AssetServer>,
		sargs			: &mut SpawnArguments,
		control			: &HerringboneControl,

	mut debug_lines		: &mut ResMut<DebugLines>,
) {
	// a little logging helper lambda
	let ir 				= state.iter_row;
	let il 				= state.iter;
	let log 			= |str_in : String| {
	 	println!		("[{} {}] {}", ir, il, str_in);
	};

	if control.verbose {
		log(format!("getting next tile pos on straight line (along +z with little +-x) prev pos: [{:.3} {:.3} {:.3}]", state.pos.x, state.pos.y, state.pos.z));
	}

	let row_offset		= calc_row_offset(state.iter_row, config);
//	let spline			= spline_in.clone_with_offset(row_offset);
	let total_length 	= spline.total_length();

	if control.verbose {
		log(format!("new brick_road_iter! t: {:.3} spline.total_length: {:.3} row_offset(width): {:.3}", state.t, total_length, row_offset));
	}

	// tile position for current iteration on a straight line
	let next_pos 		= calc_tile_pos(state.pos, state.iter, config, control, log);

	let prev_pos		= state.pos;
	let tile_pos_delta 	= next_pos - prev_pos;

	// on a straight line tile position works as t (parameter for spline sampling). Later on t gets adjusted in fit_tile_on_spline to keep tiles evenly spaced on spline
	// z is used explicitely here because we don't want to deal with 2 dimensions in spline sampling and offset by x will be accounted for later
	let init_t_delta	= tile_pos_delta.z;
	let t 				= state.t + init_t_delta;

	//
	//
	// Checking end conditions

	if end_conditions_met(t, spline, config, control, state, log) {
		return;
	}

	if control.verbose {
		log(format!("[{}] t: {:.3} next_pos:[{:.3} {:.3} {:.3}] prev_pos: [{:.3} {:.3} {:.3}] tile_pos_delta: [{:.3} {:.3} {:.3}]", state.iter, t, next_pos.x, next_pos.y, next_pos.z, state.pos.x, state.pos.y, state.pos.z, tile_pos_delta.x, tile_pos_delta.y, tile_pos_delta.z));
	}

	//
	//
	// Putting tile on spline

	// in herringbone pattern every next tile is rotated +-45 degrees from 0
	let pattern_angle	= if state.iter % 2 == 0 { FRAC_PI_4 } else { -FRAC_PI_4 };
	let pattern_rotation = Quat::from_rotation_y(pattern_angle);

	let (t, tile_p, tile_r, spline_p, spline_r) =
	if state.iter != 0 {
		let row_offset	= calc_row_offset(state.iter_row, config);
		let tile_dist_target = tile_pos_delta.length();
		// in herringbone pattern every odd tile we have horizontal offset from center of row
		let spline_offset_scalar = if state.iter % 2 != 0 { tile_pos_delta.x } else { 0.0 };
		fit_tile_on_spline(state, init_t_delta, tile_dist_target, row_offset, spline_offset_scalar, pattern_rotation, spline, control, log)
	} else {
		let p 			= next_pos;
		let r 			= spline.calc_init_rotation();
		(t, p, r * pattern_rotation, p, Quat::IDENTITY)
	};

	if control.verbose {
		log(format!("final tile_p [{:.3} {:.3} {:.3}] for t : {:.3}", tile_p.x, tile_p.y, tile_p.z, t));
	}

	//
	//
	// debug/cheats
	if state.iter == 2 && state.iter_row == 3 {
		state.finished = true;
		return;
	}

	// 
	//
	// Tile offset/rotation
	let mut pose 		= Transform::identity();
	pose.translation 	= tile_p;
	pose.rotation		= tile_r;

	if control.verbose {
		log(format!("final pose: [{:.3} {:.3} {:.3}] tile_pos_delta.x: {:.3}", pose.translation.x, pose.translation.y, pose.translation.z, tile_pos_delta.x));
	}

	// 
	//
	// Spawning
	if !control.dry_run {
		spawn_tile		(pose, config, state, sargs);
	}

	//
	//
	// Iteration ended
	state.iter	+= 1;
	state.t		 = t;
	state.pos	 = pose.translation;

	let iter_state = TileRowIterState{ t: t, tile_p: state.pos, tile_r: tile_r, spline_p: spline_p, spline_r: spline_r };
	if state.iter_row == 0 {
		tiles_row_prev.push(iter_state);
	} else {
		tiles_row_cur.push(iter_state);
	}

	if control.verbose {
		log(format!("----------------------------"));
	}
}

// fn visual_debug()
// {
// 	if state.iter > 0 && control.visual_debug {
// 		let ver = Vec3::Y * 1.5;
// 		let iter = state.iter;
// 		let prev_p = state.pos;

// 		if iter % 2 == 0 {
// 			let offset0_scalar = hlenz - hlenx;
// 			let init_rot = Quat::from_rotation_y(-FRAC_PI_4);
// 			let offset0 = (spline_r * init_rot).mul_vec3(Vec3::Z * offset0_scalar);
	
// 			// debug_lines.line_colored(prev_p + ver, prev_p + offset0 + ver, 0.01, Color::rgb(0.8, 0.2, 0.2));
	
// 			let offset1_scalar = hlenx + seam + hlenz;
// 			let init_rot = Quat::from_rotation_y(FRAC_PI_4);
// 			let offset1 = (spline_r * init_rot).mul_vec3(Vec3::Z * offset1_scalar);

// 			// debug_lines.line_colored(prev_p + offset0 + ver, prev_p + offset0 + offset1 + ver, 0.01, Color::rgb(0.8, 0.2, 0.2));
// 			if control.verbose {
// 				let next_pos = prev_p + offset0 + offset1;
// 				log(format!("[{}] 0 dbg next_pos: [{:.3} {:.3} {:.3}]", iter, next_pos.x, next_pos.y, next_pos.z));
// 			}
	
// 			offset0 + offset1
// 		} else {
// 			let offset0_scalar = hlenz - hlenx;
// 			let init_rot = Quat::from_rotation_y(FRAC_PI_4);
// 			let offset0 = (spline_r * init_rot).mul_vec3(Vec3::Z * offset0_scalar);
	
// 			// debug_lines.line_colored(prev_p + ver, prev_p + offset0 + ver, 0.01, Color::rgb(0.2, 0.2, 0.8));
	
// 			let offset1_scalar = hlenx + seam + hlenz;
// 			let init_rot = Quat::from_rotation_y(-FRAC_PI_4);
// 			let offset1 = (spline_r * init_rot).mul_vec3(Vec3::Z * offset1_scalar);
	
// 			// debug_lines.line_colored(prev_p + offset0 + ver, prev_p + offset0 + offset1 + ver, 0.01, Color::rgb(0.2, 0.2, 0.8));
// 			if control.verbose {	
// 				let next_pos = prev_p + offset0 + offset1;
// 				log(format!("[{}] 1 dbg next_pos: [{:.3} {:.3} {:.3}]", iter, next_pos.x, next_pos.y, next_pos.z));
// 			}
	
// 			offset0 + offset1
// 		};
// 	}
// }
