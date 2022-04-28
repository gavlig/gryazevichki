use bevy::prelude::*;
use bevy_rapier3d::{prelude::*, physics::JointHandleComponent};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy::app::AppExit;

use nalgebra as nalg;
use nalg::vector;

use bevy::render::mesh::shape as render_shape;

#[derive(Component)]
pub struct NameComponent {
    name			: String
}

#[derive(Component)]
pub struct WheelTag;
#[derive(Component)]
pub struct AxleTag;
#[derive(Component)]
pub struct BodyTag;

#[derive(Component)]
pub enum Tag {
	FrontWheel,
	RearWheel,
	Axle,
	Body
}

#[derive(Default)]
pub struct Game {
	  camera		: Option<Entity>
	, body 			: Option<Entity>

	, rf_axle_joint	: Option<Entity>
	, lf_axle_joint	: Option<Entity>
	, rr_axle_joint	: Option<Entity>
	, lr_axle_joint	: Option<Entity>

	, rf_wheel_joint: Option<Entity>
	, lf_wheel_joint: Option<Entity>
	, rr_wheel_joint: Option<Entity>
	, lr_wheel_joint: Option<Entity>

	, rf_wheel		: Option<Entity>
	, lf_wheel		: Option<Entity>
	, rr_wheel		: Option<Entity>
	, lr_wheel		: Option<Entity>
}

pub struct VehicleConfig {
	  front_hh		: f32
	, front_r		: f32
	, rear_hh		: f32
	, rear_r		: f32
}

impl Default for VehicleConfig {
    fn default() -> Self {
        Self {
            front_hh: 0.5,
            front_r: 0.8,
            rear_hh: 0.5,
			rear_r: 0.8, 
        }
    }
}

fn main() {
	App::new()
		.insert_resource(ClearColor(Color::rgb(
			0xF9 as f32 / 255.0,
			0xF9 as f32 / 255.0,
			0xFF as f32 / 255.0,
		)))
		.insert_resource(Msaa::default())
		.init_resource::<Game>()
		.insert_resource(VehicleConfig::default())
		.add_plugins(DefaultPlugins)
		.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
		.add_plugin(RapierRenderPlugin)
		.add_plugin(FlyCameraPlugin)
		.add_plugin(bevy_egui::EguiPlugin)
		.add_startup_system(setup_graphics_system)
		.add_startup_system(setup_physics_system)
		.add_startup_system(setup_grab_system)
		.add_startup_system_to_stage(StartupStage::PostStartup, setup_camera_system)
		.add_system(cursor_grab_system)
		.add_system(toggle_button_system)
		.add_system(camera_collision_system)
		.add_system(accelerate_system)
		.add_system(update_ui)
		.add_system_to_stage(CoreStage::PostUpdate, display_events_system)
		.run();
}

fn setup_grab_system(mut windows: ResMut<Windows>) {
	let window = windows.get_primary_mut().unwrap();

	window.set_cursor_lock_mode(true);
	window.set_cursor_visibility(false);
}

fn setup_graphics_system(
		meshes: ResMut<Assets<Mesh>>,
		materials: ResMut<Assets<StandardMaterial>>,
	mut game: ResMut<Game>,
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

	//

	spawn_world_axis(meshes, materials, &mut commands);

	spawn_camera(&mut game, &mut commands);
}

fn spawn_camera(
	game				: &mut ResMut<Game>,
	commands			: &mut Commands
) {
	let camera_collider = ColliderBundle {
		shape: ColliderShape::ball(1.0).into(),
		..ColliderBundle::default()
	};

	let camera = commands
		.spawn_bundle(PerspectiveCameraBundle {
		transform: Transform {
			translation: Vec3::new(0., 1., 10.),
			..Default::default()
		},
		..Default::default()
	})
	.insert_bundle(camera_collider)
		.insert(FlyCamera::default())
		.id();
	game.camera = Some(camera);
	println!("camera Entity ID {:?}", camera);
}

pub fn setup_physics_system(
	mut configuration	: ResMut<RapierConfiguration>,
	mut game			: ResMut<Game>,
		vehicle_cfg		: Res<VehicleConfig>,
	mut commands		: Commands
) {
	configuration.timestep_mode = TimestepMode::VariableTimestep;

	spawn_ground		(&mut game, &mut commands);

	if false {
		spawn_cubes		(&mut commands);
	}

	spawn_vehicle		(&mut game, &vehicle_cfg, &mut commands);
}

fn setup_camera_system(
		 game			: ResMut<Game>,
	mut query			: Query<&mut FlyCamera>
) {
	// initialize camera with target to look at
	if game.camera.is_some() && game.body.is_some() {
		let mut camera 	= query.get_mut(game.camera.unwrap()).unwrap();
		camera.target 	= Some(game.body.unwrap());
		println!("{:?} camera.target", camera.target);
	}
}

fn spawn_ground(
	game				: &mut ResMut<Game>,
	commands			: &mut Commands
) {
	let ground_size 	= 200.1;
	let ground_height 	= 0.1;

	let ground_bundle 	= ColliderBundle {
		shape			: ColliderShape::cuboid(ground_size, ground_height, ground_size).into(),
		position		: Vec3::new(0.0, -ground_height, 0.0).into(),
		..ColliderBundle::default()
	};

	let ground 			= commands
		.spawn_bundle	(ground_bundle)
		.insert			(ColliderDebugRender::default())
		.insert			(ColliderPositionSync::Discrete)
		.id				();

	println!			("ground Entity ID {:?}", ground);
}

fn spawn_vehicle(
		game			: &mut ResMut<Game>,
		vehicle_cfg		: &Res<VehicleConfig>,
	mut commands		: &mut Commands
) {
	let body_pos = Vec3::new(0.0, 5.5, 0.0);
	let body_half_size = Vec3::new(0.5, 0.5, 1.0);
	let body = spawn_body(body_pos, body_half_size, RigidBodyType::Dynamic, &mut commands);
	game.body = Some(body);
	println!("body Entity ID {:?}", body);

	// offsets for wheel placement relative to body center
	let x_off = 0.8;
	let y_off = 0.8;
	let z_off = 1.4;

	{
		let offset = Vec3::new(x_off, -y_off, z_off);
		let (rf_axle_joint, rf_wheel_joint, rf_wheel) =
		spawn_attached_wheel("RF".to_string(), Tag::FrontWheel, body, body_pos, offset, vehicle_cfg.front_hh, vehicle_cfg.front_r, &mut commands);
		(game.rf_axle_joint, game.rf_wheel_joint, game.rf_wheel) = (Some(rf_axle_joint), Some(rf_wheel_joint), Some(rf_wheel));
		println!("rf_wheel Entity ID {:?}", rf_wheel);
	}

	if true {
		let offset = Vec3::new(-x_off, -y_off, z_off);
		let (lf_axle_joint, lf_wheel_joint, lf_wheel) =
		spawn_attached_wheel("LF".to_string(), Tag::FrontWheel, body, body_pos, offset, vehicle_cfg.front_hh, vehicle_cfg.front_r, &mut commands);
		(game.lf_axle_joint, game.lf_wheel_joint, game.lf_wheel) = (Some(lf_axle_joint), Some(lf_wheel_joint), Some(lf_wheel));
		println!("lf_wheel Entity ID {:?}", lf_wheel);
	}

	if true {
		let offset = Vec3::new(x_off, -y_off, -z_off);
		let (rr_axle_joint, rr_wheel_joint, rr_wheel) =
		spawn_attached_wheel("RR".to_string(), Tag::RearWheel, body, body_pos, offset, vehicle_cfg.rear_hh, vehicle_cfg.rear_r, &mut commands);
		(game.rr_axle_joint, game.rr_wheel_joint, game.rr_wheel) = (Some(rr_axle_joint), Some(rr_wheel_joint), Some(rr_wheel));
		println!("rr_wheel Entity ID {:?}", rr_wheel);
	}

	if true {
		let offset = Vec3::new(-x_off, -y_off, -z_off);
		let (lr_axle_joint, lr_wheel_joint, lr_wheel) =
		spawn_attached_wheel("LR".to_string(), Tag::RearWheel, body, body_pos, offset, vehicle_cfg.rear_hh, vehicle_cfg.rear_r, &mut commands);
		(game.lr_axle_joint, game.lr_wheel_joint, game.lr_wheel) = (Some(lr_axle_joint), Some(lr_wheel_joint), Some(lr_wheel));
		println!("lr_wheel Entity ID {:?}", lr_wheel);
	}
}

fn spawn_attached_wheel(
	prefix			: String,
	tag				: Tag,
	body			: Entity,
	body_pos		: Vec3,
	main_offset		: Vec3,
	half_height		: f32,
	radius			: f32, 
	mut	commands	: &mut Commands
) -> (Entity, Entity, Entity) {
	let x_sign		= main_offset.x * (1.0 / main_offset.x.abs());
	let wheel_offset= Vec3::X * 0.8 * x_sign; // 0.2 offset by x axis

	let axle_size	= Vec3::new(0.1, 0.2, 0.1);
	let axle_pos	= body_pos + main_offset;
	let axle		= spawn_axle(&prefix, axle_pos, axle_size, RigidBodyType::Dynamic, &mut commands);

	let mut anchor1	= main_offset;
	let mut anchor2 = Vec3::ZERO;
	let axle_joint 	= spawn_axle_joint(body, axle, point![anchor1.x, anchor1.y, anchor1.z], point![anchor2.x, anchor2.y, anchor2.z], &mut commands);

	let wheel_pos 	= axle_pos + wheel_offset;
	let wheel 		= spawn_wheel(&prefix, tag, wheel_pos, half_height, radius, RigidBodyType::Dynamic, &mut commands);

	anchor1			= wheel_offset;
	anchor2 		= Vec3::ZERO;
	let wheel_joint = spawn_wheel_joint(axle, wheel, point![anchor1.x, anchor1.y, anchor1.z], point![anchor2.x, anchor2.y, anchor2.z], &mut commands);

	(axle_joint, wheel_joint, wheel)
}

fn spawn_axle(
	prefix			: &String,
	pos_in			: Vec3,
	half_size		: Vec3,
	body_type		: RigidBodyType,
	commands		: &mut Commands,
) -> Entity {
	let mut pos_comp = RigidBodyPositionComponent::default();
	pos_comp.position.translation = pos_in.clone().into();

	let rigid_body = RigidBodyBundle {
		position: pos_comp,
		body_type: RigidBodyTypeComponent(body_type),
		..RigidBodyBundle::default()
	};

	let translated_position = nalg::Isometry3 {
		translation: vector![0.0, 0.3, 0.0].into(),
		..Default::default()
	};

	let axle_collider = ColliderBundle {
		shape: ColliderShape::cuboid(half_size.x, half_size.y, half_size.z).into(),
		position: translated_position.into(),
		mass_properties: ColliderMassProps::Density(1000.0).into(),
		..ColliderBundle::default()
	};

	commands
		.spawn()
		.insert_bundle(rigid_body)
		.insert_bundle(axle_collider)
		.insert(ColliderDebugRender::default())
		.insert(ColliderPositionSync::Discrete)
		.insert(NameComponent{ name: format!("{} Axle", prefix) })
		.insert(Tag::Axle)
		.id()
}

fn spawn_wheel(
	prefix: &String,
	tag: Tag,
	pos_in: Vec3,
	half_height: f32,
	radius: f32,
	body_type: RigidBodyType,
	commands: &mut Commands,
) -> Entity {
	let mut pos_comp = RigidBodyPositionComponent::default();
	pos_comp.position.translation = pos_in.clone().into();

	let rigid_body = RigidBodyBundle {
		position: pos_comp,
		body_type: RigidBodyTypeComponent(body_type),
		..RigidBodyBundle::default()
	};

	// by default cylinder spawns with its flat surface on the ground and we want the round part
	let wheel_rotation = nalgebra::UnitQuaternion::from_axis_angle(
		&nalgebra::Vector3::z_axis(),
		std::f32::consts::FRAC_PI_2,
	);

	let rotated_position = nalg::Isometry3 {
		rotation: wheel_rotation,
		..Default::default()
	};

	let wheel_collider = ColliderBundle {
		shape: ColliderShape::cylinder(half_height, radius).into(),
		position: rotated_position.into(),
		mass_properties: ColliderMassProps::Density(2.0).into(),
		flags: (ActiveEvents::INTERSECTION_EVENTS | ActiveEvents::CONTACT_EVENTS).into(),
		..ColliderBundle::default()
	};

	commands
		.spawn()
		.insert_bundle(rigid_body)
		.insert_bundle(wheel_collider)
		.insert(ColliderDebugRender::default())
		.insert(ColliderPositionSync::Discrete)
		.insert(NameComponent{ name: format!("{} Wheel", prefix) })
		.insert(tag)
		.id()
}

fn spawn_axle_joint(
	entity1: Entity,
	entity2: Entity,
	anchor1: nalgebra::Point3<Real>,
	anchor2: nalgebra::Point3<Real>,
	commands: &mut Commands,
) -> Entity {
	let axle_joint = RevoluteJoint::new(Vector::y_axis())
		.local_anchor1(anchor1)
		.local_anchor2(anchor2)
		.motor_position(0.0, 10.0, 3.0); // by default we want axle joint to stay fixed 

	commands
		.spawn()
		.insert(JointBuilderComponent::new(axle_joint, entity1, entity2))
		.id()
}

fn spawn_wheel_joint(
	entity1: Entity,
	entity2: Entity,
	anchor1: nalgebra::Point3<Real>,
	anchor2: nalgebra::Point3<Real>,
	commands: &mut Commands,
) -> Entity {
	let wheel_joint = RevoluteJoint::new(Vector::x_axis())
		.local_anchor1(anchor1)
		.local_anchor2(anchor2);

	commands
		.spawn()
		.insert(JointBuilderComponent::new(wheel_joint, entity1, entity2))
		.id()
}

fn spawn_body(
	pos_in: Vec3,
	half_size: Vec3,
	body_type: RigidBodyType,
	commands: &mut Commands,
) -> Entity {
	let mut component = RigidBodyPositionComponent::default();
	component.position.translation = pos_in.clone().into();

	let rigid_body = RigidBodyBundle {
		position: component,
		body_type: RigidBodyTypeComponent(body_type),
		..RigidBodyBundle::default()
	};

	let box_collider = ColliderBundle {
		shape: ColliderShape::cuboid(half_size.x, half_size.y, half_size.z).into(),
		mass_properties: ColliderMassProps::Density(10.0).into(), // joints like it when there is an hierarchy of masses and we want body to be the heaviest
		..ColliderBundle::default()
	};

	commands
		.spawn()
		.insert_bundle(rigid_body)
		.insert_bundle(box_collider)
		.insert(ColliderDebugRender::default())
		.insert(ColliderPositionSync::Discrete)
		.insert(NameComponent{ name: "Body".to_string() })
		.insert(Tag::Body)
		.id()
}

fn spawn_cubes(commands: &mut Commands) {
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

fn spawn_world_axis(
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	commands: &mut Commands,
) {
	// X
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(render_shape::Box::new(1.0, 0.1, 0.1))),
		material: materials.add(Color::rgb(0.8, 0.1, 0.1).into()),
		transform: Transform::from_xyz(0.5, 0.0 + 0.05, 0.0),
		..Default::default()
	});
	// Y
	commands.spawn_bundle(PbrBundle {
		mesh: meshes.add(Mesh::from(render_shape::Box::new(0.1, 1.0, 0.1))),
		material: materials.add(Color::rgb(0.1, 0.8, 0.1).into()),
		transform: Transform::from_xyz(0.0, 0.5 + 0.05, 0.0),
		..Default::default()
	});
	// Z
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

	if key.just_pressed(KeyCode::Return) {
		window.set_cursor_lock_mode(true);
		window.set_cursor_visibility(false);
	}

	if key.just_pressed(KeyCode::Escape) {
		let toggle = !window.cursor_visible();
		window.set_cursor_visibility(toggle);
		window.set_cursor_lock_mode(!toggle);
	}
}

fn toggle_button_system(
		btn		: Res<Input<MouseButton>>,
		key		: Res<Input<KeyCode>>,
		game	: Res<Game>,
	mut exit	: EventWriter<AppExit>,
	mut query	: Query<&mut FlyCamera>,
) {
	for mut camera in query.iter_mut() {
		if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Space) {
			let toggle = !camera.enabled_follow;
			camera.enabled_follow = toggle;
		}

		if key.just_pressed(KeyCode::Escape) {
			let toggle = !camera.enabled_rotation;
			camera.enabled_rotation = toggle;
		}

		if key.pressed(KeyCode::LControl) && key.just_pressed(KeyCode::Escape) {
			exit.send(AppExit);
		}
	}
}

fn motor_velocity(velocity: f32, factor: f32, joint_e: Entity, joints: &mut ResMut<ImpulseJointSet>, query: &mut Query<&mut JointHandleComponent>) {
	let 	joint_comp	= query.get(joint_e).unwrap();
	let mut joint		= joints.get_mut(joint_comp.handle()).unwrap();
			joint.data	= joint.data.motor_velocity(JointAxis::AngX, velocity, factor);
}

fn motor_steer(angle: f32, stiffness: f32, damping: f32, joint_e: Entity, joints: &mut ResMut<ImpulseJointSet>, query: &mut Query<&mut JointHandleComponent>) {
	let 	joint_comp 	= query.get(joint_e).unwrap();

	let		angle_rad	= angle.to_radians();
	let mut joint 		= joints.get_mut(joint_comp.handle()).unwrap();
			joint.data 	= joint.data.motor_position(JointAxis::AngX, angle_rad, stiffness, damping)

//	println!("motor steer {} limit axes {:?}", angle, joint.data.limit_axes);
//	if angle.abs() > 0.0001 {
//			joint.data 	= joint.data.unlimit_axis(JointAxis::AngZ);
//	} else {
//			joint.data 	= joint.data.limit_axis(JointAxis::AngZ, [-0.0001, 0.0001]);
//	}
}

fn accelerate_system(
		key		: Res<Input<KeyCode>>,
		game	: ResMut<Game>,
	mut	joints	: ResMut<ImpulseJointSet>,
	mut	query	: Query<&mut JointHandleComponent>,
) {
	let rf_axle_joint = game.rf_axle_joint.unwrap();
	let lf_axle_joint = game.lf_axle_joint.unwrap();
	let rr_axle_joint = game.rr_axle_joint.unwrap();
	let lr_axle_joint = game.lr_axle_joint.unwrap();

	let rf_wheel_joint = game.rf_wheel_joint.unwrap();
	let lf_wheel_joint = game.lf_wheel_joint.unwrap();
	let rr_wheel_joint = game.rr_wheel_joint.unwrap();
	let lr_wheel_joint = game.lr_wheel_joint.unwrap();

	if key.just_pressed(KeyCode::W) {
		motor_velocity(10.0, 0.7, rr_wheel_joint, &mut joints, &mut query);
		motor_velocity(10.0, 0.7, lr_wheel_joint, &mut joints, &mut query);
	} else if key.just_released(KeyCode::W) {
		motor_velocity(0.0, 0.7, rr_wheel_joint, &mut joints, &mut query);
		motor_velocity(0.0, 0.7, lr_wheel_joint, &mut joints, &mut query);
	}
	
	 if key.just_pressed(KeyCode::S) {
		motor_velocity(-10.0, 0.3, rr_wheel_joint, &mut joints, &mut query);
		motor_velocity(-10.0, 0.3, lr_wheel_joint, &mut joints, &mut query);
	} else if key.just_released(KeyCode::S) {
		motor_velocity(0.0, 0.7, rr_wheel_joint, &mut joints, &mut query);
		motor_velocity(0.0, 0.7, lr_wheel_joint, &mut joints, &mut query);
	}
 
	let steer_angle = 20.0;
	let stiffness = 5.0;
	let damping = 3.0;
	if key.just_pressed(KeyCode::D) {
		motor_steer(-steer_angle, stiffness, damping, rf_axle_joint, &mut joints, &mut query);
		motor_steer(-steer_angle, stiffness, damping, lf_axle_joint, &mut joints, &mut query);
	} else if key.just_released(KeyCode::D) {
		motor_steer(0.0, stiffness, damping, rf_axle_joint, &mut joints, &mut query);
		motor_steer(0.0, stiffness, damping, lf_axle_joint, &mut joints, &mut query);
	}

 	if key.just_pressed(KeyCode::A) {
		motor_steer(steer_angle, stiffness, damping, rf_axle_joint, &mut joints, &mut query);
		motor_steer(steer_angle, stiffness, damping, lf_axle_joint, &mut joints, &mut query);
	} else if key.just_released(KeyCode::A) {
		motor_steer(0.0, stiffness, damping, rf_axle_joint, &mut joints, &mut query);
		motor_steer(0.0, stiffness, damping, lf_axle_joint, &mut joints, &mut query);
	}
}

fn camera_collision_system(
	mut query: Query<(
		&	 FlyCamera,
		&	 Transform,
		&mut ColliderPositionComponent,
	)>,
) {
	for (_options, transform, mut collider_position) in query.iter_mut() {
		collider_position.translation = transform.translation.into();
	}
}

fn display_events_system(
	mut intersection_events: EventReader<IntersectionEvent>,
	mut contact_events: EventReader<ContactEvent>,
) {
	if true {return}

	for intersection_event in intersection_events.iter() {
		println!("Received intersection event: collider1 {:?} collider2 {:?}", intersection_event.collider1.entity(), intersection_event.collider2.entity());
	}

	for contact_event in contact_events.iter() {
		match contact_event {
			ContactEvent::Started(collider1, collider2) => println!("Received contact START event: collider1 {:?} collider2 {:?}", collider1.entity(), collider2.entity()),
			ContactEvent::Stopped(collider1, collider2) => println!("Received contact STOP event: collider1 {:?} collider2 {:?}", collider1.entity(), collider2.entity()),
		}
	}
}

use bevy_egui::egui::{Slider, Ui};
use bevy_egui::{egui, EguiContext};

fn set_cylinder_hh(
	mut coll_shape: &mut ColliderShapeComponent,
	new_hh: f32,
) {
	let 	shape 	= coll_shape.make_mut();
	let mut cylinder= shape.as_cylinder_mut().unwrap();
	cylinder.half_height = new_hh;
}

fn set_cylinder_r(
	mut coll_shape: &mut ColliderShapeComponent,
	new_r: f32,
) {
	let 	shape 	= coll_shape.make_mut();
	let mut cylinder= shape.as_cylinder_mut().unwrap();
	cylinder.radius = new_r;
}

fn draw_density_param_ui(
	ui: &mut Ui,
	name: &String,
	mut mass_props_coll: &mut ColliderMassPropsComponent,
	mass_props_rbody: &mut RigidBodyMassPropsComponent,
	coll_shape: &mut ColliderShapeComponent,
) {
	let prev_props = mass_props_coll.mass_properties(&***coll_shape).clone();
	match &mut mass_props_coll as &mut ColliderMassProps {
		ColliderMassProps::Density(density) => {
			if ui.add(
				Slider::new(&mut *density, 0.01 ..= 1000.0).text(format!("{} Density", name))
			).changed() {
				mass_props_rbody.local_mprops -= prev_props;
				mass_props_rbody.local_mprops += mass_props_coll.mass_properties(&***coll_shape);
			}; 
		},
		ColliderMassProps::MassProperties(_) => (),
	};
}

fn draw_single_wheel_params_ui(
	ui: &mut Ui,
	name: &String,
	mass_props_coll: &mut ColliderMassPropsComponent,
	mass_props_rbody: &mut RigidBodyMassPropsComponent,
	coll_shape: &mut ColliderShapeComponent,
	tag: &Tag,
) {
	draw_density_param_ui(ui, &name[3..].to_string(), mass_props_coll, mass_props_rbody, coll_shape);

	match tag {
		Tag::FrontWheel | Tag::RearWheel => {
			let shape = coll_shape.make_mut();
			let cylinder = shape.as_cylinder_mut().unwrap();

			egui::CollapsingHeader::new("Wheel sizes")
				.default_open(true)
				.show(ui, |ui| {

			ui.vertical(|ui| {
			
			let label = format!("{} radius", cylinder.radius);
			ui.add(
				Slider::new(&mut cylinder.radius, 0.05 ..= 1.0)
					.text(label),
			);

			let label = format!("{} half height", cylinder.half_height);
			ui.add(
				Slider::new(&mut cylinder.half_height, 0.05 ..= 1.0)
					.text(label),
			);

			}); // ui.vertical
			}); // ui.collapsing
		},
		_ => (),
	}
}

fn draw_body_params_ui_collapsing(
	ui: &mut Ui,
	name: &String,
	mass_props_coll: &mut ColliderMassPropsComponent,
	mass_props_rbody: &mut RigidBodyMassPropsComponent,
	coll_shape: &mut ColliderShapeComponent,
	section_name: String
) {
	ui.collapsing(section_name, |ui| {
		ui.vertical(|ui| {
			draw_density_param_ui(ui, name, mass_props_coll, mass_props_rbody, coll_shape);

			let shape = coll_shape.make_mut();
			let cuboid = shape.as_cuboid_mut().unwrap();

			let label = format!("{} half height X", cuboid.half_extents[0]);
			ui.add(
				Slider::new(&mut cuboid.half_extents[0], 0.05 ..= 5.0)
					.text(label),
			);

			let label = format!("{} half height Y", cuboid.half_extents[1]);
			ui.add(
				Slider::new(&mut cuboid.half_extents[1], 0.05 ..= 5.0)
					.text(label),
			);

			let label = format!("{} half height Z", cuboid.half_extents[2]);
			ui.add(
				Slider::new(&mut cuboid.half_extents[2], 0.05 ..= 5.0)
					.text(label),
			);
		}); // ui.vertical
	}); // ui.collapsing
}

fn draw_single_wheel_params_ui_collapsing(
	ui: &mut Ui,
	wheel: Vec<(
		&String,
		Mut<ColliderMassPropsComponent>,
		Mut<RigidBodyMassPropsComponent>,
		Mut<ColliderShapeComponent>,
		&Tag
	)>,
	section_name: String,
) {
	ui.collapsing(section_name, |ui| {
		ui.vertical(|ui| {
			for (name_in, mut mass_props_coll, mut mass_props_rbody, mut coll_shape, tag) in wheel {
				draw_single_wheel_params_ui(
					ui,
					name_in,
					&mut mass_props_coll,
					&mut mass_props_rbody,
					&mut coll_shape,
					tag,
				);
			}
		});
	});
}

fn update_ui(
	mut ui_context	: ResMut<EguiContext>,
	mut vehicle_cfg	: ResMut<VehicleConfig>,
	mut	query		: Query<(
		&mut ColliderMassPropsComponent,
		&mut RigidBodyMassPropsComponent,
		&mut ColliderShapeComponent,
		&NameComponent,
		&Tag
	)>
) {
	egui::Window::new("Parameters").show(ui_context.ctx_mut(), |ui| {
		//let mut front_wheels_half_height	= vehicle_cfg.front_hh;
		let mut front_wheels_radius 		= vehicle_cfg.front_r;
		let mut rear_wheels_half_height 	= vehicle_cfg.rear_hh;
		let mut rear_wheels_radius 			= vehicle_cfg.rear_r;

		let mut front_wh_hh_changed			= false;
		let mut front_wh_r_changed			= false;
		let mut rear_wh_hh_changed			= false;
		let mut rear_wh_r_changed			= false;

		ui.collapsing("Front Wheels".to_string(), |ui| {
		ui.vertical(|ui| {

		front_wh_hh_changed = ui.add(
			Slider::new(&mut vehicle_cfg.front_hh, 0.05 ..= 1.0)
				.text("Front wheels half height"),
		).changed();

		
		front_wh_r_changed = ui.add(
			Slider::new(&mut front_wheels_radius, 0.05 ..= 1.0)
				.text("Front wheels radius"),
		).changed();

		}); // ui.vertical
		}); // ui.collapsing

		ui.collapsing("Rear Wheels".to_string(), |ui| {
		ui.vertical(|ui| {

		
		rear_wh_hh_changed = ui.add(
			Slider::new(&mut rear_wheels_half_height, 0.05 ..= 1.0)
				.text("Rear wheels half height"),
		).changed();

		rear_wh_r_changed = ui.add(
			Slider::new(&mut rear_wheels_radius, 0.05 ..= 1.0)
				.text("Rear wheels radius"),
		).changed();

		}); // ui.vertical
		}); // ui.collapsing

		let mut RF = vec![];
		let mut LF = vec![];
		let mut RR = vec![];
		let mut LR = vec![];

		for (mass_props_coll, mass_props_rbody, mut coll_shape, name_comp, tag) in query.iter_mut() {
			let name = &name_comp.name;

			match tag {
				Tag::FrontWheel => {
					if front_wh_hh_changed {
						set_cylinder_hh(&mut coll_shape, vehicle_cfg.front_hh);
					}
					if front_wh_r_changed {
						set_cylinder_r(&mut coll_shape, vehicle_cfg.front_r);
					}
				},
				Tag::RearWheel  => {
					if rear_wh_hh_changed {
						set_cylinder_hh(&mut coll_shape, vehicle_cfg.rear_hh);
					}
					if rear_wh_r_changed {
						set_cylinder_r(&mut coll_shape, vehicle_cfg.rear_r);
					}
				}
				_ => (),
			}

			let to_push = (name, mass_props_coll, mass_props_rbody, coll_shape, tag);
			if name.starts_with("RF") {
				RF.push(to_push);
			} else if name.starts_with("LF") {
				LF.push(to_push);
			} else if name.starts_with("RR") {
				RR.push(to_push);
			} else if name.starts_with("LR") {
				LR.push(to_push);
			} else if name.eq("Body") {
				// thanks kpreid!
				let (name, mut mass_props_coll, mut mass_props_rbody, mut coll_shape, _) = to_push;
				draw_body_params_ui_collapsing(ui, name, &mut mass_props_coll, &mut mass_props_rbody, &mut coll_shape, "Body".to_string());
			}
		}

		draw_single_wheel_params_ui_collapsing(ui, RF, "RF".to_string());
		draw_single_wheel_params_ui_collapsing(ui, LF, "LF".to_string());
		draw_single_wheel_params_ui_collapsing(ui, RR, "RR".to_string());
		draw_single_wheel_params_ui_collapsing(ui, LR, "LR".to_string());
	});
}

// contact info + modification. I'd rather add more info to event
/* fn display_contact_info(narrow_phase: Res<NarrowPhase>) {
    let entity1 = ...; // The first entity with a collider bundle attached.
    let entity2 = ...; // The second entity with a collider bundle attached.
    
    /* Find the contact pair, if it exists, between two colliders. */
    if let Some(contact_pair) = narrow_phase.contact_pair(entity1.handle(), entity2.handle()) {
        // The contact pair exists meaning that the broad-phase identified a potential contact.
        if contact_pair.has_any_active_contact {
            // The contact pair has active contacts, meaning that it
            // contains contacts for which contact forces were computed.
        }

        // We may also read the contact manifolds to access the contact geometry.
        for manifold in &contact_pair.manifolds {
            println!("Local-space contact normal: {}", manifold.local_n1);
            println!("Local-space contact normal: {}", manifold.local_n2);
            println!("World-space contact normal: {}", manifold.data.normal);

            // Read the geometric contacts.
            for contact_point in &manifold.points {
                // Keep in mind that all the geometric contact data are expressed in the local-space of the colliders.
                println!("Found local contact point 1: {:?}", contact_point.local_p1);
                println!("Found contact distance: {:?}", contact_point.dist); // Negative if there is a penetration.
                println!("Found contact impulse: {}", contact_point.data.impulse);
                println!("Found friction impulse: {}", contact_point.data.tangent_impulse);
            }

            // Read the solver contacts.
            for solver_contact in &manifold.data.solver_contacts {
                // Keep in mind that all the solver contact data are expressed in world-space.
                println!("Found solver contact point: {:?}", solver_contact.point);
                println!("Found solver contact distance: {:?}", solver_contact.dist); // Negative if there is a penetration.
            }
        }
    }
} */
