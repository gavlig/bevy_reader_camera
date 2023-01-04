use bevy :: prelude :: *;

pub fn movement_axis(
	input: &Res<Input<KeyCode>>,
	plus: KeyCode,
	minus: KeyCode,
) -> f32 {
	let mut axis = 0.0;
	if input.pressed(plus) {
		axis += 1.0;
	}
	if input.pressed(minus) {
		axis -= 1.0;
	}
	axis
}

pub fn forward_vector(rotation: &Quat) -> Vec3 {
	rotation.mul_vec3(Vec3::Z).normalize()
}

pub fn forward_walk_vector(rotation: &Quat) -> Vec3 {
	let f = forward_vector(rotation);
	let f_flattened = Vec3::new(f.x, 0.0, f.z).normalize();
	f_flattened
}

pub fn strafe_vector(rotation: &Quat) -> Vec3 {
	// Rotate it 90 degrees to get the strafe direction
	Quat::from_rotation_y(90.0f32.to_radians())
		.mul_vec3(forward_walk_vector(rotation))
		.normalize()
}

// thanks smooth-bevy-cameras and Dunkan!
pub fn unit_vector_from_yaw_and_pitch(yaw: f32, pitch: f32) -> Vec3 {
    let ray = Mat3::from_rotation_y(yaw) * Vec3::Z;
    let pitch_axis = ray.cross(Vec3::Y);

    Mat3::from_axis_angle(pitch_axis, pitch) * ray
}