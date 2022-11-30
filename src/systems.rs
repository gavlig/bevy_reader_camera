use bevy :: {
	prelude	:: { * },
	input	:: mouse :: { MouseMotion, MouseScrollUnit, MouseWheel },
	render	:: camera :: { * },
};

use super :: CameraMode;
use super :: TextDescriptor;
use super :: reader_camera :: { * };
use super :: util :: { * };

use crate :: KeyScroll;

pub fn fly_mode_keyboard(
		time		: Res<Time>,
		key_code	: Res<Input<KeyCode>>,
	mut q_camera	: Query<(&mut ReaderCamera, &mut Transform, &mut Projection)>,
) {
	let delta_seconds = time.delta_seconds();

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
					}
				)
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

		camera.velocity += accel * delta_seconds;

		// clamp within max speed
		if camera.velocity.length() > camera.max_speed {
			camera.velocity = camera.velocity.normalize() * camera.max_speed;
		}

		let from			= camera_transform.translation;
		let to				= camera_transform.translation + camera.velocity;

		let inertia			= (delta_seconds / camera.translation_easing_seconds).min(1.0);
		camera_transform.translation = from.lerp(to, inertia);
	}
}

pub fn fly_mode_mouse(
		time: Res<Time>,
	mut mouse_motion_event_reader: EventReader<MouseMotion>,
	mut q_camera: Query<(&mut ReaderCamera, &mut Transform)>,
) {
	let mut delta: Vec2 = Vec2::ZERO;
	for event in mouse_motion_event_reader.iter() {
		delta += event.delta;
	}
	if delta.is_nan() {
		return;
	}

	let delta_seconds = time.delta_seconds();

	for (mut camera, mut transform) in q_camera.iter_mut() {
		if camera.mode != CameraMode::Fly || !camera.enabled_rotation {
			continue;
		}
		camera.yaw			-= delta.x * camera.sensitivity * time.delta_seconds();
		camera.pitch		+= delta.y * camera.sensitivity * time.delta_seconds();

		camera.pitch		= camera.pitch.clamp(-89.0, 89.9);

		let yaw_radians		= camera.yaw.to_radians();
		let pitch_radians	= camera.pitch.to_radians();

		let from			= transform.rotation;
		let to				= Quat::from_axis_angle(Vec3::Y, yaw_radians) * Quat::from_axis_angle(-Vec3::X, pitch_radians);

		let inertia			= (delta_seconds / camera.rotation_easing_seconds).min(1.0);
		transform.rotation	= from.slerp(to, inertia);
	}
}

pub fn follow_mode_mouse(
		time						: Res<Time>,
	mut mouse_motion_event_reader	: EventReader<MouseMotion>,
	mut mouse_wheel_event_reader	: EventReader<MouseWheel>,
	mut q_camera					: Query<(&mut ReaderCamera, &mut Transform)>,
		q_target					: Query<&Transform, Without<ReaderCamera>>,
) {
	for (mut camera, mut camera_transform) in q_camera.iter_mut() {
		if camera.mode != CameraMode::Follow {
			continue
		}
		assert!(camera.target_entity != None);

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
			let target = camera.target_entity.unwrap();
			let target_transform = q_target.get(target).unwrap();

			camera_transform.translation = target_transform.translation + camera.zoom * unit_vector_from_yaw_and_pitch(yaw_radians, pitch_radians);
		}

		if camera.enabled_rotation {
			camera_transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw_radians) * Quat::from_axis_angle(-Vec3::X, pitch_radians);
		}
	}
}

use crate :: reader_mode as reader;

pub fn reader_mode(
		time						: Res<Time>,
		key							: Res<Input<KeyCode>>,
		mouse_motion_event_reader	: EventReader<MouseMotion>,
		mouse_wheel_event_reader	: EventReader<MouseWheel>,
	mut q_camera					: Query<(Entity, &mut ReaderCamera, &Projection)>,
		q_text_descriptor			: Query<&TextDescriptor>,
	mut	q_transform					: Query<&mut Transform>,
) {
	let pixels_per_line = 20.0;

	let key_scroll_state =
	if key.pressed(KeyCode::Up) && !key.just_pressed(KeyCode::Up) {
		Some(KeyScroll::Up)
	} else if key.pressed(KeyCode::Down) && !key.just_pressed(KeyCode::Down) {
		Some(KeyScroll::Down)
	} else {
		None
	};

	let Ok((camera_entity, mut camera, camera_projection)) = q_camera.get_single_mut() else {
		return
	};

	if camera.mode != CameraMode::Reader {
		return
	}

	let Some(camera_target_entity) = camera.target_entity else {
		return
	};

	let delta_wheel			= delta_wheel_from_events(pixels_per_line, mouse_wheel_event_reader);
	let delta_mouse			= delta_mouse_from_events(mouse_motion_event_reader);
	let mut delta_seconds	= time.delta_seconds();

	let wheel_event_occurred = delta_wheel.is_some();

	if !wheel_event_occurred {
		camera.scroll_idle_timer.tick(time.delta());
	} else {
		camera.scroll_idle_timer.reset();
	}

	// "make believe" fps for dormant state to avoid jitter when fps is too low
	if !camera.is_awake() {
		delta_seconds		= 1. / 60.;
	}

	let text_descriptor		= q_text_descriptor.get(camera_target_entity).unwrap();

	let row_max				= text_descriptor.rows as f32;

	let visible_rows		= if let Some(rows) = camera.visible_rows_target { rows } else { camera.visible_rows };
	let visible_rows_half	= visible_rows / 2.0;

	let target_row			= camera.row_constant_offset + camera.row_offset_app as f32 + visible_rows_half;

	let row_changed			= target_row != camera.target_row_prev;
	let row_delta			= target_row - camera.target_row_prev;

	camera.target_row_prev	= target_row;

	let text_start_reached	= camera.row_offset_app == 0;
	let text_end_reached	= target_row.ceil() + 1.0 >= row_max;

	let rows_meta = reader::RowsMetaData {
		row_changed,
		row_delta,
		row_max,
		target_row,
		visible_rows,
		visible_rows_half,
		text_start_reached,
		text_end_reached
	};

	// always look at center for now
	camera.column = (text_descriptor.columns / 2) as usize;

	//
	// Calculating camera transform for given row and column
	//

	// a copy to not have another read access to q_transform
	let target_entity_transform = q_transform.get(camera_target_entity).unwrap().clone();

	// contains expected transform for given row and column without any postprocessing
	let mut camera_transform = q_transform.get_mut(camera_entity).unwrap();

	if let Some(wheel) = delta_wheel {
		reader::zoom(
			wheel,
			&mut camera
		);
	}

	let apply_zoom = camera.is_zooming();
	if apply_zoom {
		reader::apply_zoom(delta_seconds, &mut camera);
	}

	let pitch_compensation = reader::rotation(
		delta_seconds,
		delta_mouse.y,
		&rows_meta,
		&mut camera
	);

	reader::apply_rotation(delta_seconds, &camera, &mut camera_transform);

	reader::translation(
		key_scroll_state,
		delta_wheel,
		delta_mouse,
		delta_seconds,

		pitch_compensation,
		&rows_meta,

		text_descriptor,
		&target_entity_transform,
		&mut camera,
	);

	reader::apply_translation(delta_seconds, &rows_meta, &camera, &mut camera_transform);

	// To keep camera looking at the same row when zooming we add some extra scrolling
	if apply_zoom {
		reader::zoom_adjustment(
			text_descriptor,
			&target_entity_transform,
			camera_projection,
			&mut camera_transform,
			&mut camera
		);
	}

	// Now we calculate the actual row offset we're looking at currently

	let row_offset_out = ((camera_transform.translation.y / text_descriptor.glyph_height).abs() - camera.row_constant_offset - rows_meta.visible_rows_half).round();

	camera.row_offset_camera = row_offset_out.max(0.0) as u32;
}

pub fn calc_frustum_data(
	mut q_camera : Query<(Entity, &mut ReaderCamera, &Projection)>,
		q_text_descriptor : Query<&TextDescriptor>,
		q_transform : Query<&Transform>,
) {
	let Ok((camera_entity, mut camera_reader, camera_projection)) = q_camera.get_single_mut() else { return };

	if camera_reader.mode != CameraMode::Reader {
		return;
	}

	let Some(target_entity) = camera_reader.target_entity else { return };

	let text_descriptor = q_text_descriptor.get(target_entity).unwrap();
	let target_entity_transform = q_transform.get(target_entity).unwrap();

	let target_entity_z = target_entity_transform.translation.z;

	let mut camera_transform_z_only = q_transform.get(camera_entity).unwrap().clone();
	// we remove x and y since the amount of visible rows should not depend on how much we scrolled, just how far the camera is from the surface with text
	camera_transform_z_only.translation = Vec3::Z * camera_transform_z_only.translation.z;

	// calculating frustum manually for now because using cache introduces small desync between frustum and camera position
	let frustum = calc_frustum(&camera_transform_z_only, camera_projection);

	//

	let row_height = text_descriptor.glyph_height;
	let column_width = text_descriptor.glyph_width;

	camera_reader.y_top		= calc_frustum_y_border(&frustum, target_entity_z, true);
	camera_reader.y_bottom	= calc_frustum_y_border(&frustum, target_entity_z, false);

	let visible_rows_prev = camera_reader.visible_rows;

	camera_reader.visible_rows = (camera_reader.y_top - camera_reader.y_bottom) / row_height; // calc_visible_rows(&frustum, target_object_z, row_height);

	if camera_reader.visible_rows_target.is_none() || visible_rows_prev == camera_reader.visible_rows {
		camera_reader.visible_rows_target = Some(camera_reader.visible_rows);
	}

	//

	camera_reader.x_left	= calc_frustum_x_border(&frustum, target_entity_z, false);
	camera_reader.x_right	= calc_frustum_x_border(&frustum, target_entity_z, true);

	let visible_columns_prev = camera_reader.visible_columns;

	camera_reader.visible_columns = (camera_reader.x_right - camera_reader.x_left) / column_width; // calc_visible_columns(&frustum, target_object_z, column_width);

	if camera_reader.visible_columns_target.is_none() || visible_columns_prev == camera_reader.visible_columns {
		camera_reader.visible_columns_target = Some(camera_reader.visible_columns);
	}
}
