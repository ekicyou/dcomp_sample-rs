#![cfg(windows)]

extern crate libc;
extern crate widestring;
extern crate winapi;
extern crate winit;
#[macro_use]
extern crate c_string;
#[macro_use]
extern crate lazy_static;
extern crate euclid;

mod com;
mod consts;
mod model;
mod texture;

use model::DxModel;
use winapi::shared::winerror::HRESULT;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::windows::WindowBuilderExtWindows,
    window::WindowBuilder,
};

fn main() {
    match run() {
        Err(e) => println!("err! {:?}", e),
        _ => (),
    }
}

fn run() -> Result<(), HRESULT> {
    let mut events_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("hello window")
        .with_inner_size(winit::dpi::LogicalSize::new(512, 512))
        .with_no_redirection_bitmap(true)
        .build(&events_loop)
        .unwrap();
    let mut model = DxModel::new(&window)?;

    events_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(_) => {
                let _ = model.render();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                println!(
                    "The window was resized to {}x{}",
                    size.width, size.height
                );
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
