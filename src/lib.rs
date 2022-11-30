use bevy :: prelude :: { * };

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CameraMode {
	Fly,
	Follow,
	Reader,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum KeyScroll {
	Up,
	Down
}

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct TextDescriptor {
	pub glyph_width		: f32,
	pub glyph_height	: f32,
	pub rows			: usize,
	pub columns			: usize,
}

mod reader_camera;
pub use reader_camera :: ReaderCamera;

mod util;
mod reader_mode;
mod systems;

#[derive(SystemSet, PartialEq, Eq, Hash, Clone, Debug)]
pub struct ReaderCameraUpdate;

pub struct ReaderCameraPlugin;

impl Plugin for ReaderCameraPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_system(systems::fly_mode_keyboard)
			.add_system(systems::fly_mode_mouse)
			.add_system(systems::follow_mode_mouse)

			// PreUpdate because camera transform has to be the same for all systems during update
			// and because Frustum gets desynced with camera transform and that makes the amount of visible rows jitter
			.add_system(systems::reader_mode.in_base_set(CoreSet::PreUpdate).in_set(ReaderCameraUpdate))
			.add_system(systems::calc_frustum_data.in_base_set(CoreSet::PreUpdate).after(systems::reader_mode))
		;
	}
}
