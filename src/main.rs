use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
//use bevy_editor_pls::prelude::*;
use bevy::app::AppExit;
use nalgebra as nalg;

use bevy::render::mesh::shape as render_shape;

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
		.add_system_to_stage(CoreStage::PostUpdate, display_events_system)
		.run();
}

fn setup_grab_system(
	mut windows: ResMut<Windows>,
) {
	let window = windows.get_primary_mut().unwrap();

	window.set_cursor_lock_mode(true);
	window.set_cursor_visibility(false);
}

fn setup_graphics(
	mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
	mut commands: Commands,
) {
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

	// axis

	spawn_axis(meshes, materials, &mut commands);

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
	let mut half_height: f32 = 0.5;
	let mut radius: f32 = 0.80;

	let pos = Vec3::new(0.0, 5.5, 0.0);
	let half_size = Vec3::new(0.5, 0.5, 1.0);
	let body = spawn_box(&pos, &half_size, RigidBodyType::Dynamic, &mut commands);

	{
		let rf_wheel = spawn_wheel(half_height, radius, RigidBodyType::Dynamic, &mut commands);

		let anchor1 = point![1.6,-0.8,1.4];
		let anchor2 = point![0.0,0.0,0.0];

		create_6dof_joint(body, rf_wheel, anchor1, anchor2, &mut commands)
	}

	if false {
		let lf_wheel = spawn_wheel(half_height, radius, RigidBodyType::Dynamic, &mut commands);

		let anchor1 = point![-1.6,-0.8,1.4];
		let anchor2 = point![0.0,0.0,0.0];

		create_6dof_joint(body, lf_wheel, anchor1, anchor2, &mut commands)
	}

	if false {
		let rr_wheel = spawn_wheel(half_height, radius, RigidBodyType::Dynamic, &mut commands);

		let anchor1 = point![1.6,-0.8,-1.4];
		let anchor2 = point![0.0,0.0,0.0];

		create_6dof_joint(body, rr_wheel, anchor1, anchor2, &mut commands)
	}

	if false {
		let lr_wheel = spawn_wheel(half_height, radius, RigidBodyType::Dynamic, &mut commands);

		let anchor1 = point![-1.6,-0.8,-1.4];
		let anchor2 = point![0.0,0.0,0.0];

		create_6dof_joint(body, lr_wheel, anchor1, anchor2, &mut commands)
	}
}

fn create_6dof_joint(entity1: Entity, entity2: Entity, anchor1: nalgebra::Point3<Real>, anchor2: nalgebra::Point3<Real>, commands: &mut Commands) {
	let mut joint_6dof = RevoluteJoint::new(Vector::x_axis()).local_anchor1(anchor1).local_anchor2(anchor2);
	joint_6dof = joint_6dof.motor_velocity(1., 0.1);

	commands.spawn().insert(JointBuilderComponent::new(joint_6dof, entity1, entity2));
}

fn spawn_wheel(/* pos_in: &Vec3, */ half_height: f32, radius: f32, body_type: RigidBodyType, commands: &mut Commands) -> Entity {
//	let mut component = RigidBodyPositionComponent::default();
//	component.position.translation = pos_in.clone().into();// Vec3::new(pos_in.x, pos_in.y, pos_in.z).into();

	//let axis = Unit::new_normalize(Vector3::new(1.0, 2.0, 3.0));
	//let angle = 1.2;
	//let rot = UnitQuaternion::from_axis_angle(&axis, angle);
	let rot = nalgebra::UnitQuaternion::from_axis_angle(&nalgebra::Vector3::z_axis(), std::f32::consts::FRAC_PI_2);
//	component.position.rotation = rot;

	let rigid_body = RigidBodyBundle {
//		position: component,
		body_type: RigidBodyTypeComponent(body_type),
		..RigidBodyBundle::default()
	};

	let collider_position = nalg::Isometry3 {
		rotation: rot,
		..Default::default()
	};

	let wheel_collider = ColliderBundle {
		shape: ColliderShape::cylinder(half_height, radius).into(),
		position: collider_position.into(),
//		shape: ColliderShape::ball(radius).into(),
//		shape: ColliderShape::cuboid(half_height, half_height, half_height).into(),

		flags: (ActiveEvents::INTERSECTION_EVENTS | ActiveEvents::CONTACT_EVENTS).into(),
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

fn spawn_box(pos_in: &Vec3, half_size: &Vec3, body_type: RigidBodyType, commands: &mut Commands) -> Entity {
	let mut component = RigidBodyPositionComponent::default();
	component.position.translation = Vec3::new(pos_in.x, pos_in.y, pos_in.z).into();

	let rigid_body = RigidBodyBundle {
		position: component,
		body_type: RigidBodyTypeComponent(body_type),
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

fn spawn_axis(
	mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
	commands: &mut Commands
) {
	commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(render_shape::Box::new(1.0, 0.1, 0.1))),
        material: materials.add(Color::rgb(0.8, 0.1, 0.1).into()),
		transform: Transform::from_xyz(0.5, 0.0 + 0.05, 0.0),
        ..Default::default()
    });
	commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(render_shape::Box::new(0.1, 1.0, 0.1))),
        material: materials.add(Color::rgb(0.1, 0.8, 0.1).into()),
		transform: Transform::from_xyz(0.0, 0.5 + 0.05, 0.0),
        ..Default::default()
    });
	commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(render_shape::Box::new(0.1, 0.1, 1.0))),
        material: materials.add(Color::rgb(0.1, 0.1, 0.8).into()),
		transform: Transform::from_xyz(0.0, 0.0 + 0.05, 0.5),
        ..Default::default()
    });
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

fn display_events_system(
    mut intersection_events: EventReader<IntersectionEvent>,
    mut contact_events: EventReader<ContactEvent>,
) {
    for intersection_event in intersection_events.iter() {
        println!("Received intersection event: {:?}", intersection_event);
    }

    for contact_event in contact_events.iter() {
        println!("Received contact event: {:?}", contact_event);
    }
}