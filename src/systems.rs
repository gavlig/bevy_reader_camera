use bevy :: {
	prelude	:: { * },
	input	:: mouse :: { MouseMotion, MouseScrollUnit, MouseWheel },
	render	:: { camera :: { * }, primitives :: Frustum }
};

use lerp :: Lerp;

use super :: CameraMode;
use super :: TextDescriptor;
use super :: reader_camera :: { * };
use super :: util :: { * };

pub fn keyboard_fly(
		time		: Res<Time>,
		key_code	: Res<Input<KeyCode>>,
	mut q_camera	: Query<(&mut ReaderCamera, &mut Transform, &mut Projection)>,
)
{
	for (mut camera, mut camera_transform, mut projection) in q_camera.iter_mut() {
		if camera.mode != CameraMode::Fly || !camera.enabled_translation {
			continue;
		}

		let (axis_h, axis_v, axis_float) = (
			movement_axis(&key_code, camera.key_right,		camera.key_left),
			movement_axis(&key_code, camera.key_backward,	camera.key_forward),
			movement_axis(&key_code, camera.key_up,			camera.key_down),
		);

		let modper = camera.mod_perspective;
		let perspective_mod = (modper.is_some() && key_code.pressed(modper.unwrap())) || modper.is_none();
		if perspective_mod && key_code.just_pressed(camera.key_perspective) {
			let toggle 	= !camera.perspective;
			camera.perspective = toggle;

			*projection = 
			if camera.perspective {
				Projection::Perspective(PerspectiveProjection::default())
			} else {
				camera.yaw = 0.0;
				camera.pitch = 0.0;

				Projection::Orthographic(
				OrthographicProjection {
					scale: 3.0,
					scaling_mode: ScalingMode::FixedVertical(2.0),
					..default()
				
				})
			}
		}

		let rotation = camera_transform.rotation;
		let accel: Vec3 = (strafe_vector(&rotation) * axis_h)
			+ (forward_walk_vector(&rotation) * axis_v)
			+ (Vec3::Y * axis_float);
		let accel: Vec3 = if accel.length() != 0.0 {
			accel.normalize() * camera.accel
		} else {
			Vec3::ZERO
		};

		let friction: Vec3 = if camera.velocity.length() != 0.0 {
			camera.velocity.normalize() * -1.0 * camera.friction
		} else {
			Vec3::ZERO
		};

		camera.velocity += accel * time.delta_seconds();

		// clamp within max speed
		if camera.velocity.length() > camera.max_speed {
			camera.velocity = camera.velocity.normalize() * camera.max_speed;
		}

		let delta_friction = friction * time.delta_seconds();

		camera.velocity = if (camera.velocity + delta_friction).signum()
			!= camera.velocity.signum()
		{
			Vec3::ZERO
		} else {
			camera.velocity + delta_friction
		};

		camera_transform.translation += camera.velocity;
	}
}

pub fn mouse_fly(
		time: Res<Time>,
	mut mouse_motion_event_reader: EventReader<MouseMotion>,
	mut q_camera: Query<(&mut ReaderCamera, &mut Transform)>,
)
{
	let mut delta: Vec2 = Vec2::ZERO;
	for event in mouse_motion_event_reader.iter() {
		delta += event.delta;
	}
	if delta.is_nan() {
		return;
	}

	for (mut camera, mut transform) in q_camera.iter_mut() {
		if camera.mode != CameraMode::Fly || !camera.enabled_rotation {
			continue;
		}
		camera.yaw -= delta.x * camera.sensitivity * time.delta_seconds();
		camera.pitch += delta.y * camera.sensitivity * time.delta_seconds();

		camera.pitch = camera.pitch.clamp(-89.0, 89.9);

		let yaw_radians = camera.yaw.to_radians();
		let pitch_radians = camera.pitch.to_radians();

		transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw_radians)
			* Quat::from_axis_angle(-Vec3::X, pitch_radians);
	}
}

pub fn mouse_follow(
		time						: Res<Time>,
	mut mouse_motion_event_reader	: EventReader<MouseMotion>,
	mut mouse_wheel_event_reader	: EventReader<MouseWheel>,
	mut q_camera					: Query<(&mut ReaderCamera, &mut Transform)>,
		q_target					: Query<&Transform, Without<ReaderCamera>>,
)
{
	for (mut camera, mut camera_transform) in q_camera.iter_mut() {
		if camera.mode != CameraMode::Follow {
			continue
		}
		assert!(camera.target != None);

		let mut delta: Vec2 = Vec2::ZERO;
		for event in mouse_motion_event_reader.iter() {
			delta += event.delta;
		}
		if delta.is_nan() {
			continue;
		}

		if camera.enabled_rotation {
			camera.yaw -= delta.x * camera.sensitivity * time.delta_seconds();
			camera.pitch += delta.y * camera.sensitivity * time.delta_seconds();

			camera.pitch = camera.pitch.clamp(-89.0, 89.9);
		}

		let yaw_radians = camera.yaw.to_radians();
		let pitch_radians = camera.pitch.to_radians();

		//

		let pixels_per_line = 53.0;
		let mut scalar = 1.0;
		for event in mouse_wheel_event_reader.iter() {
			// scale the event magnitude per pixel or per line
			let scroll_amount = match event.unit {
				MouseScrollUnit::Line => event.y,
				MouseScrollUnit::Pixel => event.y / pixels_per_line,
			};
			scalar *= 1.0 - scroll_amount * camera.zoom_sensitivity;
		}

		if camera.enabled_zoom {
			camera.zoom = (scalar * camera.zoom)
				.min(100.0)
				.max(1.0);
		}

		//
		if camera.enabled_translation {
			let target = camera.target.unwrap();
			let target_transform = q_target.get(target).unwrap();

			camera_transform.translation = target_transform.translation + camera.zoom * unit_vector_from_yaw_and_pitch(yaw_radians, pitch_radians);
		}

		if camera.enabled_rotation {
			camera_transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw_radians) * Quat::from_axis_angle(-Vec3::X, pitch_radians);
		}
	}
}

pub fn mouse_reader(
		time						: Res<Time>,
	mut mouse_motion_event_reader	: EventReader<MouseMotion>,
	mut mouse_wheel_event_reader	: EventReader<MouseWheel>,
	mut q_camera					: Query<(&mut ReaderCamera, &mut Transform)>,
		q_target					: Query<(&Transform, &TextDescriptor), Without<ReaderCamera>>,
)
{
	let mut delta: Vec2 = Vec2::ZERO;
	for event in mouse_motion_event_reader.iter() {
		delta += event.delta;
	}
	if delta.is_nan() {
		return;
	}

	let delta_seconds = time.delta_seconds();

	for (mut camera, mut camera_transform) in q_camera.iter_mut() {
		if camera.mode != CameraMode::Reader {
			continue;
		}
		
		{ // zoom
			let mut scalar = 0.0;

			if camera.enabled_zoom {
				let pixels_per_line = 53.0;
				for event in mouse_wheel_event_reader.iter() {
					// scale the event magnitude per pixel or per line
					let scroll_amount = match event.unit {
						MouseScrollUnit::Line => { event.y },
						MouseScrollUnit::Pixel => { event.y / pixels_per_line },
					};
					scalar = -scroll_amount * camera.zoom_sensitivity;
				}
			}

			let inertia = (delta_seconds / camera.zoom_easing_seconds).min(1.0);
			camera.target_zoom = (scalar + camera.target_zoom)
				.min(100.0)
				.max(3.0);

			camera.zoom = camera.zoom.lerp(camera.target_zoom, inertia);
		}

		let yaw_radians = camera.yaw.to_radians();
		let pitch_radians = camera.pitch.to_radians();

		// translation
		if let Some(target) = camera.target { 
			let (target_transform, text_descriptor) = q_target.get(target).unwrap();

			if camera.enabled_translation {
				let delta_x = delta.x * camera.swipe_sensitivity;
				let delta_y = delta.y * camera.scroll_sensitivity;

				camera.row_scroll_accum += delta_y * (delta_seconds / camera.scroll_easing_seconds);
				camera.column_scroll_accum += delta_x * (delta_seconds / camera.swipe_easing_seconds);
			}

			// we keep row_scroll_accum in range of 0..glyph_height
			while camera.row_scroll_accum.abs() > text_descriptor.glyph_height {
				let delta_one = delta.y.signum();
				if camera.enabled_translation && (camera.row > 0 || delta_one.is_sign_positive()) {
					camera.row_offset_out = delta_one as i32;
				}

				camera.row_scroll_accum -= text_descriptor.glyph_height * camera.row_scroll_accum.signum();
			}

			// we also keep row_scroll_accum in range of 0..glyph_width
			while camera.column_scroll_accum.abs() > text_descriptor.glyph_width {
				let delta_one = delta.x.signum();
				if camera.enabled_translation && (camera.column > 0 || delta_one.is_sign_positive()) {
					camera.column = (camera.column as f32 + delta_one) as u32;
					// clamping
					camera.column = camera.column.min(text_descriptor.columns * 2);
				}

				camera.column_scroll_accum -= text_descriptor.glyph_width * camera.column_scroll_accum.signum();
			}

			let column	= camera.column as f32;
			
			let row_min = camera.visible_rows / 2.0 - 1.0;
			let row_max = text_descriptor.rows as f32 - (row_min / 2.0);
			let row		= ((camera.row + camera.row_offset_in) as f32).clamp(row_min, row_max);
			
			camera.horizontal_scroll = column * text_descriptor.glyph_width;
			camera.vertical_scroll = row * text_descriptor.glyph_height;

			if !camera.column_scroll_mouse_quantized {
				camera.horizontal_scroll += camera.column_scroll_accum;
			}

			if !camera.row_scroll_mouse_quantized {
				camera.vertical_scroll += camera.row_scroll_accum;
			}

			if camera.slowly_quantize_camera_position { // always slowly move camera to quantized position
				let inertia = (delta_seconds / camera.slow_quantizing_easing_seconds).min(1.0);

				camera.row_scroll_accum = camera.row_scroll_accum.lerp(0.0, inertia);
				camera.column_scroll_accum = camera.column_scroll_accum.lerp(0.0, inertia);
			}

			let vertical_scroll = if camera.invert_y { camera.vertical_scroll } else { -camera.vertical_scroll };

			camera.target_translation = target_transform.translation
				+ camera.zoom * unit_vector_from_yaw_and_pitch(yaw_radians, pitch_radians)
				+ Vec3::X * camera.horizontal_scroll
				+ Vec3::Y * vertical_scroll
				;

			let inertia = (delta_seconds / camera.translation_easing_seconds).min(1.0);
			camera_transform.translation = camera_transform.translation.lerp(camera.target_translation, inertia);
		}

		{ // rotation
			let (target_pitch, inertia) = if camera.enabled_rotation {
				let value = 3.0;

				let delta_y = if camera.invert_y { -delta.y } else { delta.y };

				if delta_y < 0.0 {
					(-value, delta_seconds / camera.lean_easing_seconds)
				} else if delta_y > 0.0 {
					(value, delta_seconds / camera.lean_easing_seconds)
				} else if camera.pitch_changed {
					(camera.pitch, delta_seconds / camera.lean_easing_seconds)
				} else {
					(0.0, delta_seconds / camera.lean_reset_easing_seconds)
				}
			} else {
				(0.0, delta_seconds / camera.lean_reset_easing_seconds)
			};

			camera.pitch = camera.pitch.lerp(target_pitch, inertia);
			camera.yaw = camera.yaw.lerp(0.0, inertia); // we don't have use for yaw in this mode for now

			let from = camera_transform.rotation;
			let to = Quat::from_axis_angle(Vec3::X, camera.pitch.to_radians());

			let inertia = (delta_seconds / camera.rotation_easing_seconds).min(1.0);
			camera_transform.rotation = from.slerp(to, inertia);
		}
	}
}

pub fn calc_visible_rows(
	mut q_camera : Query<(&mut ReaderCamera, &GlobalTransform, &Projection)>,
		q_target : Query<(&TextDescriptor, &Transform)>,
)
{
	for (mut camera_reader, camera_transform, camera_projection) in q_camera.iter_mut() {
		if camera_reader.mode != CameraMode::Reader {
			continue;
		}

		if let Some(target) = camera_reader.target { 
			let (text_descriptor, target_transform) = q_target.get(target).unwrap();
			
			// we remove x and y since the amount of visible rows should depend on how much we scrolled, just how far the camera is from the surface with text
			let mut camera_transform_z_only = camera_transform.clone();
			let camera_pos = camera_transform_z_only.translation_mut();
			camera_pos.x = 0.0;
			camera_pos.y = 0.0;
			
			// calculating frustum manually for now because using cached introduces small desync between frustum and camera position
			let projection_matrix = camera_projection.get_projection_matrix() * camera_transform_z_only.compute_matrix().inverse();
			let frustum = Frustum::from_view_projection(
				&projection_matrix,
				&camera_transform_z_only.translation(),
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
			
			camera_reader.y_top = y_top;
			
			let plane_bottom = &frustum.planes[2].normal_d();
			let y_bottom = (-plane_bottom.w - plane_bottom.z * z) / plane_bottom.y;
			
			camera_reader.y_bottom = y_bottom;
			
			let plane_left = &frustum.planes[0].normal_d();
			let x_left = (-plane_left.w - plane_left.z * z) / plane_left.y;
			
			camera_reader.x_left = x_left;
			
			let plane_right = &frustum.planes[1].normal_d();
			let x_right = (-plane_right.w - plane_right.z * z) / plane_right.y;
			
			camera_reader.x_right = x_right;
			
			camera_reader.visible_rows = (y_top - y_bottom) / row_height;
		}
	}
}