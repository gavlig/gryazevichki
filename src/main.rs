use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

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
		.add_startup_system(setup_graphics.system())
		.add_startup_system(setup_physics.system())
		.run();
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

	commands.spawn_bundle(PerspectiveCameraBundle {
		transform: Transform::from_matrix(Mat4::face_toward(
			Vec3::new(-30.0, 30.0, 15.0),
			Vec3::new(0.0, 10.0, 0.0),
			Vec3::new(0.0, 1.0, 0.0),
		)),
		..Default::default()
	});
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
	let half_height: f32 = 1.13 / 2.0;
	let radius: f32 = 0.84;

	let x = -1.414;
	let y = 0.85; // 0.85
	let z = 1.468;
	let mut pos = Vec3::new(x, y, z);
	spawn_wheel(&pos, half_height, radius, &mut commands);

	pos.x = 1.141;	
	spawn_wheel(&pos, half_height, radius, &mut commands);

	pos.x = 1.35;
	pos.z = -1.89;
	spawn_wheel(&pos, half_height, radius, &mut commands);

	pos.x = -1.35;
	pos.z = -1.89;
	spawn_wheel(&pos, half_height, radius, &mut commands);

	pos.x = 0.0;
	pos.y = 1.791;
	pos.z = -0.271;
	spawn_box(&pos, &Vec3::new(0.791, 0.939, 2.599), &mut commands);
}

fn spawn_wheel(pos_in: &Vec3, half_height: f32, radius: f32, commands: &mut Commands) {
	let mut component = RigidBodyPositionComponent::default();
	component.position.translation = Vec3::new(pos_in.x, pos_in.y, pos_in.z).into();

	//let axis = Unit::new_normalize(Vector3::new(1.0, 2.0, 3.0));
	//let angle = 1.2;
	//let rot = UnitQuaternion::from_axis_angle(&axis, angle);
	let rot = nalgebra::UnitQuaternion::from_axis_angle(&nalgebra::Vector3::z_axis(), std::f32::consts::FRAC_PI_2);
	component.position.rotation = rot;//UnitQuaternion::new(); //.append_axisangle_linearized(&nalgebra::Vector3::new(0.0, 0.0, 90.0));

	let rigid_body = RigidBodyBundle {
		position: component,
		..RigidBodyBundle::default()
	};

	let wheel_collider = ColliderBundle {
		shape: ColliderShape::cylinder(half_height, radius).into(),
		..ColliderBundle::default()
	};

	commands
		.spawn()
		.insert_bundle(rigid_body)
		.insert_bundle(wheel_collider)
		.insert(ColliderDebugRender::default())
		.insert(ColliderPositionSync::Discrete);
}

fn spawn_box(pos_in: &Vec3, size: &Vec3, commands: &mut Commands) {
	let mut component = RigidBodyPositionComponent::default();
	component.position.translation = Vec3::new(pos_in.x, pos_in.y, pos_in.z).into();

	let rigid_body = RigidBodyBundle {
		position: component,
		..RigidBodyBundle::default()
	};

	let box_collider = ColliderBundle {
		shape: ColliderShape::cuboid(size.x, size.y, size.z).into(),
		..ColliderBundle::default()
	};

	commands
		.spawn()
		.insert_bundle(rigid_body)
		.insert_bundle(box_collider)
		.insert(ColliderDebugRender::default())
		.insert(ColliderPositionSync::Discrete);
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