use bevy			:: { prelude :: * };
use bevy			:: { app::AppExit };
use bevy_debug_text_overlay::screen_print;
use bevy_rapier3d	:: { prelude :: * };
use bevy_fly_camera	:: { FlyCamera };
use bevy_mod_picking:: { * };
use bevy_polyline	:: { prelude :: * };
use iyes_loopless	:: { prelude :: * };

use std				:: { path::PathBuf };

use super           :: { * };
use crate			:: vehicle;
use crate			:: herringbone;
use crate			:: bevy_spline;

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
		let config = herringbone::Herringbone2Config::default();
		let mut sargs = SpawnArguments {
			meshes : &mut meshes,
			materials : &mut materials,
			commands : &mut commands,
		};
		herringbone::spawn::brick_road(&transform, &config, false, &mut polylines, &mut polyline_materials, &mut sargs);

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

		vehicle::spawn(
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

		// enable bare minimum first to run first update (TODO: probably do this in plugin instead)
		camera.enabled_follow = false;
		camera.enabled_translation = true;
		camera.enabled_rotation = true;
	}
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
	mut q_camera	: Query<&mut FlyCamera>,
		game		: Res<GameState>,
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

	// #[cfg(debug_assertions)]
	if time.seconds_since_startup() > 1.0 {
		let is_editor = game_mode.0 == GameMode::Editor;
		set_cursor_visibility(is_editor);

		let mut camera 	= q_camera.get_mut(game.camera.unwrap()).unwrap();
		camera.enabled_rotation = !is_editor;
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
	mut q_control	: Query<(Entity, &Children, &mut bevy_spline::SplineControl, &mut herringbone::HerringboneControl)>,
		q_selection	: Query<&Selection>,
		q_children	: Query<&Children>,
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

	for (control_e, children, mut spline_ctl, mut tile_ctl) in q_control.iter_mut() {
		let selection = q_selection.get(control_e).unwrap();
		let mut selection_found = selection.selected();
		if !selection_found {
			selection_found = check_selection_recursive(children, &q_children, &q_selection, 0, 2);

			if !selection_found {
				continue;
			}
		}

		if key.pressed(KeyCode::LControl) && key.pressed(KeyCode::T) {
			tile_ctl.next 	= true;
			screen_print!	("Tile Control Spawn");
		}

		if key.just_released(KeyCode::T) && !key.pressed(KeyCode::RControl) && !key.pressed(KeyCode::RShift) {
			tile_ctl.next 	= false;
			screen_print!	("Tile Control Spawn Stopped");
		}

		if key.pressed(KeyCode::LAlt) && key.just_pressed(KeyCode::T) {
			tile_ctl.reset	= true;
			screen_print!	("Tile Control Reset");
		}

		if key.pressed(KeyCode::RControl) && !key.pressed(KeyCode::RShift) && key.just_pressed(KeyCode::T) {
			tile_ctl.animate = true;
			tile_ctl.last_update = time.seconds_since_startup();
			screen_print!	("Tile Control Animate");
		}

		if key.pressed(KeyCode::RControl) && key.pressed(KeyCode::RShift) && key.just_pressed(KeyCode::T) {
			tile_ctl.instant = true;
			tile_ctl.next = true;
			tile_ctl.last_update = time.seconds_since_startup();
			screen_print!	("Tile Control Spawn Instantly");
		}

		if btn.just_pressed(MouseButton::Right) {
			spline_ctl.new_point = true;
		}

		if key.just_pressed(KeyCode::Q) {
			tile_ctl.debug += 1;
			if tile_ctl.debug > 2 {
				tile_ctl.debug = 0;
			}
			screen_print!("Tile Control Debug: {}", tile_ctl.debug);
			tile_ctl.reset	= true;
			tile_ctl.instant = true;
			tile_ctl.next 	= true;
		}

		if key.pressed(KeyCode::LControl) && key.pressed(KeyCode::LAlt) && key.just_pressed(KeyCode::V) {
			tile_ctl.verbose = !tile_ctl.verbose;
			screen_print!("Tile Control Verbose: {}", tile_ctl.verbose);
		}

		if key.pressed(KeyCode::LControl) && key.pressed(KeyCode::LAlt) && key.just_pressed(KeyCode::D) {
			tile_ctl.dry_run = !tile_ctl.dry_run;
			screen_print!("Tile Control Dry Run: {}", tile_ctl.dry_run);
		}

		if key.pressed(KeyCode::LControl) && key.pressed(KeyCode::LAlt) && key.just_pressed(KeyCode::L) {
			tile_ctl.looped = !tile_ctl.looped;
			screen_print!("Tile Control Looped: {}", tile_ctl.looped);
		}
	}
}

fn check_selection_recursive(
	children	: &Children,
	q_children	: &Query<&Children>,
	q_selection : &Query<&Selection>,
	depth		: u32,
	max_depth 	: u32
 ) -> bool {
	let mut selection_found = false;
	for child in children.iter() {
		let selection = match q_selection.get(*child) {
			Ok(s) => s,
			Err(_) => continue,
		};

		if selection.selected() {
			selection_found = true;
		} else {
			if depth >= max_depth {
				continue;
			}
			let subchildren = q_children.get(*child);
			if subchildren.is_ok() {
				selection_found = check_selection_recursive(subchildren.unwrap(), q_children, q_selection, depth + 1, max_depth);
			}
		}

		if selection_found {
			break;
		}
	}

	selection_found
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