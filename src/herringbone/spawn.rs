use bevy				:: prelude :: { * };
use bevy_rapier3d		:: prelude :: { * };
use bevy_mod_picking	:: { * };
use bevy_polyline		:: { prelude :: * };
use bevy_mod_gizmos		:: { * };

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
		config.length,
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

fn herringbone_angle(pattern_iter : usize) -> f32 {
	if pattern_iter % 2 == 0 {
		-FRAC_PI_4
	} else {
		FRAC_PI_4
	}
}

fn calc_total_width(
	state	: &BrickRoadProgressState,
	config	: &Herringbone2Config
) -> f32 {
	config.width
}

/*

/
\
/
\
/

*/

// ^ that's what one row of tiles looks like

fn calc_next_tile_pos_zplus(
	prev_p	: Vec3,
	pattern_iter : usize,
	config	: &Herringbone2Config,
	log		: impl Fn(String)
) -> Vec3 {
	let hlenx			= config.hsize.x;
	let hlenz			= config.hsize.z;
	let seam			= config.hseam * 2.0;

	let offset0_length 	= hlenz - hlenx;
	let offset1_length 	= hlenx + seam + hlenz;
	let (offset0_rotation, offset1_rotation) = 
	if pattern_iter % 2 == 0 {
		(-FRAC_PI_4, FRAC_PI_4)
	} else {
		(FRAC_PI_4, -FRAC_PI_4)
	};
	let offset0 = Quat::from_rotation_y(offset0_rotation).mul_vec3(Vec3::Z * offset0_length);
	let offset1 = Quat::from_rotation_y(offset1_rotation).mul_vec3(Vec3::Z * offset1_length);
	let herrpos = prev_p + offset0 + offset1;

	log(format!("calc_next_tile_row_zplus: [{:.3} {:.3} {:.3}]", herrpos.x, herrpos.y, herrpos.z));
	log(format!("                    prev: [{:.3} {:.3} {:.3}]", prev_p.x, prev_p.y, prev_p.z));

	herrpos
}

fn calc_next_tile_pos_zminus(
	prev_p	: Vec3,
	pattern_iter : usize,
	config	: &Herringbone2Config,
	log		: impl Fn(String)
) -> Vec3 {
	let hlenx			= config.hsize.x;
	let hlenz			= config.hsize.z;
	let seam			= config.hseam * 2.0;

	let offset0_length 	= hlenz + seam + hlenx;
	let offset1_length 	= hlenz - hlenx;
	let (offset0_rotation, offset1_rotation) = 
	if pattern_iter % 2 == 0 {
		(-FRAC_PI_4, FRAC_PI_4)
	} else {
		(FRAC_PI_4, -FRAC_PI_4)
	};
	let offset0 = Quat::from_rotation_y(offset0_rotation).mul_vec3(Vec3::Z * -offset0_length);
	let offset1 = Quat::from_rotation_y(offset1_rotation).mul_vec3(Vec3::Z * -offset1_length);
	let herrpos = prev_p + offset0 + offset1;

	log(format!("calc_next_tile_row_zminus: [{:.3} {:.3} {:.3}]", herrpos.x, herrpos.y, herrpos.z));
	log(format!("                     prev: [{:.3} {:.3} {:.3}]", prev_p.x, prev_p.y, prev_p.z));

	herrpos
}

fn calc_next_tile_pos_xplus(
	prev_p	: Vec3,
	pattern_iter : usize,
	config	: &Herringbone2Config,
	log		: impl Fn(String)
) -> Vec3 {
	let hlenx			= config.hsize.x;
	let hlenz			= config.hsize.z;
	let seam			= config.hseam * 2.0;

	let offset0_length 	= hlenz + seam;
	let offset1_length 	= hlenx + seam + hlenx;
	let (offset0_rotation, offset1_rotation) = 
	if pattern_iter % 2 == 0 {
		(-FRAC_PI_4, FRAC_PI_4)
	} else {
		(FRAC_PI_4, -FRAC_PI_4)
	};
	let offset0 = Quat::from_rotation_y(offset0_rotation).mul_vec3(Vec3::Z * offset0_length);
	let offset1 = Quat::from_rotation_y(offset1_rotation).mul_vec3(Vec3::Z * -offset1_length);
	let herrpos = prev_p + offset0 + offset1;

	log(format!("calc_next_tile_row_xplus: [{:.3} {:.3} {:.3}]", herrpos.x, herrpos.y, herrpos.z));
	log(format!("                    prev: [{:.3} {:.3} {:.3}]", prev_p.x, prev_p.y, prev_p.z));

	herrpos
}

fn calc_next_tile_pos_xminus(
	prev_p	: Vec3,
	pattern_iter : usize,
	config	: &Herringbone2Config,
	log		: impl Fn(String)
) -> Vec3 {
	let hlenx			= config.hsize.x;
	let hlenz			= config.hsize.z;
	let seam			= config.hseam * 2.0;

	let offset0_length 	= hlenx + seam + hlenx;
	let offset1_length 	= hlenz + seam;
	let (offset0_rotation, offset1_rotation) = 
	if pattern_iter % 2 == 0 {
		(-FRAC_PI_4, FRAC_PI_4)
	} else {
		(FRAC_PI_4, -FRAC_PI_4)
	};
	let offset0 = Quat::from_rotation_y(offset0_rotation).mul_vec3(Vec3::Z * offset0_length);
	let offset1 = Quat::from_rotation_y(offset1_rotation).mul_vec3(Vec3::Z * -offset1_length);
	let herrpos = prev_p + offset0 + offset1;

	log(format!("calc_next_tile_row_xminus: [{:.3} {:.3} {:.3}]", herrpos.x, herrpos.y, herrpos.z));
	log(format!("                     prev: [{:.3} {:.3} {:.3}]", prev_p.x, prev_p.y, prev_p.z));

	herrpos
}

fn calc_next_tile_pos(
	state 	: &mut BrickRoadProgressState,
	config	: &Herringbone2Config,
	log		: impl Fn(String)	
) -> Vec3 {
	let prev_pos	= state.pos;
	let pattern_iter= state.pattern_iter;

	let next_pos = match state.dir {
		Direction2D::Up    => calc_next_tile_pos_zplus (prev_pos, pattern_iter, config, log),
		Direction2D::Right => calc_next_tile_pos_xminus(prev_pos, pattern_iter, config, log),
		Direction2D::Down  => calc_next_tile_pos_zminus(prev_pos, pattern_iter, config, log),
		Direction2D::Left  => calc_next_tile_pos_xplus (prev_pos, pattern_iter, config, log),
	};

	next_pos
}

fn calc_next_tile_pos_on_road(
		next_pos_out	: &mut Vec3,
		state_out		: &mut BrickRoadProgressState,
		spline 			: &Spline,
		config			: &Herringbone2Config,
		log				: impl Fn(String)
) -> bool {
	let mut state		= state_out.clone();
	let mut next_pos	= Vec3::ZERO;
	let mut next_pos_found = false;
	let mut dir_cnt		= 0;
	let init_dir		= state.dir;
	let init_pattern_iter = state.pattern_iter;

	while !next_pos_found && dir_cnt < 4 {
		if state.dir.is_vertical() && init_dir.is_horizontal() {
			state.pattern_iter = if init_pattern_iter == 0 { 1 } else { 0 };
			log(format!("[{}] switching pattern_iter to {}! init_dir: {:?} new_dir: {:?}", dir_cnt, state.pattern_iter, init_dir, state.dir));
		}
		next_pos		= calc_next_tile_pos(&mut state, config, &log);

		log(format!("[{}] looking for available directions. dir: {:?}, next pos: [{:.3} {:.3} {:.3}] prev pos: [{:.3} {:.3} {:.3}] pattern_iter: {:?}", dir_cnt, state.dir, next_pos.x, next_pos.y, next_pos.z, state.pos.x, state.pos.y, state.pos.z, state.pattern_iter));

		// just probe with next_pos.z as t parameter to find closest spline point
		let spline_p	= spline.calc_position(next_pos.z);
		let distance_to_spline = (next_pos - spline_p).length();

		let too_far_from_spline = distance_to_spline > (config.width / 2.0);
		if !too_far_from_spline {
			next_pos_found = true;
			log(format!("found next pos direction! {:?} distance_to_spline: {:.3}", state.dir, distance_to_spline));
		} else {
			if Direction2D::Up == state.dir {
				state.pos = next_pos;
				state.t = next_pos.z;
				log(format!("couldn't go up but switched to next row anyway! new t: {:.3}", state.t));
			}

			state.set_next_direction();
			log(format!("too far! switching direction to {:?} pattern {} (spline_p: [{:.3} {:.3} {:.3}] distance_to_spline: {:.3})", state.dir, state.pattern_iter, spline_p.x, spline_p.y, spline_p.z, distance_to_spline));
		}

		dir_cnt 		+= 1;
	}

	if !next_pos_found {
		log(format!("end condition met! last iter: {} last pos: [{:.3} {:.3} {:.3}]", state.iter, state.pos.x, state.pos.y, state.pos.z));
		log(format!("----------------------------"));
	} else {
		// we went up one iteration, let's fill another row of tiles on the road! (switching from up to left)
		if init_dir == Direction2D::Up {
			state_out.set_next_direction();
			state_out.t = next_pos.z;
			log(format!("new t: {:.3} We went up one iteration, let's fill another row of tiles on the road! (switching from up to left)", state_out.t));
		}

		*state_out		= state;
		*next_pos_out	= next_pos;
	}
	
	return next_pos_found;
}

fn find_t_on_spline(
	target_pos		: Vec3,
	state 			: &BrickRoadProgressState,
	spline			: &Spline,
	config			: &Herringbone2Config,
	log				: impl Fn(String)
) -> f32 {
	let t_without_spline = state.t;

	if t_without_spline <= 0.0 || t_without_spline >= spline.total_length() {
		return t_without_spline;
	}

	let init_t_delta = (target_pos - state.pos).length();

	let mut t 		= t_without_spline;
	let mut i 		= 0;
	let mut corrections : Vec<f32> = Vec::new();
	let eps			= 0.001; // precision is 1mm

	log(format!("find_t_on_spline started! t_without_spline: {:.3} ", t_without_spline));

	loop {
		let spline_p = spline.calc_position(t);
		log(format!("[{}] spline_pos: [{:.3} {:.3} {:.3}] target_pos: [{:.3} {:.3} {:.3}]", i, spline_p.x, spline_p.y, spline_p.z, target_pos.x, target_pos.y, target_pos.z));

		if spline_p.abs_diff_eq(target_pos, eps) {
			log(format!("[{}] find_t_on_spline finished(spline_p == target_pos)! t: {:.3} t_without_spline: {:.3} ratio: {:.3}", i, t, t_without_spline, t / t_without_spline));
			break;
		}

		if (target_pos - spline_p).length() > config.width * 2.0 {
			log(format!("[{}] find_t_on_spline finished(tile too far from spline)! t: {:.3} t_without_spline: {:.3} ratio: {:.3}", i, t, t_without_spline, t / t_without_spline));
			break;
		}

		let spline_dir = spline.calc_dir_wpos(t, spline_p);
		let spline_r = Quat::from_rotation_arc(Vec3::Z, spline_dir);

		let spline2target_dir = (target_pos - spline_p).normalize();
		let spline2target_r = Quat::from_rotation_arc(Vec3::Z, spline2target_dir);

		// if prev angle was better keep it?
		let angle = spline_r.angle_between(spline2target_r).to_degrees();

		log(format!("[{}] angle: {:.3}", i, angle));

		let angle_diff = (90.0 - angle).abs();
		if angle_diff < eps || i >= 5 {
			log(format!("[{}] find_t_on_spline finished! t: {:.3} t_without_spline: {:.3} ratio: {:.3}", i, t, t_without_spline, t / t_without_spline));
			break;
		}

		// let correction = target_pos.z / spline_p.z;
		let correction = (90.0 / angle).clamp(0.3, 1.7);
		corrections.push(correction);
		let mut corrected_offset = init_t_delta;
		for c in corrections.iter() {
			corrected_offset *= c;
		}
		t = state.t + corrected_offset;

		log(format!("[{}] t: {:.3} correction: 90.0 / angle({:.3}) = correction({:.3})", i, t, angle, correction));

		i += 1;
	};

	if t.is_infinite() || t.is_nan() {
		panic!("find_t_on_spline: t is invalid! t: {:.3}", t);
	}

	t
}

fn spawn_tile(
	pose	: Transform,
	filter_info : Option<Herringbone2TileFilterInfo>,
	state	: &mut BrickRoadProgressState,
	config	: &Herringbone2Config,
	control	: &HerringboneControl,
	sargs	: &mut SpawnArguments,
	log		: impl Fn(String)
) {
    let (me, mut ma) = (config.mesh.clone_weak(), config.material.clone_weak());

	match filter_info {
		Some(ref fi) => {
			let left = (fi.left_border).x;
			let right =  (fi.right_border).x;
			let in_range = (left <= fi.x) && (fi.x <= right);
			if !in_range {
				if control.visual_debug {
					ma = config.material_dbg.clone_weak();
					log(format!("tile was filtered out!"));
				} else {
					return;
				}
			}
		},
		_ => (),
	}

    let tile_to_spawn = PbrBundle{ mesh: me, material: ma, ..default() };
	let mut tile_entity_id	= Entity::from_raw(0);
    sargs.commands.entity(config.root_entity).with_children(|road_root| {
		tile_entity_id = road_root.spawn_bundle(tile_to_spawn)
			.insert			(config.body_type)
			.insert			(pose)
			.insert			(GlobalTransform::default())
			.insert			(Collider::cuboid(config.hsize.x, config.hsize.y, config.hsize.z))
			.insert_bundle	(PickableBundle::default())
	//			.insert			(Draggable::default())
			.insert			(Herringbone2)
			.insert			(Tile)
			.insert			(state.clone())
			.id				()
			;
	});

	if filter_info.is_some() {
		sargs.commands.entity(tile_entity_id).insert(filter_info.unwrap());
	}
}

pub fn brick_road_iter(
		spline 			: &Spline,
	mut state			: &mut BrickRoadProgressState,
		config			: &Herringbone2Config,
		_ass			: &Res<AssetServer>,
		control			: &HerringboneControl,
		sargs			: &mut SpawnArguments,
) {
	// a little logging helper lambda
	let iter 			= state.iter;
	let log 			= |str_in : String| {
		if control.verbose {
	 		println!	("[{}] {}", iter, str_in);
		}
	};

	//
	//
	// Calculating new/next tile position that fits on tile

	// on a straight line tile position's z works as "t" (parameter for spline sampling). Later on t gets adjusted in find_t_on_spline to road limits for current tile
	// z is used explicitely here because we don't want to deal with 2 dimensions in spline sampling and offset by x will be added later
	log(format!("new brick_road_iter! t: {:.3}", state.t));

	let mut next_pos	= Vec3::ZERO;
	if !calc_next_tile_pos_on_road(&mut next_pos, state, spline, config, log) {
		state.finished = true;
		return;
	}
	let prev_pos		= state.pos;
	let tile_pos_delta 	= next_pos - prev_pos;

	//
	//
	// Putting tile on spline

	let t = find_t_on_spline(next_pos, state, spline, config, log);

	log(format!("t after spline fitting: {:.3}", t));
	log(format!("t: {:.3} next_pos:[{:.3} {:.3} {:.3}] prev_pos: [{:.3} {:.3} {:.3}] tile_pos_delta: [{:.3} {:.3} {:.3}]", t, next_pos.x, next_pos.y, next_pos.z, state.pos.x, state.pos.y, state.pos.z, tile_pos_delta.x, tile_pos_delta.y, tile_pos_delta.z));

	// in herringbone pattern every next tile is rotated +-45 degrees from 0
	let pattern_angle	= herringbone_angle(state.pattern_iter);
	let pattern_rotation = Quat::from_rotation_y(pattern_angle);

	// 
	//
	// Final pose
	let mut pose 		= Transform::identity();
	pose.translation 	= next_pos;
	pose.rotation		= pattern_rotation;

	// 
	//
	// Spawning

	let spline_p 		= spline.calc_position(t);
	let spline_r		= spline.calc_rotation_wpos(t, spline_p);

	let hwidth_rotated	= spline_r.mul_vec3(Vec3::X * (config.width / 2.0));
	let filter_info	= Herringbone2TileFilterInfo {
		x						: pose.translation.x,
		t						: t,
		left_border				: spline_p - hwidth_rotated,
		right_border			: spline_p + hwidth_rotated,
		road_halfwidth_rotated	: hwidth_rotated,
		spline_p				: spline_p
	};
	if !control.dry_run {
		spawn_tile		(pose, Some(filter_info), state, config, control, sargs, log);
	}

	//
	//
	// cheat/debug: end on certain column/row id to avoid long logs etc
	let debug			= true;
	if state.iter == 12 && debug {
		log(format!("DEBUG FULL STOP"));
		state.finished = true;
		return;
	}

	//
	//
	// Iteration ended
	state.iter			+= 1;
	state.t		 		= t;
	state.pos	 		= next_pos;

	log(format!("----------------------------"));
}