#![cfg(windows)]

#[macro_use]
extern crate winapi;
extern crate winit;

//mod raw_com_if_sample;
mod hwnd_window;
mod com_rc;
mod unsafe_api;
mod dx_api;
mod model;

use winapi::shared::winerror::HRESULT;
use winit::{WindowBuilder, Event, EventsLoop, WindowEvent};
use model::DxModel;

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
            .with_dimensions(480, 640)
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
