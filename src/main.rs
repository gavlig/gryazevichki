use bevy			::	prelude :: { * };
use bevy_rapier3d	::	prelude :: { * };
use bevy_fly_camera	::	{ FlyCameraPlugin };
use bevy_egui		::	{ * };
use bevy_atmosphere	::	{ * };
use bevy_mod_picking::	{ * };
use bevy_polyline	::	{ * };
use bevy_prototype_debug_lines	:: { * };
use bevy_debug_text_overlay		:: { screen_print, OverlayPlugin };
// use bevy_transform_gizmo :: { * }; 

use iyes_loopless	::	prelude :: { * };

#[macro_use(defer)] extern crate scopeguard;

mod Game;
use Game			:: 	{ *, Vehicle::* };

fn main() {
	App::new()
		.insert_resource(ClearColor(Color::rgb(
			0xF9 as f32 / 255.0,
			0xF9 as f32 / 255.0,
			0xFF as f32 / 255.0,
		)))
		.insert_resource		(Msaa				::default())
		.insert_resource		(GameState			::default())
		.insert_resource		(DespawnResource	::default())
		.insert_resource		(AtmosphereMat		::default()) // Default Earth sky

		.insert_resource		(HerringboneIO		::default())
		.insert_resource		(HerringboneStepRequest::default())

		.add_loopless_state		(GameMode::InGame)

		.add_plugins			(DefaultPlugins)
		.add_plugin				(RapierPhysicsPlugin::<NoUserData>::default())
		.add_plugin				(RapierDebugRenderPlugin::default())
		.add_plugin				(FlyCameraPlugin)
		.add_plugin				(EguiPlugin)
		.add_plugins			(DefaultPickingPlugins)

		.add_plugin				(PolylinePlugin)
		.add_plugin				(DebugLinesPlugin::default())
		.add_plugin				(OverlayPlugin { font_size: 32.0, ..default() })
		// .add_plugin				(TransformGizmoPlugin::default())
		// it glitches and hides the ground
		// .add_plugin				(AtmospherePlugin {
		// 	dynamic				: true,  // Set to false since we aren't changing the sky's appearance
		//     sky_radius			: 10.0,
		// })

		.add_startup_system		(setup_cursor_visibility_system)
 		.add_startup_system		(setup_lighting_system)
 		.add_startup_system		(setup_world_system)
 		.add_startup_system_to_stage(StartupStage::PostStartup, setup_camera_system)

		// input
 		.add_system				(cursor_visibility_system)
 		.add_system				(input_misc_system)
 		.add_system				(vehicle_controls_system) // TODO: probably split input processing from gamelogic here
		// game logic 
		.add_system				(herringbone_brick_road_system)
		// ui
		.add_system				(coords_on_hover_ui_system.run_in_state(GameMode::Editor))
		.add_system				(vehicle_params_ui_system.run_in_state(GameMode::Editor))

// 		.add_system				(daylight_cycle)

 		.add_system_to_stage	(CoreStage::Last, save_vehicle_config_system)
 		.add_system_to_stage	(CoreStage::Last, load_vehicle_config_system)

		.add_system_to_stage	(CoreStage::PostUpdate, mouse_dragging_start_system)
		.add_system_to_stage	(CoreStage::PostUpdate, mouse_dragging_system)
		.add_system_to_stage	(CoreStage::PostUpdate, mouse_dragging_stop_system)

		.add_system_to_stage	(CoreStage::PostUpdate, on_spline_tangent_moved)
		.add_system_to_stage	(CoreStage::PostUpdate, on_spline_control_point_moved)
		.add_system_to_stage	(CoreStage::PostUpdate, on_object_root_moved)

 		.add_system_to_stage	(CoreStage::PostUpdate, display_events_system)
 		.add_system_to_stage	(CoreStage::PostUpdate, respawn_vehicle_system)
 		.add_system_to_stage	(CoreStage::PostUpdate, despawn_system)
		.run					();
}

// Marker for updating the position of the light, not needed unless we have multiple lights
#[derive(Component)]
struct Sun;

// We can edit the SkyMaterial resource and it will be updated automatically, as long as AtmospherePlugin.dynamic is true
fn _daylight_cycle(
	mut sky_mat: ResMut<AtmosphereMat>,
	mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
	time: Res<Time>,
) {
	let mut pos = sky_mat.sun_position;
	let t = time.time_since_startup().as_millis() as f32 / 20000.0;
	pos.y = t.sin();
	pos.z = t.cos();
	sky_mat.sun_position = pos;

	if let Some((mut light_trans, mut directional)) = query.single_mut().into() {
		light_trans.rotation = Quat::from_rotation_x(-pos.y.atan2(pos.z));
		directional.illuminance = t.sin().max(0.0).powf(2.0) * 100000.0;
	}
}

fn _spawn_gltf(
	mut commands: Commands,
	ass: Res<AssetServer>,
) {
	// note that we have to include the `Scene0` label
	let my_gltf = ass.load("corvette/wheel/corvette_wheel.gltf#Scene0");

	// to be able to position our 3d model:
	// spawn a parent entity with a TransformBundle
	// and spawn our gltf as a scene under it
	commands.spawn_bundle(TransformBundle {
		local: Transform::from_xyz(0.0, 0.0, 0.0),
		global: GlobalTransform::identity(),
	}).with_children(|parent| {
		parent.spawn_scene(my_gltf);
	});
}

fn _screen_print_text(time: Res<Time>) {
    let current_time = time.seconds_since_startup();
    let at_interval = |t: f64| current_time % t < time.delta_seconds_f64();
    let x = (13, 3.4, vec![1,2,3,4,5,6,7,8]);
    if at_interval(0.1) {
        let last_fps = 1.0 / time.delta_seconds();
        screen_print!(col: Color::CYAN, "fps: {last_fps:.0}");
        screen_print!("current time: {current_time:.2}")
    }
    if at_interval(2.0) {
        let col = Color::FUCHSIA;
        screen_print!(sec: 0.5, col: col, "every two seconds: {}, {:?}", x.0, x.2)
    }
    if at_interval(5.0) {
        screen_print!(sec: 3.0, "every five seconds: {x:#?}");
    }
}
