use bevy :: prelude	:: { * };

use lerp :: Lerp;

use super :: TextDescriptor;
use super :: reader_camera :: { * };
use super :: util :: { * };

use crate :: KeyScroll;

pub fn zoom(
	zoom_scalar_raw	: f32,
	camera			: &mut ReaderCamera
 ) -> f32 {
	let mut zoom_scalar	= 0.0;

	if camera.enabled_zoom {
		zoom_scalar = zoom_scalar_raw * camera.zoom_sensitivity;
	}

	camera.target_zoom = (zoom_scalar + camera.target_zoom)
		.min(100.0)
		.max(3.0);

	zoom_scalar
}

pub fn apply_zoom(
	delta_seconds	: f32,
	camera			: &mut ReaderCamera
 ) {
	let inertia = (delta_seconds / camera.zoom_easing_seconds).min(1.0);
	camera.zoom = camera.zoom.lerp(camera.target_zoom, inertia);
}

pub fn zoom_adjustment(
	text_descriptor			: &TextDescriptor,
	target_entity_transform	: &Transform,
	camera_projection		: &Projection,
	camera_transform		: &mut Transform,
	camera					: &mut ReaderCamera,
) {
	let target_z			= target_entity_transform.translation.z;
	let mut camera_transform_z_only = camera_transform.clone();
	// we remove x and y since the amount of visible rows should depend on how much we scrolled and how far the camera is from the surface with text
	camera_transform_z_only.translation = Vec3::Z * (camera.target_zoom + target_z);

	let frustum				= calc_frustum		(&camera_transform_z_only, camera_projection);
	let visible_rows_new	= calc_visible_rows	(&frustum, target_z, text_descriptor.glyph_height);
	let visible_rows_old	= if let Some(rows) = camera.visible_rows_target { rows } else { camera.visible_rows };

	camera.row_offset_delta	= ((visible_rows_old - visible_rows_new) / 2.0) as i32;
	camera.visible_rows_target = Some(visible_rows_new);

	// same for visible columns but without sending delta out
	let visible_columns_new	= calc_visible_columns(&frustum, target_z, text_descriptor.glyph_width);
	camera.visible_columns_target = Some(visible_columns_new);
}

pub fn rotation(
	delta_seconds		: f32,
	delta_scroll_in		: f32,
	rows_meta			: &RowsMetaData,
	camera				: &mut ReaderCamera
) -> f32 {
	let (target_pitch, inertia) = if camera.enabled_rotation && !rows_meta.text_start_reached && !rows_meta.text_end_reached {
		let value		= camera.pitch_max;

		let delta_scroll = if camera.invert_y { -delta_scroll_in } else { delta_scroll_in };

		if delta_scroll < 0.0 {
			(value, delta_seconds / camera.lean_easing_seconds)
		} else if delta_scroll > 0.0 {
			(-value, delta_seconds / camera.lean_easing_seconds)
		} else if camera.pitch_changed {
			(camera.pitch, delta_seconds / camera.lean_easing_seconds)
		} else {
			(0.0, delta_seconds / camera.lean_reset_easing_seconds)
		}
	} else {
		(0.0, delta_seconds / camera.lean_reset_easing_seconds)
	};

	camera.pitch		= camera.pitch.lerp(target_pitch, inertia);
	camera.yaw			= camera.yaw.lerp(0.0, inertia); // we don't have use for yaw in this mode for now

	let pitch_sin		= camera.pitch.to_radians().sin();
	let pitch_compensation = pitch_sin * camera.zoom * 2.0;

	pitch_compensation
}

pub fn apply_rotation(
	delta_seconds		: f32,
	camera				: &ReaderCamera,
	camera_transform	: &mut Transform,
) {
	let from			= camera_transform.rotation;
	let to				= Quat::from_axis_angle(Vec3::X, camera.pitch.to_radians());

	let inertia			= (delta_seconds / camera.rotation_easing_seconds).min(1.0);
	camera_transform.rotation = from.slerp(to, inertia);
}

fn translation_swipe(
	delta_mouse_swipe	: f32,
	text_descriptor		: &TextDescriptor,
	camera				: &mut ReaderCamera,
) {
	// we keep column_scroll_accum in range of 0..glyph_width and use the leftover offset to change camera.column
    while camera.swipe_accum.abs() > text_descriptor.glyph_width {
		let delta_one		= delta_mouse_swipe.signum();
		if camera.column > 0 || delta_one.is_sign_positive() {
			camera.column	= (camera.column as f32 + delta_one) as usize;
			// clamping
			camera.column	= camera.column.min(text_descriptor.columns * 2);
		}

		camera.swipe_accum -= text_descriptor.glyph_width * camera.swipe_accum.signum();
	}

	camera.swipe			= camera.column as f32 * text_descriptor.glyph_width;
	camera.swipe			+= camera.swipe_accum;
}

pub struct RowsMetaData {
	pub row_changed			: bool,
	pub row_delta			: f32,
	pub row_max				: f32,
	pub target_row			: f32,
	pub visible_rows		: f32,
	pub visible_rows_half	: f32,
	pub text_start_reached	: bool,
	pub text_end_reached	: bool
}

fn translation_scroll(
	scroll_event_occurred	: bool,
	scroll_signum			: f32,
	pitch_compensation		: f32,
	rows_meta				: &RowsMetaData,
	text_descriptor			: &TextDescriptor,
	camera					: &mut ReaderCamera,
) {
	// slowly snap back to precise row offset (it is possible to scroll in between rowss)
	if !scroll_event_occurred && camera.scroll_idle_timer.finished() {
		let target = if camera.scroll_accum.abs() < text_descriptor.glyph_height / 2.0 {
			0.0
		} else {
			text_descriptor.glyph_height * camera.scroll_accum.signum()
		};

		camera.scroll_accum = camera.scroll_accum.lerp(target, 0.1);

		if (target - camera.scroll_accum).abs() < 0.001 {
			camera.scroll_accum = target;
		}
	}

	// we keep row_scroll_accum in range of 0..glyph_height and use the leftover offset to change camera.row_offset_delta
	while camera.scroll_accum.abs() >= text_descriptor.glyph_height {
		let scroll_accum_signum = camera.scroll_accum.signum();
		if (!rows_meta.text_start_reached || scroll_signum.is_sign_positive()) && (!rows_meta.text_end_reached || scroll_signum.is_sign_negative()) {
			// row_offset_delta tells app that we need to scroll and we expect actual scroll state in row_offset_in
			camera.row_offset_delta += scroll_accum_signum as i32;
		}

		let row_scroll_unit = scroll_accum_signum;
		camera.scroll_accum -= text_descriptor.glyph_height * row_scroll_unit;
	}

	camera.scroll			= (rows_meta.target_row + camera.row_offset_delta as f32) * text_descriptor.glyph_height;
	camera.scroll			+= pitch_compensation;

	if !rows_meta.text_start_reached && !rows_meta.text_end_reached {
		camera.scroll		+= camera.scroll_accum;
	}
}

pub fn translation(
	key_scroll_state		: Option<KeyScroll>,
	delta_wheel				: Option<f32>,
	delta_mouse				: Vec2,
	delta_seconds			: f32,

	pitch_compensation		: f32,
	rows_meta				: &RowsMetaData,

	text_descriptor			: &TextDescriptor,
	target_object_transform	: &Transform,
	camera					: &mut ReaderCamera,
) {
	let mut scroll_signum = 0.0;

	if camera.enabled_translation {
		scroll_signum		+= delta_mouse.y;

		let delta_x			= delta_mouse.x * camera.swipe_sensitivity;
		let delta_y			= delta_mouse.y * camera.mouse_scroll_sensitivity;

		camera.scroll_accum	+= delta_y * (delta_seconds / camera.mouse_scroll_easing_seconds);
		camera.swipe_accum	+= delta_x * (delta_seconds / camera.swipe_easing_seconds);
	}

	if camera.enabled_scroll && delta_wheel.is_some() {
		let delta_wheel		= delta_wheel.unwrap();
		scroll_signum		+= delta_wheel;

		camera.scroll_accum	+= delta_wheel * camera.wheel_scroll_sensitivity;
	}

	scroll_signum			= scroll_signum.signum();

	// Column related stuff
	translation_swipe(
		delta_mouse.x,
		text_descriptor,
		camera
	);

	// Row related stuff
	translation_scroll(
		delta_wheel.is_some(),
		scroll_signum,
		pitch_compensation,
		rows_meta,
		text_descriptor,
		camera,
	);

	// Final composition + easing

	let yaw_radians			= camera.yaw.to_radians();
	let pitch_radians		= camera.pitch.to_radians();

	let scroll				= if camera.invert_y { camera.scroll } else { -camera.scroll };

	camera.target_translation = target_object_transform.translation
		+ camera.zoom * unit_vector_from_yaw_and_pitch(yaw_radians, pitch_radians)
		+ Vec3::X * camera.swipe
		+ Vec3::Y * scroll
	;

	let easing_target = if key_scroll_state.is_some() { camera.translation_easing_scroll_seconds } else { camera.translation_easing_seconds };
	camera.translation_easing_current = camera.translation_easing_current.lerp(easing_target, 0.1);
}

pub fn apply_translation(
	delta_seconds			: f32,
	rows_meta				: &RowsMetaData,
	camera					: &ReaderCamera,
	camera_transform		: &mut Transform,
) {
	let instant_translate	= rows_meta.row_delta.abs() > (rows_meta.visible_rows + rows_meta.visible_rows_half);
	let inertia				= if !instant_translate { (delta_seconds / camera.translation_easing_current).min(1.0) } else { 1.0 };
	camera_transform.translation = camera_transform.translation.lerp(camera.target_translation, inertia);
}