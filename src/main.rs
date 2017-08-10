#![cfg(windows)]

extern crate winapi;
extern crate libc;
extern crate winit;
extern crate widestring;
#[macro_use]
extern crate c_string;
#[macro_use]
extern crate lazy_static;
extern crate euclid;

mod hwnd_window;
mod com;
mod consts;
mod texture;
mod model;

use model::DxModel;
use winapi::shared::winerror::HRESULT;
use winit::{ControlFlow, Event, EventsLoop, WindowBuilder, WindowEvent};

fn main() {
    match run() {
        Err(e) => println!("err! {:?}", e),
        _ => (),
    }
}

fn run() -> Result<(), HRESULT> {
    let mut model = {
        let events_loop = EventsLoop::new();
        let window = WindowBuilder::new()
            .with_title("hello window")
            .with_dimensions(512, 512)
            .with_transparency(true)
            .with_multitouch()
            .build(&events_loop)
            .unwrap();
        DxModel::new(events_loop, window)?
    };

    let mut events_loop = model.events_loop_mut();
    events_loop.run_forever(|event| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(w, h), ..
            } => {
                println!("The window was resized to {}x{}", w, h);
            }
            Event::WindowEvent { event: WindowEvent::Closed, .. } => {
                return ControlFlow::Break
            }
            _ => {}
        }
        ControlFlow::Continue
    });
    Ok(())
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
