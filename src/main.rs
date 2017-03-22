#![cfg(windows)]

extern crate winapi;
extern crate winit;

mod dcomp_window;

use winit::{WindowBuilder, Event, EventsLoop, WindowEvent};
use dcomp_window::DCompWindow;

fn main() {
    let events_loop = EventsLoop::new();
    let window = WindowBuilder::new().with_title("hello window").build(&events_loop).unwrap();
    let handle = window.handle();
    println!("window handle = {:?}", handle);
    loop {
        let mut closed = false;
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event: WindowEvent::Resized(w, h), .. } => {
                    println!("The window was resized to {}x{}", w, h);
                },
                Event::WindowEvent{event: WindowEvent::Closed,..} =>{
                    closed = true;
                },
                _ => ()
            }
        });
        if closed {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
