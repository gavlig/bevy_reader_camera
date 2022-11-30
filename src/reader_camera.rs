use bevy :: prelude :: *;

use super :: CameraMode;

#[derive(Component)]
pub struct ReaderCamera {
	/// The speed the ReaderCamera accelerates at. Defaults to `1.0`
	pub accel: f32,
	/// The maximum speed the ReaderCamera can move at. Defaults to `0.5`
	pub max_speed: f32,
	/// The sensitivity of the ReaderCamera's motion based on mouse movement. Defaults to `3.0`
	pub sensitivity: f32,
	/// The amount of deceleration to apply to the camera's motion. Defaults to `1.0`
	pub friction: f32,
	///
	pub zoom_sensitivity: f32,

	///
	pub vertical_scroll_easing_seconds: f32,
	///
	pub horizontal_scroll_easing_seconds: f32,
	///
	pub translation_easing_seconds: f32,
	///
	pub rotation_easing_seconds: f32,
	///
	pub zoom_easing_seconds: f32,
	///
	pub lean_easing_seconds: f32,
	///
	pub lean_reset_easing_seconds: f32,
	/// The current pitch of the ReaderCamera in degrees
	pub pitch: f32,
	/// The current pitch of the ReaderCamera in degrees
	pub yaw: f32,
	///
	pub zoom: f32,
	///
	pub vertical_scroll: f32,
	///
	pub horizontal_scroll: f32,
	///
	pub column: u32,
	///
	pub row: u32,
	///
	pub column_scroll_accum: f32,
	///
	pub row_scroll_accum: f32,
	///
	pub column_scroll_mouse_quantized: bool,
	///
	pub row_scroll_mouse_quantized: bool,
	///
	pub slowly_quantize_camera_position: bool,
	///
	pub slow_quantizing_easing_seconds: f32,
	///
	pub target_translation: Vec3,
	///
	pub target_rotation: Quat,

	///
	pub key_scroll_delay_seconds: f32,
	///
	pub key_scroll_delay_column_inc: f32,
	///
	pub key_scroll_delay_column_dec: f32,
	///
	pub key_scroll_delay_row_inc: f32,
	///
	pub key_scroll_delay_row_dec: f32,
	/// The current velocity of the ReaderCamera. This value is always up-to-date, enforced by [ReaderCameraPlugin](struct.ReaderCameraPlugin.html)
	pub velocity: Vec3,

	/// Key used to move forward. Defaults to <kbd>W</kbd>
	pub key_forward: KeyCode,
	/// Key used to move backward. Defaults to <kbd>S</kbd>
	pub key_backward: KeyCode,
	/// Key used to move left. Defaults to <kbd>A</kbd>
	pub key_left: KeyCode,
	/// Key used to move right. Defaults to <kbd>D</kbd>
	pub key_right: KeyCode,
	/// Key used to move up. Defaults to <kbd>Space</kbd>
	pub key_up: KeyCode,
	/// Key used to move forward. Defaults to <kbd>LShift</kbd>
	pub key_down: KeyCode,
	/// Key used to toggle perspective mode on camera. Defaults to <kbd>Return</kbd>
	pub key_perspective: KeyCode,
	/// Key used as modifier along with key_perspective. Optional. Defaults to Some(<kbd>LControl</kbd>)
	pub mod_perspective: Option<KeyCode>,
	/// If `true` camera rotation gets reset to 0 when projection switches to orthographic from perspective. Defaults to `true`
	pub reset_rotation_on_ortho: bool,
	/// indicates if camera has perspective projection. Switches to orthographic when `false`
	pub perspective: bool, 
	/// If `false`, disable keyboard control of the camera. Defaults to `true`
	
	///
	pub invert_y: bool,
	///
	pub pitch_changed: bool,

	///
	pub mode: CameraMode,
	///
	pub enabled_translation: bool,
	/// If `false`, disable mouse control of the camera. Defaults to `true`
	pub enabled_rotation: bool,
	///
	pub enabled_zoom: bool,

	///
	pub target: Option<Entity>,
}

impl Default for ReaderCamera {
	fn default() -> Self {
		Self {
			accel								: 1.5,
			max_speed							: 0.5,
			sensitivity							: 3.0,
			friction							: 1.0,
			zoom_sensitivity					: 0.15,
			vertical_scroll_easing_seconds		: 5.0,
			horizontal_scroll_easing_seconds	: 6.0,
			translation_easing_seconds			: 0.2,
			rotation_easing_seconds				: 1.0,
			zoom_easing_seconds					: 0.01,
			lean_easing_seconds					: 1.0,
			lean_reset_easing_seconds			: 0.2,
			pitch								: 0.0,
			yaw									: 0.0,
			zoom								: 6.7,
			vertical_scroll						: 0.0,
			horizontal_scroll					: 0.0,
			column								: 51,
			row									: 19,
			column_scroll_accum					: 0.0,
			row_scroll_accum					: 0.0,
			column_scroll_mouse_quantized		: false,
			row_scroll_mouse_quantized			: false,
			slowly_quantize_camera_position		: true,
			slow_quantizing_easing_seconds		: 0.25,
			target_translation					: Vec3::ZERO,
			target_rotation						: Quat::IDENTITY,
			key_scroll_delay_seconds			: 0.03,
			key_scroll_delay_row_inc			: 0.0,
			key_scroll_delay_row_dec			: 0.0,
			key_scroll_delay_column_inc			: 0.0,
			key_scroll_delay_column_dec			: 0.0,
			velocity							: Vec3::ZERO,
			key_forward							: KeyCode::W,
			key_backward						: KeyCode::S,
			key_left							: KeyCode::A,
			key_right							: KeyCode::D,
			key_up								: KeyCode::Space,
			key_down							: KeyCode::LShift,
			key_perspective						: KeyCode::Return,
			mod_perspective						: Some(KeyCode::LControl),
			reset_rotation_on_ortho				: true,
			perspective							: true,

			enabled_translation					: true,
			enabled_rotation					: true,
			enabled_zoom						: true,

			mode								: CameraMode::Fly,

			invert_y							: false,
			pitch_changed						: false,
			target								: None,
		}
	}
}

impl ReaderCamera {
	pub fn set_mode_wrestrictions(
		&mut self,
		camera_mode		: CameraMode,
		translation		: bool,
		rotation		: bool,
		zoom			: bool,
	)
	{
		self.mode = camera_mode;
		self.set_restrictions(translation, rotation, zoom);
	}
	
	pub fn set_restrictions(
		&mut self,
		translation		: bool,
		rotation		: bool,
		zoom			: bool,
	)
	{
		self.enabled_translation = translation;
		self.enabled_rotation = rotation;
		self.enabled_zoom = zoom;
	}
	
	pub fn set_pitch(&mut self, pitch: f32) {
		self.pitch = pitch;
		self.pitch_changed = true;
	}

	pub fn column_inc(&mut self, delta_seconds: f32) {
		self.key_scroll_delay_column_inc += delta_seconds;
		if self.key_scroll_delay_column_inc >= self.key_scroll_delay_seconds {
			self.column += 1;
			self.key_scroll_delay_column_inc -= self.key_scroll_delay_seconds;
		}
	}

	pub fn column_dec(&mut self, delta_seconds: f32) {
		self.key_scroll_delay_column_dec += delta_seconds;
		if self.column > 0 && self.key_scroll_delay_column_dec >= self.key_scroll_delay_seconds {
			self.column -= 1;
			self.key_scroll_delay_column_dec -= self.key_scroll_delay_seconds;
		}
	}

	pub fn row_inc(&mut self, delta_seconds: f32) {
		self.key_scroll_delay_row_inc += delta_seconds;

		self.set_pitch(-1.0);

		if self.key_scroll_delay_row_inc >= self.key_scroll_delay_seconds {
			self.row += 1;
			self.key_scroll_delay_row_inc -= self.key_scroll_delay_seconds;
		}

	}

	pub fn row_dec(&mut self, delta_seconds: f32) {
		self.key_scroll_delay_row_dec += delta_seconds;

		self.set_pitch(1.0);

		if self.row > 0 && self.key_scroll_delay_row_dec >= self.key_scroll_delay_seconds {
			self.row -= 1;
			self.key_scroll_delay_row_dec -= self.key_scroll_delay_seconds;
		}
	}
}