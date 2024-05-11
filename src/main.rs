use lise::renderer::{shader::Shader, vkcontext::VkContext, Renderer};
use simple_logger::SimpleLogger;
use simple_window::{Window, WindowEvent};

fn main() {
    SimpleLogger::new().init().unwrap();

    let mut window = Window::new("LiSE Test", 200, 200, 400, 500);

    let vkcontext = VkContext::new(&window);

    let renderer = Renderer::new(&vkcontext);

    log::debug!("Entering game loop.");
    let mut is_running = true;
    while is_running {
        window.poll_messages(|event| {
            match event {
                WindowEvent::Close => is_running = false,
                _ => (),
            }
        });
    }
}
