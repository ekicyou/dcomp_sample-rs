use windows::Win32::Graphics::DirectComposition::IDCompositionVisual;
fn test(v: &IDCompositionVisual) {
    unsafe { v.SetOpacity(0.5); }
}
