use bevy			    ::	prelude :: *;
use bevy_rapier3d       ::	prelude :: *;
use bevy_fly_camera     ::  FlyCamera;

use super               ::  *;

pub fn spawn_vehicle(
		veh_cfg			: &self::Config,
		body_pos		: Transform,
		game			: &mut ResMut<GameState>,
		ass				: &Res<AssetServer>,
	mut commands		: &mut Commands
) {
	let body_cfg		= veh_cfg.body.unwrap();

	let body 			= spawn_body(body_pos, &body_cfg, &veh_cfg.bophys.unwrap(), &veh_cfg.accel.unwrap(), &veh_cfg.steer.unwrap(), ass, &mut commands);
	game.body 			= Some(RespawnableEntity { entity : body, ..default() });
	println!			("body Entity ID {:?}", body);

	// 0..1 {
	for side_ref in WHEEL_SIDES {
		let side 		= *side_ref;

		let axle_cfg	= veh_cfg.axles[side].unwrap();
		let axle_phys_cfg = veh_cfg.axphys[side].unwrap();
		let axle_offset = body_cfg.axle_offset(side);
		
		let wheel_cfg	= veh_cfg.wheels[side].unwrap();
		let wheel_phys_cfg = veh_cfg.whphys[side].unwrap();

		let (axle, wheel) = spawn_attached_wheel(side, body, body_pos, axle_offset, &axle_cfg, &axle_phys_cfg, &wheel_cfg, &wheel_phys_cfg, ass, &mut commands);

		game.axles[side] = Some(axle);
		game.wheels[side] = Some(wheel);

		println!		("{} Wheel spawned! {:?}", wheel_side_name(side), game.wheels[side]);
	}
}

fn spawn_attached_wheel(
    side			: WheelSideType,
    body			: Entity,
    body_pos		: Transform,
    axle_offset		: Vec3,
    axle_cfg		: &AxleConfig,
    axle_phys_cfg	: &PhysicsConfig,
    wheel_cfg		: &WheelConfig,
    wheel_phys_cfg	: &PhysicsConfig,
    ass				: &Res<AssetServer>,
    commands		: &mut Commands
) -> (RespawnableEntity, RespawnableEntity) { // axle + wheel 
    let (axle, axle_pos) = spawn_axle_with_joint(side, body, body_pos, axle_offset, axle_cfg, axle_phys_cfg, ass, commands);

    let wheel_offset = axle_cfg.wheel_offset(side);

    let wheel		= spawn_wheel_with_joint(side, axle, axle_pos, wheel_offset, wheel_cfg, wheel_phys_cfg, ass, commands);

    let wheel_pos	= axle_pos * wheel_offset;

    (
    RespawnableEntity{ entity : axle,	..Default::default() },
    RespawnableEntity{ entity : wheel, 	..Default::default() }
    )
}

fn spawn_axle_with_joint(
    side			: WheelSideType,
    body			: Entity,
    body_pos		: Transform,
    offset			: Vec3,
    cfg				: &AxleConfig,
    phys			: &PhysicsConfig,
    ass				: &Res<AssetServer>,
    mut	commands	: &mut Commands
) -> (Entity, Transform) {
    let axle		= spawn_axle(side, body, body_pos, offset, cfg, phys, ass, &mut commands);
    let axle_pos	= body_pos * Transform::from_translation(offset);

    let anchor1		= offset;
    let anchor2 	= Vec3::ZERO;
    spawn_axle_joint(body, axle, anchor1, anchor2, &mut commands);

    (axle, axle_pos)
}

fn spawn_wheel_with_joint(
    side			: WheelSideType,
    axle			: Entity,
    axle_pos		: Transform,
    offset			: Vec3,
    wheel_cfg		: &WheelConfig,
    phys_cfg		: &PhysicsConfig,
    ass				: &Res<AssetServer>,
    mut	commands	: &mut Commands
) -> Entity {
    let wheel 		= spawn_wheel(
        side
        , axle_pos
        , offset
        , wheel_cfg
        , phys_cfg
        , ass
        , &mut commands
    );

    let anchor1		= offset;
    let anchor2 	= Vec3::ZERO;
    spawn_wheel_joint(axle, wheel, anchor1, anchor2, &mut commands);

    wheel
}

fn spawn_axle(
    side			: WheelSideType,
    body			: Entity,
    body_pos		: Transform,
    offset			: Vec3,
    cfg				: &AxleConfig,
    phys			: &PhysicsConfig,
    _ass			: &Res<AssetServer>,
    commands		: &mut Commands,
) -> Entity {
    let side_name	= wheel_side_name(side);
    let (sidez, sidex) = wheel_side_to_zx(side);

    let mut axle_id = Entity::from_bits(0);
    let 	axle_pos= body_pos * Transform::from_translation(offset);
    let		body_type = if phys.fixed { RigidBody::Fixed } else { RigidBody::Dynamic };

    axle_id 		=
    commands
        .spawn		()
        
        .insert		(*cfg)
        .insert		(*phys)

        .insert		(axle_pos)
        .insert		(GlobalTransform::default())

        .insert		(body_type)
        .insert		(MassProperties::default())
        .insert		(Damping::default())
        
        .insert		(NameComponent{ name: format!("{} Axle", side_name) })
        .insert		(Part::Axle)
        .insert		(sidex)
        .insert		(sidez)

        .with_children(|parent| {
            parent
            .spawn	()
            .insert	(Transform::default())
            .insert	(GlobalTransform::default())
            .insert	(Collider::cuboid(cfg.half_size.x, cfg.half_size.y, cfg.half_size.z))
            .insert	(ColliderMassProperties::Density(phys.density))
            .insert	(Friction::default())
            .insert	(Restitution::default());
        })
        .id			();

    // let axis_cube	= _ass.load("utils/axis_cube.gltf#Scene0");
    // commands.spawn_bundle(
    // 	TransformBundle {
    // 		local: axle_pos,
    // 		global: GlobalTransform::default(),
    // }).with_children(|parent| {
    // 	parent.spawn_scene(axis_cube);
    // });

    axle_id
}

fn spawn_wheel(
    side			: WheelSideType,
    axle_pos		: Transform,
    offset			: Vec3,
    cfg				: &WheelConfig,
    phys			: &PhysicsConfig,
    ass				: &Res<AssetServer>,
    commands		: &mut Commands,
) -> Entity {
    let side_name	= wheel_side_name(side);
    let (sidez, sidex) = wheel_side_to_zx(side);
    let mut wheel_id = Entity::from_bits(0);
    let body_type	= if phys.fixed { RigidBody::Fixed } else { RigidBody::Dynamic };

    let phys_rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2); // by default cylinder spawns with its flat surface on the ground and we want the round part
    let render_rotation = wheel_side_rotation(side);

    let wheel_pos 	= axle_pos * Transform::from_translation(offset);//local_pos;
    let model		= ass.load("corvette/wheel/corvette_wheel.gltf#Scene0");

    wheel_id 		=
    commands.spawn()
        .insert		(*cfg)
        .insert		(*phys)

        .insert		(wheel_pos)
        .insert		(GlobalTransform::default())
        // physics
        .insert		(body_type)
        .insert		(MassProperties::default())
        .insert		(Damping{ linear_damping: phys.lin_damping, angular_damping: phys.ang_damping })
        .insert		(NameComponent{ name: format!("{} Wheel", side_name) })
        .insert		(Part::Wheel)
        .insert		(sidex)
        .insert		(sidez)
        // collider
        .with_children(|parent| {
            parent.spawn()
            .insert	(Transform::from_rotation(phys_rotation))
            .insert (GlobalTransform::default())
            .insert	(Collider::cylinder(cfg.hh, cfg.r))
            .insert	(ColliderMassProperties::Density(phys.density))
            .insert	(Friction::new(phys.friction))
            .insert	(Restitution::new(phys.restitution));
    //			.insert	(ActiveEvents::COLLISION_EVENTS);
        })
        // render model
        .with_children(|parent| {
            parent.spawn()
            .insert	(Transform::from_rotation(render_rotation))
            .insert	(GlobalTransform::default())
            .with_children(|parent| {
                parent.spawn_scene(model);
            });
        })
        .id			();

    // let axis_cube	= ass.load("utils/axis_cube.gltf#Scene0");
    // commands.spawn_bundle(
    // 	TransformBundle {
    // 		local: wheel_pos,
    // 		global: GlobalTransform::default(),
    // }).with_children(|parent| {
    // 	parent.spawn_scene(axis_cube);
    // });

    wheel_id
}

fn spawn_axle_joint(
    entity1			: Entity,
    entity2			: Entity,
    anchor1			: Vec3,
    anchor2			: Vec3,
    commands		: &mut Commands,
) {
    let axle_joint = RevoluteJointBuilder::new(Vec3::Y)
        .local_anchor1(anchor1)
        .local_anchor2(anchor2)
        .limits		([0.000001, 0.000001]);

    commands
        .entity		(entity2)
        .insert		(ImpulseJoint::new(entity1, axle_joint));
}

fn spawn_wheel_joint(
    entity1			: Entity,
    entity2			: Entity,
    anchor1			: Vec3,
    anchor2			: Vec3,
    commands		: &mut Commands,
) {
    let wheel_joint = RevoluteJointBuilder::new(Vec3::X)
        .local_anchor1(anchor1)
        .local_anchor2(anchor2);

    commands
        .entity		(entity2)
        .insert		(ImpulseJoint::new(entity1, wheel_joint));
}

fn spawn_body(
    pos				: Transform,
    cfg				: &BodyConfig,
    phys			: &PhysicsConfig,
    accel_cfg		: &AcceleratorConfig,
    steer_cfg		: &SteeringConfig,
    ass				: &Res<AssetServer>,
    commands		: &mut Commands,
) -> Entity {
    let body_type	= if phys.fixed { RigidBody::Fixed } else { RigidBody::Dynamic };
    let half_size	= cfg.half_size;
    let density		= phys.density;

    let body_model	= ass.load("corvette/body/corvette_body.gltf#Scene0");

    commands
        .spawn		()
        .insert		(pos)
        .insert		(GlobalTransform::default())

        .insert		(*cfg)
        .insert		(*phys)
        .insert		(*accel_cfg)
        .insert		(*steer_cfg)

        .insert		(NameComponent{ name: "Body".to_string() })
        .insert		(Part::Body)
        .insert		(SideX::Center)
        .insert		(SideZ::Center)

        .insert		(body_type)
        .insert		(MassProperties::default())
        .insert		(Damping::default())
        // collider
        .with_children(|children| {
        children.spawn()
            .insert_bundle(TransformBundle::default())
            .insert	(Collider::cuboid(half_size.x, half_size.y, half_size.z))
            .insert	(ColliderMassProperties::Density(density)) // joints like it when there is an hierarchy of masses and we want body to be the heaviest
            .insert	(Friction::default())
            .insert	(Restitution::default());
        })
        // render model
        .with_children(|children| {
        children.spawn_bundle(
            TransformBundle {
                local: Transform::from_xyz(0.0, -1.0, 0.0),
                global: GlobalTransform::default(),
            }).with_children(|parent| {
                parent.spawn_scene(body_model);
            });
        })
        .id			()
}

pub fn respawn_vehicle_system(
	mut	game		: ResMut<GameState>,

		q_phys		: Query<&PhysicsConfig>,
	mut	q_body		: Query<(
		&	 BodyConfig,
		&	 PhysicsConfig,
		&mut Transform
	)>,
		q_accel_cfg	: Query<&AcceleratorConfig>,
		q_steer_cfg	: Query<&SteeringConfig>,
		q_axle_cfg	: Query<&AxleConfig>,
		q_wheel_cfg	: Query<&WheelConfig>,
	mut	q_camera	: Query<&mut FlyCamera>,
		ass			: Res<AssetServer>,
	mut	commands	: Commands,
) {
	let (mut body, respawn_body) = match game.body {
		Some(re)		=> (re.entity, re.respawn),
		_				=> return,
	};
	let (body_cfg, body_phys_cfg, mut body_pos) = q_body.get_mut(body).unwrap();
	let accel_cfg		= q_accel_cfg.get(body).unwrap();
	let steer_cfg		= q_steer_cfg.get(body).unwrap();

	if true == respawn_body {
		commands.entity(body).despawn_recursive();

		body_pos.translation = Vec3::new(0.0, 5.5, 0.0);
		body_pos.rotation = Quat::IDENTITY;
		body 			= spawn_body(*body_pos, body_cfg, body_phys_cfg, accel_cfg, steer_cfg, &ass, &mut commands);
		game.body 		= Some(RespawnableEntity { entity : body, ..Default::default() });
		// TODO: is there an event we can attach to? 
		let mut camera 	= q_camera.get_mut(game.camera.unwrap()).unwrap();
		camera.target 	= Some(body);
		println!		("camera.target Entity ID {:?}", camera.target);

		println!		("respawned body Entity ID {:?}", body);
	}

	for side_ref in WHEEL_SIDES {
		let side 		= *side_ref;
		let re_axle 	= game.axles[side].unwrap();
		let re_wheel	= game.wheels[side].unwrap();

		let mut axle	= re_axle.entity;
		let mut axle_pos : Transform;

		let axle_cfg	= q_axle_cfg.get(axle).unwrap().clone();
		let axle_phys_cfg = q_phys.get(axle).unwrap().clone();

		if !re_axle.respawn && !re_wheel.respawn && !respawn_body {
			continue;
		}

		commands.entity(axle).despawn_recursive();

		let axle_offset = body_cfg.axle_offset(side);
		(axle, axle_pos) = spawn_axle_with_joint(
			  side
			, body
			, *body_pos
			, axle_offset
			, &axle_cfg
			, &axle_phys_cfg
			, &ass
			, &mut commands
		);

		game.axles[side] = Some(RespawnableEntity{ entity : axle, respawn: false });

		println!		("respawned {} axle Entity ID {:?}", side, axle);
		
		let mut wheel	= re_wheel.entity;
		let wheel_cfg	= q_wheel_cfg.get(wheel).unwrap().clone();
		let wheel_phys_cfg = q_phys.get(wheel).unwrap().clone();

		commands.entity(wheel).despawn_recursive();

		let wheel_offset = axle_cfg.wheel_offset(side);
		wheel = spawn_wheel_with_joint(
			  side
			, axle
			, axle_pos
			, wheel_offset
			, &wheel_cfg
			, &wheel_phys_cfg
			, &ass
			, &mut commands
		);

		game.wheels[side] = Some(RespawnableEntity{ entity : wheel, respawn: false });

		println!		("respawned {} wheel Entity ID {:?}", side, wheel);
	}
}