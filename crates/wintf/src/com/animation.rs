use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::DirectComposition::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Animation::*;

pub trait IDCompositionAnimationExt {
    /// GetCurve
    fn get_curve<P0>(&self, animation: P0) -> Result<()>
    where
        P0: Param<IDCompositionAnimation>;
}

impl IDCompositionAnimationExt for IUIAnimationVariable2 {
    #[inline(always)]
    fn get_curve<P0>(&self, animation: P0) -> Result<()>
    where
        P0: Param<IDCompositionAnimation>,
    {
        unsafe { self.GetCurve(animation) }
    }
}

pub trait IUIAnimationManagerExt {
    fn create_animation_variable(&self, initialvalue: f64) -> Result<IUIAnimationVariable2>;
    fn update(&self, time: f64) -> Result<()>;
    fn create_storyboard(&self) -> Result<IUIAnimationStoryboard2>;
}

impl IUIAnimationManagerExt for IUIAnimationManager2 {
    fn create_animation_variable(&self, initialvalue: f64) -> Result<IUIAnimationVariable2> {
        unsafe { self.CreateAnimationVariable(initialvalue) }
    }
    fn update(&self, time: f64) -> Result<()> {
        unsafe { self.Update(time, None).map(|_| ()) }
    }
    fn create_storyboard(&self) -> Result<IUIAnimationStoryboard2> {
        unsafe { self.CreateStoryboard() }
    }
}

pub fn create_animation_manager() -> Result<IUIAnimationManager2> {
    unsafe { CoCreateInstance(&UIAnimationManager2, None, CLSCTX_INPROC_SERVER) }
}

pub fn create_animation_transition_library() -> Result<IUIAnimationTransitionLibrary2> {
    unsafe { CoCreateInstance(&UIAnimationTransitionLibrary2, None, CLSCTX_INPROC_SERVER) }
}
