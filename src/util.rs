use bevy :: prelude :: *;

use bevy :: render :: { camera :: { * }, primitives :: Frustum };

use super :: TextDescriptor;

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

pub fn calc_visible_rows(
	camera_transform	: &Transform,
	camera_projection	: &Projection,
	text_descriptor		: &TextDescriptor,
	target_transform	: &Transform,
) -> f32 {
	// we remove x and y since the amount of visible rows should depend on how much we scrolled, just how far the camera is from the surface with text
	let mut camera_transform_z_only = camera_transform.clone();
	camera_transform_z_only.translation.x = 0.0;
	camera_transform_z_only.translation.y = 0.0;
	
	// calculating frustum manually for now because using cached introduces small desync between frustum and camera position
	let projection_matrix = camera_projection.get_projection_matrix() * camera_transform_z_only.compute_matrix().inverse();
	let frustum = Frustum::from_view_projection(
		&projection_matrix,
		&camera_transform_z_only.translation,
		&camera_transform_z_only.back(),
		camera_projection.far(),
	);
	
	// in frustum:
	// plane 0 is left
	// plane 1 is right
	// plane 2 is bottom
	// plane 3 is top
	// plane 4 is near (or back)
	// plane 5 is far
	
	// plane equation by three vertices
	// Ax + By + Cz + D = 0
	
	let row_height = text_descriptor.glyph_height;
	let _column_width = text_descriptor.glyph_width;
	
	let z = target_transform.translation.z + 0.05;
	
	let plane_top = &frustum.planes[3].normal_d();
	let y_top = (-plane_top.w - plane_top.z * z) / plane_top.y;
	
	let plane_bottom = &frustum.planes[2].normal_d();
	let y_bottom = (-plane_bottom.w - plane_bottom.z * z) / plane_bottom.y;
	
	(y_top - y_bottom) / row_height
}