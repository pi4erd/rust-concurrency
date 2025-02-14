use voxelgame::VoxelGame;
use window::GameWindow;
use winit::event_loop::EventLoop;

mod voxelgame;
mod window;

fn main() {
    pretty_env_logger::init();

    // TODO: Add panic hook with message box, `dialog` crate

    let event_loop = EventLoop::new().unwrap();
    let mut window: GameWindow<VoxelGame<'_>> = GameWindow::new("Voxel game");

    event_loop
        .run_app(&mut window)
        .expect("Error while running application");
}
