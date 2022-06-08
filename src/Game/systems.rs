use bevy			:: { prelude :: * };
use bevy			:: { app::AppExit };
use bevy_rapier3d	:: { prelude :: * };
use bevy_fly_camera	:: { FlyCamera };
use bevy_mod_picking:: { * };
use bevy_polyline	:: { prelude :: * };
use iyes_loopless	:: { prelude :: * };

use std				:: { path::PathBuf };

use super           :: { * };
use crate			:: Vehicle;
use crate			:: Herringbone;

pub fn setup_camera_system(
		game			: ResMut<GameState>,
	mut query			: Query<&mut FlyCamera>
) {
	// initialize camera with target to look at
	if game.camera.is_some() {
		let mut camera 	= query.get_mut(game.camera.unwrap()).unwrap();
		if game.body.is_some() {
			camera.target = Some(game.body.unwrap().entity);
			println!	("camera.target Entity ID {:?}", camera.target);
		}

		// temp
		camera.enabled_follow = false;
		camera.enabled_translation = true;
		camera.enabled_rotation = true;
	}
}

#[derive(Component)]
pub struct RedLine;

#[derive(Component)]
pub struct GreenLine;

#[derive(Component)]
pub struct BlueLine;

pub fn setup_world_system(
	mut _configuration	: ResMut<RapierConfiguration>,
	mut	phys_ctx		: ResMut<DebugRenderContext>,
	mut game			: ResMut<GameState>,
	mut	meshes			: ResMut<Assets<Mesh>>,
	mut	materials		: ResMut<Assets<StandardMaterial>>,
		ass				: Res<AssetServer>,
	mut commands		: Commands,

	mut polylines		: ResMut<Assets<Polyline>>,
	mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
) {
//	configuration.timestep_mode = TimestepMode::VariableTimestep;
	phys_ctx.enabled	= false;

	spawn::camera		(&mut game, &mut commands);

	spawn::ground		(&game, &mut meshes, &mut materials, &mut commands);

	spawn::world_axis	(&mut meshes, &mut materials, &mut commands);

	if false {
		spawn::cubes	(&mut commands);
	}

	if false {
		spawn::friction_tests(&mut meshes, &mut materials, &mut commands);
	}

	if false {
		spawn::obstacles(&mut meshes, &mut materials, &mut commands);
	}

	if false {
		spawn::spheres	(&mut meshes, &mut materials, &mut commands);
	}

	if false {
		spawn::wall		(&mut meshes, &mut materials, &mut commands);
	}

	if true {
		let y = 1.5;
		let transform = Transform::from_translation(Vec3::new(0.0, y, 0.0));
		let config = Herringbone::Herringbone2Config::default();
		let mut sargs = SpawnArguments {
			meshes : &mut meshes,
			materials : &mut materials,
			commands : &mut commands,
		};
		Herringbone::spawn::brick_road(transform, &config, false, &mut polylines, &mut polyline_materials, &mut sargs);

		// let transform = Transform::from_translation(Vec3::new(0.0, y, 0.0));
		// Herringbone::spawn::brick_road(transform, &config, true, &mut polylines, &mut polyline_materials, &mut sargs);
	}

	// polyline
	if true {
		commands.spawn_bundle(PolylineBundle {
			polyline: polylines.add(Polyline {
				vertices: vec![-Vec3::Z, Vec3::Z],
				..default()
			}),
			material: polyline_materials.add(PolylineMaterial {
				width: 100.0,
				color: Color::RED,
				perspective: true,
				..default()
			}),
			..default()
		})
		.insert(RedLine);
		
		commands.spawn_bundle(PolylineBundle {
			polyline: polylines.add(Polyline {
				vertices: vec![-Vec3::Z, Vec3::Z],
				..default()
			}),
			material: polyline_materials.add(PolylineMaterial {
				width: 100.0,
				color: Color::SEA_GREEN,
				perspective: true,
				..default()
			}),
			..default()
		})
		.insert(GreenLine);
	
		commands.spawn_bundle(PolylineBundle {
			polyline: polylines.add(Polyline {
				vertices: vec![-Vec3::Z, Vec3::Z],
				..default()
			}),
			material: polyline_materials.add(PolylineMaterial {
				width: 100.0,
				color: Color::MIDNIGHT_BLUE,
				perspective: true,
				..default()
			}),
			..default()
		})
		.insert(BlueLine);
	}

	if false {
		let veh_file	= Some(PathBuf::from("corvette.ron"));
		let veh_cfg		= load_vehicle_config(&veh_file).unwrap();

		let body_pos 	= Transform::from_xyz(0.0, 5.5, 0.0);

		Vehicle::spawn(
			  &veh_cfg
			, body_pos
			, &mut game
			, &ass
			, &mut commands
		);
	}
}

pub fn setup_lighting_system(
	mut commands				: Commands,
) {
	const HALF_SIZE: f32		= 100.0;

	commands.spawn_bundle(DirectionalLightBundle {
		directional_light: DirectionalLight {
			illuminance: 8000.0,
			// Configure the projection to better fit the scene
			shadow_projection	: OrthographicProjection {
				left			: -HALF_SIZE,
				right			:  HALF_SIZE,
				bottom			: -HALF_SIZE,
				top				:  HALF_SIZE,
				near			: -10.0 * HALF_SIZE,
				far				: 100.0 * HALF_SIZE,
				..default()
			},
			shadows_enabled		: true,
			..default()
		},
		transform				: Transform {
			translation			: Vec3::new(10.0, 2.0, 10.0),
			rotation			: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
			..default()
		},
		..default()
	});

	// commands
	//     .spawn_bundle(DirectionalLightBundle {
	//         ..Default::default()
	//     })
	//     .insert(Sun); // Marks the light as Sun

	//
}

pub fn setup_cursor_visibility_system(
	mut windows	: ResMut<Windows>,
	mut picking	: ResMut<PickingPluginsState>,
) {
	let window = windows.get_primary_mut().unwrap();

	window.set_cursor_lock_mode	(true);
	window.set_cursor_visibility(false);

	picking.enable_picking 		= false;
	picking.enable_highlighting = false;
	picking.enable_interacting 	= false;
}

pub fn cursor_visibility_system(
	mut windows		: ResMut<Windows>,
	btn				: Res<Input<MouseButton>>,
	key				: Res<Input<KeyCode>>,
	time			: Res<Time>,
	mut window_focused : EventReader<bevy::window::WindowFocused>,
		game_mode	: Res<CurrentState<GameMode>>,
	mut picking		: ResMut<PickingPluginsState>,
	mut	commands	: Commands
) {
	let window 		= windows.get_primary_mut().unwrap();
	let cursor_visible = window.cursor_visible();
	let window_id	= window.id();

	let mut set_cursor_visibility = |v| {
		window.set_cursor_visibility(v);
		window.set_cursor_lock_mode(!v);
	};

	let mut set_visibility = |v| {
		set_cursor_visibility(v);

		picking.enable_picking = v;
		picking.enable_highlighting = v;
		picking.enable_interacting = v;

		commands.insert_resource(NextState(
			if v { GameMode::Editor } else { GameMode::InGame }
		));
	};

	if key.just_pressed(KeyCode::Escape) {
		let toggle 	= !cursor_visible;
		set_visibility(toggle);
	}

	if btn.just_pressed(MouseButton::Left) && game_mode.0 == GameMode::InGame{
		set_cursor_visibility(false);
	}

	if time.seconds_since_startup() > 1.0 {
		for ev in window_focused.iter() {
			if ev.id == window_id {

				if !ev.focused {
					set_cursor_visibility(true);
				} else {
					// this works bad because winit says we cant grab cursor right after window gets alt-tabbed back to focused
					set_cursor_visibility(game_mode.0 == GameMode::Editor);
				}
			}
		}
	}
}

pub fn input_misc_system(
		btn			: Res<Input<MouseButton>>,
		key			: Res<Input<KeyCode>>,
		_game		: Res<GameState>,
		time		: Res<Time>,
	mut	phys_ctx	: ResMut<DebugRenderContext>,
	mut exit		: EventWriter<AppExit>,
	mut q_camera	: Query<&mut FlyCamera>,
	mut q_control	: Query<(Entity, &mut Herringbone::Control, &Children)>,
		q_selection	: Query<&Selection>,
) {
	for mut camera in q_camera.iter_mut() {
		if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Space) {
			let toggle 	= !camera.enabled_follow;
			camera.enabled_follow = toggle;
		}

		if key.just_pressed(KeyCode::Escape) {
			let toggle 	= !camera.enabled_rotation;
			camera.enabled_rotation = toggle;
		}
	}

	if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Escape) {
		exit.send(AppExit);
	}

	if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Key1) {
		phys_ctx.enabled = !phys_ctx.enabled;
	}
	
	for (control_e, mut control, children) in q_control.iter_mut() {
		let selection = q_selection.get(control_e).unwrap();
		let mut selection_found = selection.selected();
		if !selection_found {
			for child in children.iter() {
				let selection = match q_selection.get(*child) {
					Ok(s) => s,
					Err(_) => continue,
				};

				if selection.selected() {
					selection_found = true;
					break;
				}
			}

			if !selection_found {
				continue;
			}
		}

		if key.pressed(KeyCode::LControl) && key.pressed(KeyCode::T) {
			control.next 	= true;
		}

		if key.just_released(KeyCode::T) {
			control.next 	= false;
		}

		if key.pressed(KeyCode::LAlt) && key.just_pressed(KeyCode::T) {
			control.reset	= true;
		}

		if key.pressed(KeyCode::RControl) && !key.pressed(KeyCode::RShift) && key.just_pressed(KeyCode::T) {
			control.animate	= true;
			control.last_update = time.seconds_since_startup();
		}

		if key.pressed(KeyCode::RControl) && key.pressed(KeyCode::RShift) && key.just_pressed(KeyCode::T) {
			control.instant	= true;
			control.next 	= true;
			control.last_update = time.seconds_since_startup();
		}

		if btn.just_pressed(MouseButton::Right) {
			control.new_spline_point = true;
			control.reset	= true;
			control.instant	= true;
			control.next 	= true;
		}

		if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Q) {
			control.debug 	= !control.debug;
			control.reset	= true;
			control.instant	= true;
			control.next 	= true;
		}
	}
}

pub fn despawn_system(mut commands: Commands, time: Res<Time>, mut despawn: ResMut<DespawnResource>) {
	if time.seconds_since_startup() > 0.1 {
		for entity in &despawn.entities {
//			println!("Despawning entity {:?}", entity);
			commands.entity(*entity).despawn_recursive();
		}
		despawn.entities.clear();
	}
}

// Convert engine Transform of an entity to spline tangent Vec3. Spline tangents are in the same space as control points.
// In engine spline tangent handles(as in entities with transforms) are children of control point entities so we have to juggle between spline space and tangent space
pub fn on_spline_tangent_moved(
		time			: Res<Time>,
		key				: Res<Input<KeyCode>>,
	mut	polylines		: ResMut<Assets<Polyline>>,
		q_polyline		: Query<&Handle<Polyline>>,
	 	q_control_point	: Query<(&Parent, &Children, &Transform), With<SplineControlPoint>>,
		mut set: ParamSet<(
			Query<(&Parent, Entity, &Transform, &SplineTangent), (Changed<Transform>, Without<SplineControlPoint>)>,
			Query<(&mut Transform), (With<SplineTangent>, (Without<DraggableActive>, Without<SplineControlPoint>))>
		)>,
	mut	q_tangent_opp	: Query<Entity, Without<DraggableActive>>,
	mut q_spline		: Query<&mut Spline>
) {
	if time.seconds_since_startup() < 0.1 {
		return;
	}

	if q_spline.is_empty() {
		return;
	}

	let sync_tangents	= key.pressed(KeyCode::LControl);

	let mut cpc : Vec<Entity> = Vec::new();
	let mut tane = Entity::from_raw(0);
	let mut tan11 = Vec3::ZERO;
	let mut cptrans = Vec3::ZERO;

	for (control_point_e, tan_e, tan_tform, tan) in set.p0().iter() {//q_tangent.iter() {
		let (spline_e, control_point_children_e, control_point_tform) = q_control_point.get(control_point_e.0).unwrap();
		let mut spline	= q_spline.get_mut(spline_e.0).unwrap();

		// in spline space (or parent space for tangent handles). _p == parent space
		let tan_tform_p	= (*control_point_tform) * (*tan_tform);
		let tan_pos_p	= tan_tform_p.translation;

		let (tan0, tan1) =
		if !sync_tangents {
			let prev_interpolation = spline.get_interpolation(tan.global_id);
			let opposite_tan_pos_p = match prev_interpolation {
				SplineInterpolation::StrokeBezier(V0, V1) => {
					if tan.local_id == 0 { *V1 } else { *V0 }
				},
				_ => panic!("unsupported interpolation type!"),
			};

			let tan0 = if tan.local_id == 0 { tan_pos_p } else { opposite_tan_pos_p };
			let tan1 = if tan.local_id == 1 { tan_pos_p } else { opposite_tan_pos_p };

			(tan0, tan1)
		} else {
			let opposite_tan_tform = tan_tform.clone();
			let mat_inv = opposite_tan_tform.compute_matrix().inverse();
			let opposite_tan_tform_p = (*control_point_tform) * Transform::from_matrix(mat_inv);
			let opposite_tan_pos_p = opposite_tan_tform_p.translation;

			(tan_pos_p, opposite_tan_pos_p)
		};

		spline.set_interpolation(tan.global_id, SplineInterpolation::StrokeBezier(tan0, tan1));

		tane = tan_e;
		tan11 = tan1;
		cptrans = control_point_tform.translation;

		for child_e_ref in control_point_children_e.iter() {
			let child_e = *child_e_ref;
			cpc.push(child_e);
			// if let Ok(mut tform) = set.p1().get_mut(child_e) {
			// 	if child_e != tan_e && sync_tangents {
					
			// 		//tform.translation = tan1 - control_point_tform.translation;
			// 		// commands.entity(child_e).insert(ToRecalculateOpposite);
			// 		// screen_print!("inserting opp {:?} tan_e {:?}", child_e,  tan_e);
			// 		// println!("inserting opp {:?} tan_e {:?} ", child_e, tan_e);
			// 	}
			// } else {
			// 	screen_print!("shitty basket!");
			// 	println!("shitty basket!");
			// }
			// if q_tangent_opp.get(child_e).is_ok() {
			// 	if child_e != tan_e {
			// 		commands.entity(child_e).insert(ToRecalculateOpposite);
			// 		screen_print!("inserting opp {:?} tan_e {:?}", child_e,  tan_e);
			// 		println!("inserting opp {:?} tan_e {:?} ", child_e, tan_e);
			// 	}
			// } else {
			// 	screen_print!("shitty basket!");
			// 	println!("shitty basket!");
			// }

			if let Ok(handle) = q_polyline.get(child_e) {
				let line	= polylines.get_mut(handle).unwrap();
				line.vertices.resize(3, Vec3::ZERO);
				let tan0 	= tan0 - control_point_tform.translation;
				line.vertices[0] = tan0;
				let tan1 	= tan1 - control_point_tform.translation;
				line.vertices[2] = tan1;

				line.vertices[1] = Vec3::ZERO;
			}
		}
	}

	for c in cpc {
		if let Ok(mut tform) = set.p1().get_mut(c) {
			if c != tane && sync_tangents {
				tform.translation = tan11 - cptrans;
			}
		}
	}
}

pub fn on_spline_control_point_moved(
		time			: Res<Time>,
		q_controlp 		: Query<(&Parent, &Children, &Transform, &SplineControlPoint), Changed<Transform>>,
		q_tangent 		: Query<(&Transform, &SplineTangent)>,
	mut q_spline		: Query<&mut Spline>,
) {
	if time.seconds_since_startup() < 0.1 {
		return;
	}

	if q_spline.is_empty() {
		return;
	}

	for (spline_e, children_e, control_point_tform, controlp) in q_controlp.iter() {
		let mut spline = q_spline.get_mut(spline_e.0).unwrap();

		let controlp_pos = control_point_tform.translation;
		match controlp {
			SplineControlPoint::ID(id_ref) => {
				let id = *id_ref;
				spline.set_control_point(id, controlp_pos);

				// we have to recalculate tangent positions because in engine they are parents of control point
				// but spline wants them in the same space as control points
				let mut tan0 = Vec3::ZERO;
				let mut tan1 = Vec3::ZERO;
				for tangent_e in children_e.iter() {
					let (tan_tform, tan) = match q_tangent.get(*tangent_e) {
						Ok((tf, tn)) => (tf, tn),
						Err(_) => { continue },
					};
					let final_tform = (*control_point_tform) * (*tan_tform);
					if tan.local_id == 0 {
						tan0 = final_tform.translation;
					} else if tan.local_id == 1 {
						tan1 = final_tform.translation;
					}
				}
				
				spline.set_interpolation(id, SplineInterpolation::StrokeBezier(tan0, tan1));
			},
		}
	}
}

pub fn on_root_handle_moved(
	time : Res<Time>,
) {
	if time.seconds_since_startup() < 0.1 {
		return;
	}
}