mod erosion;
use std::error::Error;

use erosion::ErosionApp;
use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn Error>>  {
    
    let event_loop =  EventLoop::new()?;
    let mut app = ErosionApp::new();




    println!("Hello, world!");



    Ok(event_loop.run_app(&mut app)?)
}
