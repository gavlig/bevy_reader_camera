use bevy :: prelude :: { * };

#[derive(PartialEq, Eq)]
pub enum CameraMode {
	Fly,
	Follow,
	Reader,
}

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct TextDescriptor {
	pub glyph_width: f32,
	pub glyph_height: f32,
	pub rows: u32,
	pub columns: u32,
}

mod reader_camera;
pub use reader_camera :: ReaderCamera;

mod util;
mod systems;

#[derive(Component, Default)]
pub struct Row(pub u32);

#[derive(Component, Default)]
pub struct Column(pub u32);

pub struct ReaderCameraPlugin;

impl Plugin for ReaderCameraPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_system(systems::init_camera)
			.add_system(systems::keyboard_fly)
			.add_system(systems::mouse_fly)
			.add_system(systems::mouse_follow)
			.add_system(systems::mouse_reader)
			
			.add_system_to_stage(CoreStage::PreUpdate, systems::calc_visible_rows) // PreUpdate because Frustum gets desynced with camera transform and that makes the amount of visible rows jitter
		;
	}
}
