use bevy :: prelude :: *;
use bevy :: render :: primitives :: { Sphere, Frustum };

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

pub fn binary_search_visible_rows(
	center			: Vec3,
	frustum			: &Frustum,
	text_desc		: &TextDescriptor,
) -> u32
{
	let mut half_offset		= 20 as u32; // assume we have 40 rows visible by default
	
	let row_height	= text_desc.glyph_height;
	let mut half_offset_min	= 1 as u32;
	let mut half_offset_max	= 5000 as u32; // not usable but could be pretty to render
	
	while half_offset_min <= half_offset_max {
		let mut sphere_from_glyph = Sphere {
			center: (center + (Vec3::Y * row_height * half_offset as f32)).into(),
			radius: row_height,
		};
		
		let is_inside = frustum.intersects_sphere(&sphere_from_glyph, /*intersect_far=*/false);
		
		if is_inside {
			// check one row above, if it's outside frustum we found the border
			sphere_from_glyph.center.y += row_height;
			if !frustum.intersects_sphere(&sphere_from_glyph, /*intersect_far=*/false) {
				return half_offset * 2;
			} else {
				half_offset_min = half_offset + 1;
			}
		} else {
			// check one row below, if it's inside frustum we found the border
			sphere_from_glyph.center.y -= row_height;
			if frustum.intersects_sphere(&sphere_from_glyph, /*intersect_far=*/false) {
				return half_offset * 2;
			} else {
				half_offset_max = half_offset - 1;
			}
		}
		
		half_offset = half_offset_min + (half_offset_max - half_offset_min) / 2;
	}
	
	return 1;
}