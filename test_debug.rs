use windows::Win32::Graphics::DirectComposition::IDCompositionVisual3;
fn main() {
    // Check if Debug is implemented
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_debug::<IDCompositionVisual3>();
}
