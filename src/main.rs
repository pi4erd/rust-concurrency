use voxelgame::VoxelGame;
use window::GameWindow;
use winit::event_loop::EventLoop;

mod window;
mod voxelgame;

fn main() {
    pretty_env_logger::init();
    
    let event_loop = EventLoop::new().unwrap();
    let mut window: GameWindow<VoxelGame<'_>> = GameWindow::new("Voxel game");

    event_loop.run_app(&mut window)
        .expect("Error while running application");
}
