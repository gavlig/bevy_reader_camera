use bevy :: prelude :: *;

use bevy :: render	:: { camera :: { * }, primitives :: Frustum };
use bevy :: input	:: mouse :: { MouseScrollUnit, MouseWheel, MouseMotion };

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

pub fn calc_frustum(
	camera_transform	: &Transform,
	camera_projection	: &Projection,
) -> Frustum {
	let projection_matrix = camera_projection.get_projection_matrix() * camera_transform.compute_matrix().inverse();
	Frustum::from_view_projection_custom_far(
		&projection_matrix,
		&camera_transform.translation,
		&camera_transform.back(),
		camera_projection.far(),
	)
}

// in bevy frustum:
// plane 0 is left
// plane 1 is right
// plane 2 is bottom
// plane 3 is top
// plane 4 is near (or back)
// plane 5 is far

pub enum FrustumPlane {
	Left	= 0,
	Right	= 1,
	Bottom	= 2,
	Top		= 3,
//	Near	= 4,
//	Far		= 5
}

// plane equation by three vertices
// Ax + By + Cz + D = 0

pub fn calc_frustum_y_border(frustum: &Frustum, z_in: f32, top: bool) -> f32 {
	let z = z_in + 0.05; // FIXME: add z offset for glyphs as a camera parameter? maybe to text descriptor

	let plane_index = if top { FrustumPlane::Top } else { FrustumPlane::Bottom } as usize;

	let plane = &frustum.planes[plane_index].normal_d();

	(-plane.w - plane.z * z) / plane.y // assume x = 0
}

pub fn calc_frustum_x_border(frustum: &Frustum, z_in: f32, right: bool) -> f32 {
	let z = z_in + 0.05; // FIXME: add z offset for glyphs as a camera parameter? maybe to text descriptor

	let plane_index = if right { FrustumPlane::Right } else { FrustumPlane::Left } as usize;

	let plane = &frustum.planes[plane_index].normal_d();

	(-plane.w - plane.z * z) / plane.x // assume y = 0
}

pub fn calc_visible_rows(
	frustum				: &Frustum,
	target_z			: f32,
	row_height			: f32,
) -> f32 {
	let y_top			= calc_frustum_y_border(frustum, target_z, true);
	let y_bottom		= calc_frustum_y_border(frustum, target_z, false);

	(y_top - y_bottom) / row_height
}

pub fn calc_visible_columns(
	frustum				: &Frustum,
	target_z			: f32,
	column_width		: f32,
) -> f32 {
	let x_right			= calc_frustum_x_border(frustum, target_z, true);
	let x_left			= calc_frustum_x_border(frustum, target_z, false);

	(x_right - x_left) / column_width
}

pub fn delta_wheel_from_events(
	pixels_per_line					: f32,
	mut mouse_wheel_event_reader	: EventReader<MouseWheel>,
) -> Option<f32> {
	let scroll_event_occured = !mouse_wheel_event_reader.is_empty();
	let mut wheel_scalar	= 0.0;
	for event in mouse_wheel_event_reader.iter() {
		// scale the event magnitude per pixel or per line
		let scroll_amount = match event.unit {
			MouseScrollUnit::Line => { event.y },
			MouseScrollUnit::Pixel => { event.y / pixels_per_line },
		};
		wheel_scalar += -scroll_amount;
	}

	if scroll_event_occured { Some(wheel_scalar) } else { None }
}

pub fn delta_mouse_from_events(
	mut mouse_motion_event_reader	: EventReader<MouseMotion>
) -> Vec2 {
	let mut delta_mouse: Vec2 = Vec2::ZERO;
	for event in mouse_motion_event_reader.iter() {
		delta_mouse += event.delta;
	}

	delta_mouse
}