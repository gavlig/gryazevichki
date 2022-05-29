use bevy			::	prelude :: *;
use bevy_rapier3d	::	prelude :: *;
use bevy_egui		::	egui :: { Slider, Ui };

use super			::	{ * };

pub fn cylinder_hh(
	new_hh          		: f32,
	shared_shape    		: &mut Mut<Collider>,
) {
	let 	shape 			= shared_shape.raw.make_mut();
	let mut cylinder		= shape.as_cylinder_mut().unwrap();
	cylinder.half_height 	= new_hh;
}

pub fn cylinder_r(
	new_r					: f32,
	shared_shape			: &mut Mut<Collider>,
) {
	let 	shape 			= shared_shape.raw.make_mut();
	let mut cylinder		= shape.as_cylinder_mut().unwrap();
	cylinder.radius 		= new_r;
}

pub fn box_half_size(
	new_hs					: Vec3,
	shared_shape			: &mut Mut<Collider>,
) {
	let 	shape 			= shared_shape.raw.make_mut();
	let mut cuboid			= shape.as_cuboid_mut().unwrap();
	cuboid.half_extents     = new_hs.into();
}

pub fn density(
		density_in			: f32,
	mut mass_props_co		: &mut Mut<ColliderMassProperties>,
) {
	match &mut mass_props_co as &mut ColliderMassProperties {
		ColliderMassProperties::Density(density) => {
			*density 		= density_in;
			**mass_props_co = ColliderMassProperties::Density(*density);
		},
		ColliderMassProperties::MassProperties(_) => (),
	};
}

pub fn friction(
		friction_in			: f32,
		friction			: &mut Mut<Friction>,
) {
	friction.as_mut().coefficient = friction_in;
}

pub fn restitution(
		restitution_in		: f32,
		restitution			: &mut Mut<Restitution>,
) {
	restitution.as_mut().coefficient = restitution_in;
}

pub fn damping(
		lin_damping_in		: f32,
		ang_damping_in		: f32,
		damping				: &mut Mut<Damping>,
) {
	damping.as_mut().linear_damping = lin_damping_in;
	damping.as_mut().angular_damping = ang_damping_in;
}