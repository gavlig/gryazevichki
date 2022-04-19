use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
//use bevy_editor_pls::prelude::*;

fn main() {
	App::new()
		.insert_resource(ClearColor(Color::rgb(
			0xF9 as f32 / 255.0,
			0xF9 as f32 / 255.0,
			0xFF as f32 / 255.0,
		)))
		.insert_resource(Msaa::default())
		.add_plugins(DefaultPlugins)
		.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
		.add_plugin(RapierRenderPlugin)
		.add_plugin(FlyCameraPlugin)
//		.add_plugin(EditorPlugin)
		.add_startup_system(setup_graphics.system())
		.add_startup_system(setup_physics.system())
		.add_startup_system(setup_grab_system)
		.add_system(cursor_grab_system)
		.add_system(toggle_button_system)
		.add_system(camera_collision_system)
		.run();
}

fn setup_grab_system(
	mut windows: ResMut<Windows>,
) {
	let window = windows.get_primary_mut().unwrap();

	window.set_cursor_lock_mode(true);
	window.set_cursor_visibility(false);
}

fn setup_graphics(mut commands: Commands) {
	const HALF_SIZE: f32 = 100.0;

	commands.spawn_bundle(DirectionalLightBundle {
		directional_light: DirectionalLight {
			illuminance: 10000.0,
			// Configure the projection to better fit the scene
			shadow_projection: OrthographicProjection {
				left: -HALF_SIZE,
				right: HALF_SIZE,
				bottom: -HALF_SIZE,
				top: HALF_SIZE,
				near: -10.0 * HALF_SIZE,
				far: 100.0 * HALF_SIZE,
				..Default::default()
			},
			shadows_enabled: true,
			..Default::default()
		},
		transform: Transform {
			translation: Vec3::new(10.0, 2.0, 10.0),
			rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
			..Default::default()
		},
		..Default::default()
	});

	// camera

	let camera_collider = ColliderBundle {
		shape: ColliderShape::ball(1.0).into(),
		..ColliderBundle::default()
	};

	commands.spawn_bundle(PerspectiveCameraBundle {
		transform: Transform {
			translation: Vec3::new(0., 1., 10.),
			..Default::default()
		},
		..Default::default()
	})
	.insert_bundle(camera_collider)
	.insert(FlyCamera::default());
}

pub fn setup_physics(mut commands: Commands) {
	/*
	 * Ground
	 */
	let ground_size = 200.1;
	let ground_height = 0.1;

	let ground = ColliderBundle {
		shape: ColliderShape::cuboid(ground_size, ground_height, ground_size).into(),
		position: Vec3::new(0.0, -ground_height, 0.0).into(),
		..ColliderBundle::default()
	};

	commands
		.spawn_bundle(ground)
		.insert(ColliderDebugRender::default())
		.insert(ColliderPositionSync::Discrete);

	if false {
		spawn_cubes(&mut commands);
	}

	// wheels 1.13
	let mut half_height: f32 = 1.13 / 2.0;
	let mut radius: f32 = 0.84;

	let x = -1.414;
	let y = 0.85; // 0.85
	let z = 1.468;
	let mut pos = Vec3::new(x, y, z);
//	let rf_wheel = spawn_wheel(&pos, half_height, radius, &mut commands);

	pos.x = 1.141;	
//	let lf_wheel = spawn_wheel(&pos, half_height, radius, &mut commands);

	pos.x = 1.35;
	pos.z = -1.89;
//	let lr_wheel = spawn_wheel(&pos, half_height, radius, &mut commands);

	pos.x = -1.35;
	pos.z = -1.89;
//	let rr_wheel = spawn_wheel(&pos, half_height, radius, &mut commands);

	pos.x = 0.0;
	pos.y = 1.791;
	pos.z = -0.271;
	let half_size = Vec3::new(0.55, 0.939, 2.599); // original x half size is 0.791
//	let body = spawn_box(&pos, &half_size, &mut commands);

	let locked_axes = JointAxesMask::ANG_X;
	let limited_axes = JointAxesMask::X
			| JointAxesMask::Y
			| JointAxesMask::Z
			| JointAxesMask::ANG_Y
			| JointAxesMask::ANG_Z;

	let mut anchor = point![-0.864, 0.85, 1.468];

//	create_6dof_joint(rf_wheel, body, locked_axes, anchor, &mut commands)

	//
	//
	//

	radius = 0.4;
	half_height = 0.4;

	pos.x = 0.0;
	pos.y = 2.0;
	pos.z = 0.0;
	let wheel1 = spawn_wheel(&pos, half_height, radius, RigidBodyType::Static, &mut commands);

	pos.x = 0.0;
	pos.y = 1.0;
	pos.z = 0.0;
	let wheel2 = spawn_wheel(&pos, half_height, radius, RigidBodyType::Dynamic, &mut commands);

	let anchor1 = point![0.0,0.0,0.0];
	let anchor2 = point![0.0,1.0,0.0];

	create_6dof_joint(wheel1, wheel2, locked_axes, anchor1, anchor2, &mut commands);
}

fn create_6dof_joint(entity1: Entity, entity2: Entity, locked_axes: JointAxesMask, anchor1: nalgebra::Point3<Real>, anchor2: nalgebra::Point3<Real>, commands: &mut Commands) {//, origin: Point3<f32>, num: usize) {
	let mut joint_6dof = SphericalJoint::new().local_anchor1(anchor1).local_anchor2(anchor2); // JointAxesMask::FREE_FIXED_AXES

	commands.spawn_bundle((JointBuilderComponent::new(joint_6dof, entity1, entity2),));
}

fn spawn_wheel(pos_in: &Vec3, half_height: f32, radius: f32, body_type: RigidBodyType,commands: &mut Commands) -> Entity {
	let mut component = RigidBodyPositionComponent::default();
	component.position.translation = Vec3::new(pos_in.x, pos_in.y, pos_in.z).into();

	//let axis = Unit::new_normalize(Vector3::new(1.0, 2.0, 3.0));
	//let angle = 1.2;
	//let rot = UnitQuaternion::from_axis_angle(&axis, angle);
	let rot = nalgebra::UnitQuaternion::from_axis_angle(&nalgebra::Vector3::z_axis(), std::f32::consts::FRAC_PI_2);
//	component.position.rotation = rot;

	let rigid_body = RigidBodyBundle {
		position: component,
		body_type: RigidBodyTypeComponent(body_type),
		..RigidBodyBundle::default()
	};

	let wheel_collider = ColliderBundle {
		shape: ColliderShape::cylinder(half_height, radius).into(),
//		shape: ColliderShape::ball(radius).into(),
//		shape: ColliderShape::cuboid(half_height, half_height, half_height).into(),
		..ColliderBundle::default()
	};

	commands
		.spawn()
		.insert_bundle(rigid_body)
		.insert_bundle(wheel_collider)
		.insert(ColliderDebugRender::default())
		.insert(ColliderPositionSync::Discrete)
		.id()
}

fn spawn_box(pos_in: &Vec3, half_size: &Vec3, commands: &mut Commands) -> Entity {
	let mut component = RigidBodyPositionComponent::default();
	component.position.translation = Vec3::new(pos_in.x, pos_in.y, pos_in.z).into();

	let rigid_body = RigidBodyBundle {
		position: component,
		..RigidBodyBundle::default()
	};

	let box_collider = ColliderBundle {
		shape: ColliderShape::cuboid(half_size.x, half_size.y, half_size.z).into(),
//		shape: ColliderShape::ball(half_size.x).into(),
		..ColliderBundle::default()
	};

	commands
		.spawn()
		.insert_bundle(rigid_body)
		.insert_bundle(box_collider)
		.insert(ColliderDebugRender::default())
		.insert(ColliderPositionSync::Discrete)
		.id()
}

fn spawn_cubes(commands: &mut Commands) {
	/*
	 * Create the cubes
	 */
	 let num = 8;
	 let rad = 1.0;
 
	 let shift = rad * 2.0 + rad;
	 let centerx = shift * (num / 2) as f32;
	 let centery = shift / 2.0;
	 let centerz = shift * (num / 2) as f32;
 
	 let mut offset = -(num as f32) * (rad * 2.0 + rad) * 0.5;
	 let mut color = 0;
 
	 for j in 0usize..20 {
		 for i in 0..num {
			 for k in 0usize..num {
				 let x = i as f32 * shift - centerx + offset;
				 let y = j as f32 * shift + centery + 3.0;
				 let z = k as f32 * shift - centerz + offset;
				 color += 1;
 
				 // Build the rigid body.
				 let rigid_body = RigidBodyBundle {
					 position: Vec3::new(x, y, z).into(),
					 ..RigidBodyBundle::default()
				 };
 
				 let collider = ColliderBundle {
					 shape: ColliderShape::cuboid(rad, rad, rad).into(),
					 ..ColliderBundle::default()
				 };
 
				 commands
					 .spawn()
					 .insert_bundle(rigid_body)
					 .insert_bundle(collider)
					 .insert(ColliderDebugRender::with_id(color))
					 .insert(ColliderPositionSync::Discrete);
			 }
		 }
 
		 offset -= 0.05 * rad * (num as f32 - 1.0);
	 }
}

fn cursor_grab_system(
	mut windows: ResMut<Windows>,
	btn: Res<Input<MouseButton>>,
	key: Res<Input<KeyCode>>,
) {
	let window = windows.get_primary_mut().unwrap();

	if btn.just_pressed(MouseButton::Left) {
		window.set_cursor_lock_mode(true);
		window.set_cursor_visibility(false);
	}

	if key.just_pressed(KeyCode::Escape) {
		window.set_cursor_lock_mode(false);
		window.set_cursor_visibility(true);
	}
}

fn toggle_button_system(
	btn: Res<Input<MouseButton>>,
	key: Res<Input<KeyCode>>,
	mut exit: EventWriter<AppExit>,
	mut query: Query<&mut FlyCamera>,
) {
	for mut options in query.iter_mut() {
		if btn.just_pressed(MouseButton::Left) {
			options.enabled = true;
		}

		if key.just_pressed(KeyCode::Escape) {
			if options.enabled {
			options.enabled = false;
			} else {
				exit.send(AppExit);
			}
		}
	}
}

fn camera_collision_system(
	time: Res<Time>,
	keyboard_input: Res<Input<KeyCode>>,
	mut query: Query<(&mut FlyCamera, &mut Transform, &mut ColliderPositionComponent)>,
) {
	for (mut options, mut transform, mut collider_position) in query.iter_mut() {
		collider_position.translation = transform.translation.into();
	}
}
