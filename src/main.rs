mod erosion;
use std::{collections::HashMap, error::Error};

use erosion::ErosionApp;
use winit::{event_loop::EventLoop, keyboard::KeyCode};
use maths::{Camera, CameraDirections};

fn main() -> Result<(), Box<dyn Error>>  {
    
    let event_loop =  EventLoop::new()?;
    


    let camera_map = camera_controls();
    let camera = Camera::new(None, None, None, None, Some(camera_map));

    let mut app = ErosionApp::new(100, 100, camera);

    Ok(event_loop.run_app(&mut app)?)
}



fn camera_controls() -> HashMap<KeyCode, CameraDirections> {
    let mut camera_map = HashMap::new();
    camera_map.insert(KeyCode::KeyW, CameraDirections::Forward);
    camera_map.insert(KeyCode::KeyS, CameraDirections::Backwards);
    camera_map.insert(KeyCode::KeyA, CameraDirections::Left);
    camera_map.insert(KeyCode::KeyD, CameraDirections::Right);
    camera_map.insert(KeyCode::Space, CameraDirections::Up);
    camera_map.insert(KeyCode::KeyC, CameraDirections::Down);
    camera_map.insert(KeyCode::KeyQ, CameraDirections::SpinLeft);
    camera_map.insert(KeyCode::KeyE, CameraDirections::SpinRight);
    camera_map.insert(KeyCode::KeyR, CameraDirections::SpinForward);
    camera_map.insert(KeyCode::KeyF, CameraDirections::SpinBackward);
    camera_map
}