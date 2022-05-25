use bevy			::	prelude :: *;
use bevy_rapier3d	::	prelude :: *;
use rapier3d		::	dynamics :: { JointAxis };

use super			::	{ RespawnableEntity };

pub fn velocity(
	velocity		: f32,
	factor			: f32,
	joint_e			: Option<RespawnableEntity>,
	query			: &mut Query<&mut ImpulseJoint>
) {
	match joint_e {
		Some(j) => {
			let mut	joint	= query.get_mut(j.entity).unwrap();
			joint.data.set_motor_velocity(JointAxis::AngX, velocity, factor);
		}
		_ => ()
	}
}

pub fn steer(
	angle			: f32,
	stiffness		: f32,
	damping			: f32,
	joint_re		: Option<RespawnableEntity>,
	query			: &mut Query<&mut ImpulseJoint>) {
	match joint_re {
		Some(re) => {
			let mut joint 	= query.get_mut(re.entity).unwrap();
			let	angle_rad	= angle.to_radians();
			joint.data.set_motor_position(JointAxis::AngX, angle_rad, stiffness, damping);
			
			if angle.abs() > 0.0001 {
				joint.data.set_limits(JointAxis::AngX, [-angle_rad.abs(), angle_rad.abs()]);// [-3.14, 3.14]);
			}
		}
		_ => ()
	}
}
