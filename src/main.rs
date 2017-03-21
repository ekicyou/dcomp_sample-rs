#![cfg(windows)]

extern crate winapi;
extern crate winit;

fn main() {
    let window = winit::Window::new().unwrap();

    for event in window.wait_events() {
        match event {
            winit::Event::Closed => break,
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
