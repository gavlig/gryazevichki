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

	config.material_dbg	=
	sargs.materials.add(
	StandardMaterial { 
		base_color : Color::PINK,
		..default()
	});

	sargs.commands.entity(root_e)
		.insert			(RoadWidth::W(config.width))
		.insert			(config)
		.insert			(HerringboneControl::default())
		.insert			(BrickRoadProgressState::default())
		;

	root_e
}

fn herringbone_angle(row_id : usize) -> f32 {
	if row_id % 2 == 0 {
		-FRAC_PI_4
	} else {
		FRAC_PI_4
	}
}

fn calc_total_width(
	state	: &BrickRoadProgressState,
	config	: &Herringbone2Config
) -> f32 {
	config.width - state.min_spline_offset + state.max_spline_offset
}

fn calc_single_column_offset(
	config	: &Herringbone2Config,
) -> f32 {
	let hlenx	= config.hsize.x;
	let hlenz	= config.hsize.z;
	let seam	= config.hseam * 2.0;

	let single_column_offset = ((hlenz + seam).powf(2.0) + (hlenx + seam + hlenx).powf(2.0)).sqrt();

	single_column_offset
}

fn calc_column_count(
	state	: &BrickRoadProgressState,
	config	: &Herringbone2Config,
) -> usize {
	let single_column_offset = calc_single_column_offset(config);
	let total_width	= calc_total_width(state, config);

	calc_column_count_ext(single_column_offset, total_width)
}

fn calc_column_count_ext(
	single_column_offset : f32,
	total_width : f32
) -> usize {
	let column_count	= (total_width / single_column_offset).round() + 1.0;

	column_count.round() as usize
}

pub fn calc_init_column_offset(
	state	: &BrickRoadProgressState,
	config	: &Herringbone2Config,
) -> f32 {
	let single_column_offset = calc_single_column_offset(config);

	let hwidth			= config.width / 2.0;
	let min_offset		= state.min_spline_offset - hwidth;
	let column_id		= (min_offset / single_column_offset).round();

	let offset 			= column_id * single_column_offset;
	offset
}

// calculate tile position in a row, pattern: herringbone2
// "in a row" means that we don't take into account row number and it should be accounted for elsewhere

/*

/
\
/
\
/

*/

// ^ that's what one row of tiles looks like
fn calc_tile_row_pos(
	prev_p	: Vec3,
	row_id 	: usize,
	config	: &Herringbone2Config,
	log		: impl Fn(String)
) -> Vec3 {
	if row_id == 0 {
		return prev_p;
	}

	let hlenx			= config.hsize.x;
	let hlenz			= config.hsize.z;
	let seam			= config.hseam * 2.0;

	let offset0_scalar 	= hlenz - hlenx;
	let offset1_scalar 	= hlenx + seam + hlenz;
	let (rotation0, rotation1) = 
	if row_id % 2 == 0 {
		(-FRAC_PI_4, FRAC_PI_4)
	} else {
		(FRAC_PI_4, -FRAC_PI_4)
	};
	let offset0 = Quat::from_rotation_y(rotation0).mul_vec3(Vec3::Z * offset0_scalar);
	let offset1 = Quat::from_rotation_y(rotation1).mul_vec3(Vec3::Z * offset1_scalar);
	let herrpos = prev_p + offset0 + offset1;

	log(format!("calc_tile_pos: [{:.3} {:.3} {:.3}]", herrpos.x, herrpos.y, herrpos.z));
	log(format!("         prev: [{:.3} {:.3} {:.3}]", prev_p.x, prev_p.y, prev_p.z));

	herrpos
}

// pattern is built around points on spline as a center position and every odd number we have a side offset
fn calc_spatial_offset(rotation : Quat, offset_scalar : f32) -> Vec3 {
	// let offset = 
	(rotation).mul_vec3(Vec3::X * offset_scalar)
}

fn fit_tile_on_spline(
	init_t_delta         : f32,
	tile_dist_target   	 : f32,
	column_offset_scalar : f32,
	spline_offset_scalar : f32,
	pattern_rotation     : Quat,
	state                : &BrickRoadProgressState,
	spline               : &Spline,
	log                  : impl Fn(String)
) -> (f32, Vec3, Quat, Vec3, Quat) {
	let state_t		= state.t;
	let prev_p		= state.pos;
	let t_without_spline = state_t + init_t_delta;

	let mut t 		= t_without_spline;
	let mut i 		= 0;
	let mut corrections : Vec<f32> = Vec::new();

	let (new_p, new_r, spline_p, spline_r) =
	loop {
		let spline_p = match spline.clamped_sample(t) {
			Some(p)	=> p,
			None	=> panic!("fit_tile_on_spline spline.clamped_sample failed! t: {:.3} spline.keys: {:?}", t, spline.keys()),
		};

		let spline_r = spline.calc_rotation_wpos(t, spline_p);
		let spline_offset = calc_spatial_offset(spline_r, spline_offset_scalar);
		let column_offset = calc_spatial_offset(spline_r, column_offset_scalar);

		// if control.visual_debug {
		// 	debug_lines.line_colored(spline_p + ver, spline_p + spline_offset + ver, 0.01, Color::rgb(0.8, 0.2, 0.8));
		// }

		let new_p = spline_p + spline_offset + column_offset;
		let new_r = pattern_rotation * spline_r;

		let tile_dist_actual = (new_p - prev_p).length();

		let correction = tile_dist_target / tile_dist_actual;
		corrections.push(correction);
		let mut corrected_offset = init_t_delta;
		for c in corrections.iter() {
			corrected_offset *= c;
		}
		t = state_t + corrected_offset;

		log(format!("[{}] tile_dist_target : {:.3} tile_dist_actual : {:.3} = new_p({:.3} {:.3} {:.3}) - prev_p({:.3} {:.3} {:.3}) (spline_p: ({:.3} {:.3} {:.3}) spline_offset_scalar: {:.3})", i, tile_dist_target, tile_dist_actual, new_p.x, new_p.y, new_p.z, prev_p.x, prev_p.y, prev_p.z, spline_p.x, spline_p.y, spline_p.z, spline_offset_scalar));
		log(format!("[{}] t : {:.3} correction : tile_dist_target({:.3}) / tile_dist_actual({:.3}) = correction({:.3})", i, t, tile_dist_target, tile_dist_actual, correction));

		i += 1;

		if (1.0 - correction).abs() <= 0.01  || i >= 5 {
			// let column_offset_rotated = spline_r.mul_vec3(column_offset);
			// let new_p = new_p + column_offset;//_rotated; // TODO: check what works better

			log(format!("[{}] fit_tile_on_spline finished! t: {:.3} t_without_spline: {:.3} ratio: {:.3}", i, t, t_without_spline, t / t_without_spline));

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
	mut state 			: &mut BrickRoadProgressState,
		log				: impl Fn(String)
) -> bool {
	let total_length	= spline.total_length();

	// cheat/debug: make rows shorter to avoid having long log. Add/Remove "|| true" to turn off/on.
	let debug			= state.row_id < 2 || true;
	if t < total_length && debug {
		// very verbose
		// log(format!("end condition (t >= total_length) not met! t: {:.3} total_length: {:.3} state.row_id: {:.3}", t, total_length, state.row_id));
		return false;
	}
	log(format!("total_length limit reached! t: {:.3} total spline length: {:.3}", t, total_length));

	// cheat/debug: make rows shorter to avoid having long log. Add/Remove "|| true" to turn off/on.
	let debug			= state.column_id < 1 || true;

	let column_count = calc_column_count(state, config);
	if state.column_id < column_count && debug {
		state.t			= 0.0;
		state.row_id 	= 0;
		state.column_id	+= 1;

		// let column_offset = calc_column_offset(state.column_id, state, config);

		let column_offset = (state.column_id as f32 * calc_single_column_offset(config)) + calc_init_column_offset(state, config);

		state.pos		= Vec3::Y * 0.5 + Vec3::X * column_offset; // VERTICALITY

		log(format!("width limit not reached(max column_id: {}), inc column_id({} -> {})", column_count - 1, state.column_id - 1, state.column_id));
	} else {
		state.set_default();
		state.finished = true;

		log(format!("width limit reached! finished! last column_id: {} column_count: {}", state.column_id, column_count));
	}
	
	log(format!("----------------------------"));

	return true;
}

fn spawn_tile(
	pose	: Transform,
	filtered_out : bool,
	state	: &mut BrickRoadProgressState,
	config	: &Herringbone2Config,
	sargs	: &mut SpawnArguments,
	log		: impl Fn(String)
) {
    // spawn first brick with a strong reference to keep reference count > 0 and keep mesh/material from dying when out of scope
    let (me, mut ma) = (config.mesh.clone_weak(), config.material.clone_weak());

	if filtered_out {
		// ma = config.material_dbg.clone_weak();
		log(format!("tile was filtered out!"));
		return;
	}

    // this can be done without macro now, but i need it for a reference
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
		config			: &Herringbone2Config,
		spline 			: &Spline,
		_ass			: &Res<AssetServer>,
		sargs			: &mut SpawnArguments,
		control			: &HerringboneControl,

	mut debug_lines		: &mut ResMut<DebugLines>,
) {
	// a little logging helper lambda
	let ir 				= state.column_id;
	let il 				= state.row_id;
	let log 			= |str_in : String| {
		if control.verbose {
	 		println!	("[{} {}] {}", ir, il, str_in);
		}
	};

	let total_length 	= spline.total_length();

	log(format!("new brick_road_iter! spline.total_length: {:.3} column_id: {} row_id: {}", total_length, state.column_id, state.row_id));

	let prev_pos		= state.pos;
	// tile position for current iteration on a straight line
	let next_pos 		= calc_tile_row_pos(prev_pos, state.row_id, config, log);

	let tile_pos_delta 	= next_pos - prev_pos;

	// on a straight line tile position works as "t" (parameter for spline sampling). Later on t gets adjusted in fit_tile_on_spline to keep tiles evenly spaced on spline
	// z is used explicitely here because we don't want to deal with 2 dimensions in spline sampling and offset by x will be added later
	let init_t_delta	= tile_pos_delta.z;
	let t 				= state.t + init_t_delta;

	//
	//
	// Checking end conditions

	if end_conditions_met(t, spline, config, state, log) {
		return;
	}

	log(format!("t: {:.3} next_pos:[{:.3} {:.3} {:.3}] prev_pos: [{:.3} {:.3} {:.3}] tile_pos_delta: [{:.3} {:.3} {:.3}]", t, next_pos.x, next_pos.y, next_pos.z, state.pos.x, state.pos.y, state.pos.z, tile_pos_delta.x, tile_pos_delta.y, tile_pos_delta.z));

	//
	//
	// Putting tile on spline

	// in herringbone pattern every next tile is rotated +-45 degrees from 0
	let pattern_angle	= herringbone_angle(state.row_id);
	let pattern_rotation = Quat::from_rotation_y(pattern_angle);

	let spline_p 		= spline.calc_position(t);

	state.max_spline_offset = spline_p.x.max(state.max_spline_offset);
	state.min_spline_offset = spline_p.x.min(state.min_spline_offset);

	// 
	//
	// Final pose
	let mut pose 		= Transform::identity();
	pose.translation 	= next_pos;
	pose.rotation		= pattern_rotation;

	// 
	//
	// Spawning

	let x				= pose.translation.x;
	let spx				= spline_p.x;
	let hwidth			= config.width / 2.0;
	let in_range 		= (spx - hwidth) <= x && x <= (spx + hwidth);
	if !control.dry_run {
		spawn_tile		(pose, !in_range, state, config, sargs, log);
	}

	//
	//
	// cheat/debug: end on certain column/row id to avoid long logs etc
	let debug			= false;
	if state.row_id == 5 && state.column_id == 1 && debug {
		log(format!("DEBUG FULL STOP state.row_id: {} state.column_id: {}", state.row_id, state.column_id));
		state.finished = true;
		return;
	}

	//
	//
	// Iteration ended
	state.row_id		+= 1;
	state.t		 		= t;
	state.pos	 		= next_pos;

	log(format!("----------------------------"));
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