#![cfg(windows)]

extern crate winapi;
extern crate winit;

mod hwnd_window;
mod com_rc;
mod unsafe_api;
mod unsafe_util;
mod dx_api;
mod model;

use model::DxModel;
use winapi::shared::winerror::HRESULT;
use winit::{Event, EventsLoop, WindowBuilder, WindowEvent};

fn main() {
    match run() {
        Err(e) => println!("err! {:?}", e),
        _ => (),
    }
}

fn run() -> Result<(), HRESULT> {
    let model = {
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

    let events_loop = model.events_loop();
    events_loop.run_forever(|event| {
        let rc = match event {
            Event::WindowEvent { event: WindowEvent::Resized(w, h), .. } => {
                println!("The window was resized to {}x{}", w, h);
            }
            Event::WindowEvent { event: WindowEvent::Closed, .. } => {
                events_loop.interrupt();
            }
            _ => (),
        };
        rc
    });
    Ok(())
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
