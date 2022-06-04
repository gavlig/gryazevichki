use bevy			:: { prelude :: * };
use bevy			:: { app::AppExit };
use bevy			:: { input :: mouse :: * };
use bevy_rapier3d	:: { prelude :: * };
use bevy_fly_camera	:: { FlyCamera };
use bevy_mod_picking:: { * };
use bevy_mod_raycast:: { * };
use bevy_polyline	:: { prelude :: * };
use bevy_prototype_debug_lines :: { * };
use iyes_loopless	:: { prelude :: * };
use bevy_debug_text_overlay :: { screen_print };

use std				:: { path::PathBuf };
use splines			:: { Interpolation, Key, Spline };

use bevy::render::mesh::shape as render_shape;
use std::f32::consts:: { * };

use super           :: { * };

pub mod spawn;

#[derive(Component)]
pub struct Tile;

#[derive(Component)]
pub struct Herringbone;

pub struct StepRequest {
	pub next			: bool,
	pub animate			: bool,
	pub reset			: bool,
	pub instant			: bool,
	pub last_update		: f64,
	pub anim_delay_sec	: f64,
}

impl Default for StepRequest {
	fn default() -> Self {
		Self {
			next		: false,
			animate		: false,
			reset		: false,
			instant		: false,
			last_update	: 0.0,
			anim_delay_sec: 0.001,
		}
	}
}

#[derive(Component)]
pub struct IO {
	// going to change
	pub x 				: u32,
	pub z 				: u32,
	pub iter			: u32,
	pub orientation		: Orientation2D,
	pub finished_hor	: bool,
	pub finished		: bool,
	pub transform		: Transform,
	pub parent			: Option<Entity>,

	pub spline			: Option<Spline<f32, Vec3>>,
	pub prev_spline_p	: Option<Vec3>,

	// read only - ish
	pub body_type 		: RigidBody,
	pub hsize 			: Vec3,
	pub seam			: f32,
	
	// read only
	pub width			: f32,
	pub length			: f32,
	pub limit			: u32,

	// cant copy
	pub mesh			: Handle<Mesh>,
	pub material		: Handle<StandardMaterial>,
}

impl Default for IO {
	fn default() -> Self {
		Self {
			x 			: 0,
			z 			: 0,
			iter		: 0,
			orientation	: Orientation2D::Horizontal,
			finished_hor: false,
			finished	: false,
			transform	: Transform::identity(),
			parent		: None,

			spline		: None,
			prev_spline_p: None,

			body_type 	: RigidBody::Fixed,
			hsize 		: Vec3::ZERO,
			seam		: 0.01,
			width		: 0.0,
			length		: 0.0,
			limit		: 0,

			mesh		: Handle::<Mesh>::default(),
			material	: Handle::<StandardMaterial>::default(),
		}
	}
}

impl IO {
	pub fn set_default(&mut self) {
		*self			= Self::default();
	}

	pub fn reset_changed(&mut self) {
		self.iter 		= 0;
		self.x 			= 0;
		self.z 			= 0;
		self.finished 	= false;
		self.finished_hor = false;
	}

	pub fn clone(&self) -> Self {
		Self {
			x 			: self.x,
			z 			: self.z,
			iter		: self.iter,
			orientation	: self.orientation,
			finished_hor: self.finished_hor,
			finished	: self.finished,
			transform	: self.transform,
			parent		: self.parent,

			spline		: self.spline.clone(),
			prev_spline_p: self.prev_spline_p.clone(),

			body_type 	: self.body_type,
			hsize 		: self.hsize,
			seam		: self.seam,
			
			width		: self.width,
			length		: self.length,
			limit		: self.limit,

			mesh		: self.mesh.clone_weak(),
			material	: self.material.clone_weak(),
		}
	}

	pub fn set_spline_interpolation(&mut self, id : usize, interpolation : Interpolation<f32, Vec3>) { // TODO: declare Interpolation<f32, Vec3> somewhere?
		*self.spline.as_mut().unwrap().get_mut(id).unwrap().interpolation = interpolation;
	}

	pub fn set_spline_control_point(&mut self, id : usize, controlp_pos : Vec3) { // TODO: declare Key<f32, Vec3> somewhere?
		let t = controlp_pos.z;
		self.spline.as_mut().unwrap().replace(id, |k : &Key<f32, Vec3>| { Key::new(t, controlp_pos, k.interpolation) });
	}
}

pub fn brick_road_iter(
	mut io				: &mut IO,
		ass				: &Res<AssetServer>,
		commands		: &mut Commands
) {
	let init_rotation	= match io.orientation {
	Orientation2D::Horizontal 	=> Quat::from_rotation_y(FRAC_PI_2),
	Orientation2D::Vertical 	=> Quat::IDENTITY,
	};

	let seam			= io.seam;

	let hlenz			= io.hsize.z;
	let lenz			= hlenz * 2.0;

	let hlenx			= io.hsize.x;
	let lenx			= hlenx * 2.0;

	// main tile center calculation without seams
	//
	//

	let iter0			= (io.iter + 0) as f32;
	let iter1			= (io.iter + 1) as f32;

	let calc_offset_x = |x : f32, iter : f32, orientation : Orientation2D| -> f32 {
		match orientation {
		Orientation2D::Horizontal 	=> (iter + 1.0) * hlenz 				+ (x * (lenz * 2.0)),
		Orientation2D::Vertical 	=> (iter + 0.0) * hlenz + (hlenx * 1.0)	+ (x * (lenz * 2.0)),
		}
	};

	let calc_offset_z = |z : f32, iter : f32, orientation : Orientation2D| -> f32 {
		match orientation {
		Orientation2D::Horizontal 	=> (iter + 0.0) * hlenz + (hlenx * 1.0)	+ (z * (lenz * 2.0)),
		Orientation2D::Vertical 	=> (iter + 0.0) * hlenz + (hlenz * 2.0) + (z * (lenz * 2.0)),
		}
	};

	let offset_x 		= calc_offset_x(io.x as f32, iter0, io.orientation);
	let offset_z 		= calc_offset_z(io.z as f32, iter0, io.orientation);

	// now seams are tricky
	//
	//

	let calc_seam_offset_x = |x : f32, z : f32, iter : f32, orientation : Orientation2D, seam: f32| -> f32 {
		let mut offset_x = ((iter + 0.0) * seam) + ((x + 0.0) * seam * 3.0);

		if Orientation2D::Horizontal == orientation && z > 0.0 {
			offset_x 	+= seam * 0.5;
		}

		offset_x
	};

	let calc_seam_offset_z = |z : f32, iter : f32, orientation : Orientation2D, seam: f32| -> f32 {
		let mut offset_z = ((iter + 0.0) * seam) + ((z + 0.0) * seam * 3.0);

		if Orientation2D::Vertical == orientation {
			offset_z 	+= seam * 1.5;
		}
		offset_z 		+= (z + 0.0) * seam * 0.5;

		offset_z
	};

	let seam_offset_x 	= calc_seam_offset_x(io.x as f32, io.z as f32, iter0, io.orientation, seam);
	let seam_offset_z 	= calc_seam_offset_z(io.z as f32, iter0, io.orientation, seam);

	// now let me interject for a moment with a spline (Hi Freya!)
	//
	//

	// println!			("no spline {} x = {} z = {} offx {:.2} offz {:.2} {:?} body_type: {:?}", io.iter, io.x, io.z, offset_x + seam_offset_x, offset_z + seam_offset_z, io.orientation, io.body_type);

	// spline is in the same local space as each brick is
	let t				= offset_z + seam_offset_z;
	let spline_p		= match io.spline.as_ref() {
		// ok, we have a spline, sample it
		Some(spline)	=> match spline.sample(t) {

		// ok, sample was a success, get the point from it
		Some(p)			=> p,
		// sample wasnt a succes, try previuos point on spline
		None			=> {
			match io.prev_spline_p {
				Some(p)	=> p,
				None	=> Vec3::ZERO,
			}
		},
		},
		// there is no spline, no offset
		None			=> Vec3::ZERO,
	};
	let spline_r		= match io.prev_spline_p {
		Some(prev_spline_p) => {
			let spline_dir	= (spline_p - prev_spline_p).normalize();
			Quat::from_rotation_arc(Vec3::Z, spline_dir)
		},
		None => Quat::IDENTITY,
	};

	io.prev_spline_p	= Some(spline_p);

	// Final pose
	//
	//

	let mut pose 		= Transform::identity();

	// half width shift to appear in the middle
	pose.translation.x	-= io.width / 2.0;

	// tile offset/rotation
	pose.translation.x	+= offset_x + seam_offset_x;
	pose.translation.z	+= offset_z + seam_offset_z;
	pose.rotation		*= init_rotation;

	// spline
	pose.translation.x	+= spline_p.x;
	// spline is sampled by z so it doesnt bring any offset on z

	pose.rotation		*= spline_r;

	// spawn
	//
	//

	// spawn first brick with a strong reference to keep reference count > 0 and mesh/material from dying when out of scope
	let (mut me, mut ma) = (io.mesh.clone_weak(), io.material.clone_weak());
	match (io.x, io.z) {
		(0, 0) => {
			(me, ma)	= (io.mesh.clone(), io.material.clone());
		}
		_ => (),
	}

	{
		macro_rules! insert_tile_components {
			($a:expr) => {
				$a	
				.insert			(io.body_type)
				.insert			(io.transform * pose)
				.insert			(GlobalTransform::default())
				.insert			(Collider::cuboid(io.hsize.x, io.hsize.y, io.hsize.z))
				// .insert			(Friction{ coefficient : friction, combine_rule : CoefficientCombineRule::Average });
				.insert_bundle	(PickableBundle::default())
				// .insert			(Draggable::default())
				.insert			(Herringbone)
				.insert			(Tile)
				.insert			(io.clone());
			}
		}

		let bundle = PbrBundle{ mesh: me, material: ma, ..default() };

		if io.parent.is_some() {
			commands.entity(io.parent.unwrap()).with_children(|parent| {
				insert_tile_components!(parent.spawn_bundle(bundle));
			});
		} else {
			insert_tile_components!(commands.spawn_bundle(bundle));
		}
	}

	// if only io.limit is given set limits in cordinates anyway because otherwise we don't know where to stop not on diagonal
	if io.iter == io.limit {
		if io.width == 0.0 {
			io.width = offset_x;
		}

		if io.length == 0.0 {
			io.length = offset_z;
		}
	}

	// check for end conditions
	//
	//

	let newoffx	= calc_offset_x		(io.x as f32, iter1, io.orientation) 
				+ calc_seam_offset_x(io.x as f32, io.z as f32, iter1, io.orientation, seam);

	let newoffz	= calc_offset_z		(io.z as f32, iter1, io.orientation)
				+ calc_seam_offset_z(io.z as f32, iter1, io.orientation, seam);

	if ((newoffx >= io.width) && (io.width != 0.0))
	|| ((newoffz >= io.length) && (io.length != 0.0))
	|| (io.iter >= io.limit && io.limit != 0)
	{
		let prev_orientation = io.orientation.clone();

		io.iter			= 0;
		io.orientation.flip();

		// println!		("Flipped orientation x_limit: {} z_limit: {} limit: {}", io.x_limit, io.z_limit, io.limit);

		if prev_orientation == Orientation2D::Vertical {
			let newoffx	= calc_offset_x		((io.x + 1) as f32, io.iter as f32, io.orientation) 
						+ calc_seam_offset_x((io.x + 1) as f32, io.z as f32, io.iter as f32, io.orientation, seam);

			let newoffz	= calc_offset_z		((io.z + 1) as f32, io.iter as f32, io.orientation)
						+ calc_seam_offset_z((io.z + 1) as f32, io.iter as f32, io.orientation, seam);

			if newoffx < io.width && !io.finished_hor {
				io.x	+= 1;
				// println!("x =+ 1 new offx {:.3}", newoffx);
			} else if newoffz < io.length {
				io.x	= 0;
				io.z	+= 1;
				io.finished_hor = true;
				// println!("x = 0, z += 1 new offz {:.3}", newoffz);
			} else {
				io.finished = true;
				// println!("herringbone_brick_road_iter finished!");
			}
		}
	}

	io.iter				+= 1;
}

pub fn brick_road_setup(
	io					: &mut ResMut<IO>,
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	ass					: &Res<AssetServer>,
	mut commands		: &mut Commands
) {
//	let hsize 			= Vec3::new(0.1075 / 2.0, 0.065 / 2.0, 0.215 / 2.0);
	let hsize 			= Vec3::new(0.2 / 2.0, 0.05 / 2.0, 0.4 / 2.0);
//	let hsize 			= Vec3::new(0.1075, 0.065, 0.215);

	io.set_default		();

	io.width			= 10.0;
	io.length			= 10.0;
	io.limit			= 100;

	io.body_type		= RigidBody::Fixed;
	io.hsize			= hsize;
	io.seam				= 0.01;
	io.mesh				= meshes.add(Mesh::from(render_shape::Box::new(hsize.x * 2.0, hsize.y * 2.0, hsize.z * 2.0)));
	io.material			= materials.add(StandardMaterial { base_color: Color::ALICE_BLUE,..default() });

	let y_offset		= 0.5;
	
	// spline requires at least 4 points: 2 control points(Key) and 2 tangents
	//
	//
	let tangent0		= Vec3::new(0.0, y_offset, 2.5);
	let tangent1		= Vec3::new(0.0, y_offset, 7.5);
	// z_limit is used both for final coordinate and for final value of t to have road length tied to spline length and vice versa
	let control_point0_pos = Vec3::new(0.0, y_offset, 0.0);
	let control_point1_pos = Vec3::new(0.0, y_offset, io.length);
	// z_limit as final 't' value lets us probe spline from any z offset of a tile
	let t0				= 0.0;
	let t1				= io.length;

	let control_point0	= Key::new(t0, control_point0_pos, Interpolation::StrokeBezier(tangent0, tangent0));
	let control_point1	= Key::new(t1, control_point1_pos, Interpolation::StrokeBezier(tangent1, tangent1));

	io.spline 			= Some(Spline::from_vec(vec![control_point0, control_point1]));
}

pub fn brick_road_system(
	mut step			: ResMut<StepRequest>,
	mut io				: ResMut<IO>,
	mut despawn			: ResMut<DespawnResource>,
		time			: Res<Time>,
		ass				: Res<AssetServer>,

		query			: Query<Entity, With<Herringbone>>,

	mut commands		: Commands
) {
	if step.reset {
		for e in query.iter() {
			despawn.entities.push(e);
		}
	
		io.reset_changed();
	
		step.reset		= false;
	}

	let do_spawn 		= step.next || step.animate;
	if !do_spawn || io.finished {
		return;
	}

	let cur_time		= time.seconds_since_startup();
	if (cur_time - step.last_update) < step.anim_delay_sec && !step.instant {
		return;
	}

	step.last_update 	= cur_time;

	if !step.instant {
		brick_road_iter(&mut io, &ass, &mut commands);
	} else {
		loop {
			brick_road_iter(&mut io, &ass, &mut commands);
			if io.finished {
				step.instant = false;
				break;
			}
		}
	}

	step.next			= false;

	if io.finished {
		step.animate	= false;
	}
}

pub fn on_spline_tangent_moved(
	mut step			: ResMut<StepRequest>,
	mut io				: ResMut<IO>,
		time			: Res<Time>,
		q_tangent 		: Query<(&Transform, &SplineTangent), Changed<Transform>>,
) {
	if time.seconds_since_startup() < 1.0 {
		return;
	}

	for (tform, tan) in q_tangent.iter() {
		let tan_pos		= tform.translation;
		match tan {
			SplineTangent::ID(id) => {
				io.set_spline_interpolation(*id, Interpolation::StrokeBezier(tan_pos, tan_pos));
				step.reset = true;
				step.next = true;
				step.instant = true;
			},
		}
	}
}

pub fn on_spline_control_point_moved(
	mut step			: ResMut<StepRequest>,
	mut io				: ResMut<IO>,
		time			: Res<Time>,
		q_controlp 		: Query<(&Transform, &SplineControlPoint), Changed<Transform>>,
) {
	if time.seconds_since_startup() < 1.0 {
		return;
	}

	for (tform, controlp) in q_controlp.iter() {
		let controlp_pos = tform.translation;
		match controlp {
			SplineControlPoint::ID(id) => {
				io.set_spline_control_point(*id, controlp_pos);

				// io.x_limit = 

				step.reset = true;
				step.next = true;
				step.instant = true;
			},
		}
	}
}

pub fn on_object_root_moved(
	mut step			: ResMut<StepRequest>,
	mut io				: ResMut<IO>,
		time			: Res<Time>,
		q_root			: Query<&Transform, (With<ObjectRoot>, Changed<Transform>)>,
) {
	if time.seconds_since_startup() < 1.0 {
		return;
	}

	if q_root.is_empty() {
		return;
	}

	let root_pos		= match q_root.get_single() {
		Ok(pos)			=> *pos,
		Err(_)			=> Transform::identity(),
	};

	step.reset 			= true;
	step.next 			= true;
	step.instant 		= true;
}