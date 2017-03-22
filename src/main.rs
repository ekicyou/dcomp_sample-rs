#![cfg(windows)]

extern crate winapi;
extern crate winit;

use winit::{Event, EventsLoop, Window, WindowEvent};

fn main() {
    let events_loop = EventsLoop::new();
    let window = winit::Window::new(&events_loop).unwrap();
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
