#![cfg(windows)]

extern crate winapi;
extern crate winit;

//mod raw_com_if_sample;
mod com_rc;
mod dcomp_api;

mod dcomp_window;

use winapi::shared::windef::HWND;
use winit::{WindowBuilder, Event, EventsLoop, WindowEvent};
use dcomp_window::DCompWindow;

impl DCompWindow for winit::Window {
    fn hwnd(&self) -> HWND {
        unsafe {
            #[allow(deprecated)]
            let p = self.platform_window();
            p as HWND
        }
    }
}

fn main() {
    let events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_title("hello window")
        .with_dimensions(480, 640)
        .with_transparency(true)
        .with_multitouch()
        .build(&events_loop)
        .unwrap();
    let hwnd = window.hwnd();
    println!("window hwnd = {:?}", hwnd);
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
