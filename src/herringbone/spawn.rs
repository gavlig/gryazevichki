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

fn calc_max_distance_to_spline(config: &Herringbone2Config) -> f32 {
	(config.width / 2.0) + config.hsize.x
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

	log(format!("calc_next_tile_pos_zplus: [{:.3} {:.3} {:.3}]", herrpos.x, herrpos.y, herrpos.z));
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

	log(format!("calc_next_tile_pos_zminus: [{:.3} {:.3} {:.3}]", herrpos.x, herrpos.y, herrpos.z));
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

	let offset0_length 	= hlenx + seam + hlenx;
	let offset1_length 	= hlenz + seam;
	let (offset0_rotation, offset1_rotation) = 
	if pattern_iter % 2 == 0 {
		(FRAC_PI_4, -FRAC_PI_4)
	} else {
		(-FRAC_PI_4, FRAC_PI_4)
	};
	let offset0 = Quat::from_rotation_y(offset0_rotation).mul_vec3(Vec3::Z * -offset0_length);
	let offset1 = Quat::from_rotation_y(offset1_rotation).mul_vec3(Vec3::Z * offset1_length);
	let herrpos = prev_p + offset0 + offset1;

	log(format!("calc_next_tile_pos_xplus: [{:.3} {:.3} {:.3}]", herrpos.x, herrpos.y, herrpos.z));
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

	log(format!("calc_next_tile_pos_xminus: [{:.3} {:.3} {:.3}]", herrpos.x, herrpos.y, herrpos.z));
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
	let mut t			= state.t;
	let mut dir_cnt		= 0;
	let init_dir		= state.dir;
	let init_pattern_iter = state.pattern_iter;

	while !next_pos_found && dir_cnt < 4 {
		if state.dir.is_vertical() && init_dir.is_horizontal() {
			state.pattern_iter = if init_pattern_iter == 0 { 1 } else { 0 };
			log(format!("[{}] [hor -> ver]: next pattern_iter: {} init_dir: {:?} dir: {:?}", dir_cnt, state.pattern_iter, init_dir, state.dir));
		}
		next_pos		= calc_next_tile_pos(&mut state, config, &log);

		log(format!("[{}] checking dir {:?}, next pos: [{:.3} {:.3} {:.3}] prev pos: [{:.3} {:.3} {:.3}] pattern_iter: {:?}", dir_cnt, state.dir, next_pos.x, next_pos.y, next_pos.z, state.pos.x, state.pos.y, state.pos.z, state.pattern_iter));

		t				= find_t_on_spline(next_pos, state.pos, state.t, spline, &log);
		let spline_p	= spline.calc_position(t);

		let distance_to_spline = (next_pos - spline_p).length();
		log(format!("[{}] spline_p: [{:.3} {:.3} {:.3}] distance_to_spline: {:.3}", dir_cnt, spline_p.x, spline_p.y, spline_p.z, distance_to_spline));

		let too_far_from_spline = distance_to_spline > calc_max_distance_to_spline(config);
		if too_far_from_spline {
			if Direction2D::Up == state.dir {
				state.pos = next_pos;
				state.t = t;
				log(format!("[{}] couldn't go up but switched to next row since previous row ended. new t: {:.3}", dir_cnt, state.t));
			}

			state.set_next_direction(init_dir);
			log(format!("[{}] new pos is too far from border! switching direction to {:?}", dir_cnt, state.dir));
		} else {
			next_pos_found = true;
			log(format!("[{}] all good! dir {:?} distance_to_spline: {:.3}", dir_cnt, state.dir, distance_to_spline));
		} 

		dir_cnt 		+= 1;
	}

	if !next_pos_found {
		log(format!("end condition met! last iter: {} last pos: [{:.3} {:.3} {:.3}]", state.iter, state.pos.x, state.pos.y, state.pos.z));
		log(format!("----------------------------"));
	} else {
		// we went up one iteration, let's fill another row of tiles on the road! (switching from up to left)
		if state.dir == Direction2D::Up {
			state.set_next_direction(init_dir);
			state.t		= t;
			log(format!("new t: {:.3} We went up one iteration, let's fill another row of tiles on the road! (switching from up to left)", state.t));
		}

		*state_out		= state;
		*next_pos_out	= next_pos;
	}
	
	return next_pos_found;
}

fn find_t_on_spline(
	tile_pos		: Vec3,
	prev_pos		: Vec3,
	t_prev			: f32,
	spline			: &Spline,
	log				: impl Fn(String)
) -> f32 {
	// if t_prev <= 0.0 || t_prev >= spline.total_length() {
	// 	log(format!("find_t_on_spline early out! t_prev: {:.3} spline.total_length: {:.3}", t_prev, spline.total_length()));
	// 	return		t_prev;
	// }

	let init_t_delta = (tile_pos - prev_pos).length();
	let spline_p_prev = spline.calc_position(t_prev);

	let mut step	= init_t_delta;
	let mut t 		= t_prev + step;

	let mut t_best	= t_prev;
	let mut distance_to_spline_best = (tile_pos - spline_p_prev).length();
	let mut distance_to_spline_prev = distance_to_spline_best.clone();
	let mut i 		= 0;
	let eps			= 0.001; // precision is 1mm

	log(format!("find_t_on_spline started! t_without_spline: {:.3} ", t_prev));

	loop {
		let spline_p = spline.calc_position(t);
		log(format!("[{}] t: {:.3} spline_pos: [{:.3} {:.3} {:.3}] target_pos: [{:.3} {:.3} {:.3}]", i, t, spline_p.x, spline_p.y, spline_p.z, tile_pos.x, tile_pos.y, tile_pos.z));

		if spline_p.abs_diff_eq(tile_pos, eps) {
			log(format!("find_t_on_spline finished(spline_p == target_pos)! t: {:.3} t_without_spline: {:.3} ratio: {:.3}", t, t_prev, t / t_prev));
			break;
		}

		let delta_pos = tile_pos - spline_p;
		let distance_to_spline = delta_pos.length();

		log(format!("[{}] distance_to_spline: {:.3}", i, distance_to_spline));

		let distance_diff = distance_to_spline - distance_to_spline_prev;
		if distance_diff.abs() < eps || i >= 4 {
			log(format!("find_t_on_spline finished! t: {:.3} t_without_spline: {:.3} ratio: {:.3}", t, t_prev, t / t_prev));
			break;
		}
		distance_to_spline_prev = distance_to_spline;

		if i != 0 || distance_diff < 0.0 {
			step *= 0.5;
		}

		if distance_diff > 0.0 {
			step = -step;
		}

		if distance_to_spline < distance_to_spline_best {
			t_best = t;
			distance_to_spline_best = distance_to_spline;
			log(format!("new t_best! {:.3} distance_to_spline_best: {:.3}", t_best, distance_to_spline_best));
		}

		t += step;

		log(format!("[{}] step: {:.3}", i, step));

		i += 1;
	};

	if t_best.is_infinite() || t_best.is_nan() {
		panic!("find_t_on_spline: t_best is invalid! t_best: {:.3}", t_best);
	}

	t_best
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
			let left = fi.left_border;
			let right =  fi.right_border;
			let in_range = ((left.x <= fi.pos.x) && (fi.pos.x <= right.x)) || ((right.x <= fi.pos.x) && (fi.pos.x <= left.x))
						|| ((left.z <= fi.pos.z) && (fi.pos.z <= right.z)) || ((right.z <= fi.pos.z) && (fi.pos.z <= left.z));
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
	// Find closes point on spline to figure out borders

	let t = find_t_on_spline(next_pos, state.pos, state.t, spline, log);

	log(format!("t after spline fitting: {:.3}", t));
	log(format!("t: {:.3} next_pos:[{:.3} {:.3} {:.3}] prev_pos: [{:.3} {:.3} {:.3}] tile_pos_delta: [{:.3} {:.3} {:.3}]", t, next_pos.x, next_pos.y, next_pos.z, state.pos.x, state.pos.y, state.pos.z, tile_pos_delta.x, tile_pos_delta.y, tile_pos_delta.z));

	// in herringbone pattern every next tile is rotated +-45 degrees from 0
	let pattern_angle	= herringbone_angle(state.pattern_iter);
	let pattern_rotation = Quat::from_rotation_y(pattern_angle);

	// 
	//
	// Final pose
	let mut tile_pose 	= Transform::identity();
	tile_pose.translation = next_pos;
	tile_pose.rotation	= pattern_rotation;

	// 
	//
	// Filtering + Spawning

	let spline_p 		= spline.calc_position(t);
	let spline_r		= spline.calc_rotation_wpos(t, spline_p);

	let hwidth_rotated	= spline_r.mul_vec3(Vec3::X * calc_max_distance_to_spline(config));

	let tile2spline		= tile_pose.translation - spline_p;
	let tile_pos_rotated =
	if tile2spline.length() > 0.001 {
		let tile_azimuth	= Quat::from_rotation_arc(Vec3::Z, tile2spline.normalize());
		spline_p + (tile_azimuth).mul_vec3(Vec3::Z * tile2spline.length())
	} else {
		tile_pose.translation
	};

	let filter_info	= Herringbone2TileFilterInfo {
		pos						: tile_pos_rotated,
		t						: t,
		left_border				: spline_p - hwidth_rotated,
		right_border			: spline_p + hwidth_rotated,
		road_halfwidth_rotated	: hwidth_rotated,
		spline_p				: spline_p
	};
	if !control.dry_run {
		spawn_tile		(tile_pose, Some(filter_info), state, config, control, sargs, log);
	}

	//
	//
	// cheat/debug: end on certain column/row id to avoid long logs etc
	let debug			= false;
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