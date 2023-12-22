use bevy :: prelude :: *;

use super :: CameraMode;

#[derive(PartialEq, Eq)]
pub(crate) enum AwakeState {
	Dormant,
	Awake,
}

#[derive(Component)]
pub struct ReaderCamera {
	///
	pub speed: f32,
	///
	pub sensitivity : f32,
	///
	pub swipe_sensitivity: f32,
	///
	pub mouse_scroll_sensitivity: f32,
	///
	pub wheel_scroll_sensitivity: f32,
	///
	pub zoom_sensitivity: f32,
	///
	pub pitch_max: f32,

	///
	pub mouse_scroll_easing_seconds: f32,
	///
	pub swipe_easing_seconds: f32,
	///
	pub translation_easing_scroll_seconds: f32,
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

	///
	pub pitch: f32,
	///
	pub yaw: f32,
	///
	pub zoom: f32,
	///
	pub target_zoom: f32,
	///
	pub scroll: f32,
	///
	pub swipe: f32,
	///
	pub column: usize,
	///
	pub target_row_prev : f32,
	///
	pub row_constant_offset: f32,
	///
	pub(crate) row_offset_app: u32,
	///
	pub(crate) row_offset_camera: u32,
	///
	pub(crate) row_offset_delta: i32,
	///
	pub visible_rows: f32,
	///
	pub visible_rows_target: Option<f32>,
	///
	pub visible_columns: f32,
	///
	pub visible_columns_target: Option<f32>,
	///
	pub y_top: f32,
	///
	pub y_bottom: f32,
	///
	pub x_left: f32,
	///
	pub x_right: f32,
	///
	pub translation_easing_current: f32,

	///
	pub swipe_accum: f32,
	///
	pub scroll_accum: f32,
	///
	pub target_translation: Vec3,
	///
	pub target_rotation: Quat,

	///
	pub key_forward: KeyCode,
	///
	pub key_backward: KeyCode,
	///
	pub key_left: KeyCode,
	///
	pub key_right: KeyCode,
	///
	pub key_up: KeyCode,
	///
	pub key_down: KeyCode,
	///
	pub key_perspective: KeyCode,
	///
	pub mod_perspective: Option<KeyCode>,
	///
	pub perspective: bool,
	///

	///
	pub invert_y: bool,
	///
	pub pitch_changed: bool,
	///
	pub(crate) awake: AwakeState,
	/// prevents snapping back to precise row offset instantly
	pub(crate) scroll_idle_timer: Timer,

	///
	pub mode: CameraMode,

	// restrictions
	
	///
	pub enabled_translation: bool,
	///
	pub enabled_zoom: bool,
	///
	pub enabled_rotation: bool,
	///
	pub enabled_scroll: bool,

	// default restrictions
	
	///
	pub default_enabled_translation: bool,
	///
	pub default_enabled_zoom: bool,
	///
	pub default_enabled_rotation: bool,
	///
	pub default_enabled_scroll: bool,

	/// Object the camera is looking at. Kind of the same as setting camera as a child of this entity but with more flexibility
	pub target_entity: Option<Entity>,
}

impl Default for ReaderCamera {
	fn default() -> Self {
		Self {
			speed								: 10.0,
			sensitivity							: 3.0,
			swipe_sensitivity					: 0.0,
			mouse_scroll_sensitivity			: 1.0,
			wheel_scroll_sensitivity			: 0.3,
			zoom_sensitivity					: 1.0,
			pitch_max							: 1.0,

			mouse_scroll_easing_seconds			: 5.0,
			swipe_easing_seconds				: 6.0,
			translation_easing_seconds			: 0.05,
			translation_easing_scroll_seconds	: 0.15,
			rotation_easing_seconds				: 0.1,
			zoom_easing_seconds					: 0.01,
			lean_easing_seconds					: 0.1,
			lean_reset_easing_seconds			: 0.05,

			pitch								: 0.0,
			yaw									: 0.0,
			zoom								: 7.0,
			target_zoom							: 7.0,
			scroll								: 0.0,
			swipe								: 0.0,
			column								: 51,
			target_row_prev						: 0.0,
			row_constant_offset					: 0.0,
			row_offset_app						: 0,
			row_offset_camera					: 0,
			row_offset_delta					: 0,
			visible_rows						: 40.0,
			visible_rows_target					: None,
			visible_columns						: 80.0,
			visible_columns_target				: None,
			y_top								: 0.0,
			y_bottom							: 0.0,
			x_left								: 0.0,
			x_right								: 0.0,
			translation_easing_current			: 0.05,

			swipe_accum							: 0.0,
			scroll_accum						: 0.0,
			target_translation					: Vec3::ZERO,
			target_rotation						: Quat::IDENTITY,
			key_forward							: KeyCode::W,
			key_backward						: KeyCode::S,
			key_left							: KeyCode::A,
			key_right							: KeyCode::D,
			key_up								: KeyCode::Space,
			key_down							: KeyCode::ShiftLeft,
			key_perspective						: KeyCode::Return,
			mod_perspective						: Some(KeyCode::ControlLeft),
			perspective							: true,

			enabled_scroll						: true,
			enabled_translation					: false,
			enabled_rotation					: false,
			enabled_zoom						: false,

			default_enabled_scroll				: true,
			default_enabled_translation			: false,
			default_enabled_rotation			: false,
			default_enabled_zoom				: false,

			mode								: CameraMode::Fly,

			invert_y							: false,
			pitch_changed						: false,
			awake								: AwakeState::Awake,
			scroll_idle_timer					: Timer::from_seconds(0.25, TimerMode::Once),

			target_entity						: None,
		}
	}
}

impl ReaderCamera {
	pub fn set_mode(&mut self, mode: CameraMode) {
		self.mode = mode;
	}

	pub fn set_mode_wrestrictions(
		&mut self,
		camera_mode		: CameraMode,
		translation		: bool,
		rotation		: bool,
		zoom			: bool,
		scroll			: bool,
	) {
		self.mode = camera_mode;
		self.set_restrictions(translation, rotation, zoom, scroll);
	}

	pub fn set_restrictions(
		&mut self,
		translation		: bool,
		rotation		: bool,
		zoom			: bool,
		scroll			: bool,
	) {
		self.enabled_translation = translation;
		self.enabled_rotation = rotation;
		self.enabled_zoom = zoom;
		self.enabled_scroll = scroll;
	}

	pub fn apply_default_restrictions(&mut self) {
		self.enabled_translation = self.default_enabled_translation;
		self.enabled_rotation = self.default_enabled_rotation;
		self.enabled_zoom = self.default_enabled_zoom;
		self.enabled_scroll = self.default_enabled_scroll;
	}

	pub fn restrictions_are_default(&self) -> bool {
		self.enabled_translation == self.default_enabled_translation &&
		self.enabled_rotation == self.default_enabled_rotation &&
		self.enabled_zoom == self.default_enabled_zoom &&
		self.enabled_scroll == self.default_enabled_scroll
	}

	pub fn set_all_default_restrictions_false(&mut self) {
		self.default_enabled_translation = false;
		self.default_enabled_rotation = false;
		self.default_enabled_zoom = false;
		self.default_enabled_scroll = false;
	}

	pub fn set_row_offset_in(&mut self, row_offset_in: u32) {
		self.row_offset_app = row_offset_in;
	}

	pub fn row_offset_in(&self) -> u32 {
		self.row_offset_app
	}

	pub fn row_offset_out(&self) -> u32 {
		self.row_offset_camera
	}

	pub fn row_offset_delta(&self) -> i32 {
		self.row_offset_delta
	}

	pub fn row_offset_delta_apply(&mut self) -> i32 {
		let cache = self.row_offset_delta;
		self.row_offset_delta = 0;
		cache
	}

	pub fn put_to_sleep(&mut self) {
		self.awake = AwakeState::Dormant;
	}

	pub fn wake_up(&mut self) {
		self.awake = AwakeState::Awake;
	}

	pub fn is_awake(&self) -> bool {
		self.awake == AwakeState::Awake
	}

	pub fn is_dormant(&self) -> bool {
		self.awake == AwakeState::Dormant
	}

	pub fn move_requested(&self) -> bool {
		if !self.enabled_translation {
			return false
		}

		self.row_offset_in() != self.row_offset_out() || self.row_offset_delta() != 0
	}

	pub fn is_zooming(&self) -> bool {
		(self.target_zoom - self.zoom).abs() >= 0.001 // TODO: magic numbers bad, settings or constants good
	}

	pub fn is_moving(&self, transform: &Transform) -> bool {
		!self.target_translation.abs_diff_eq(transform.translation, 0.001) // TODO: magic numbers bad, settings or constants good
	}
}
