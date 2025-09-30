use windows::core::*;
use windows::Win32::Graphics::DirectComposition::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Animation::*;

pub trait UIAnimationVariableExt {
    /// GetCurve
    fn get_curve<P0>(&self, animation: P0) -> Result<()>
    where
        P0: Param<IDCompositionAnimation>;
}

impl UIAnimationVariableExt for IUIAnimationVariable2 {
    #[inline(always)]
    fn get_curve<P0>(&self, animation: P0) -> Result<()>
    where
        P0: Param<IDCompositionAnimation>,
    {
        unsafe { self.GetCurve(animation) }
    }
}

pub trait UIAnimationManagerExt {
    fn create_animation_variable(&self, initialvalue: f64) -> Result<IUIAnimationVariable2>;
    fn update(&self, time: f64) -> Result<()>;
    fn create_storyboard(&self) -> Result<IUIAnimationStoryboard2>;
}

impl UIAnimationManagerExt for IUIAnimationManager2 {
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

pub trait UIAnimationTransitionLibraryExt {
    fn create_accelerate_decelerate_transition(
        &self,
        duration: f64,
        finalvalue: f64,
        accelerationratio: f64,
        decelerationratio: f64,
    ) -> Result<IUIAnimationTransition2>;
}

impl UIAnimationTransitionLibraryExt for IUIAnimationTransitionLibrary2 {
    #[inline(always)]
    fn create_accelerate_decelerate_transition(
        &self,
        duration: f64,
        finalvalue: f64,
        accelerationratio: f64,
        decelerationratio: f64,
    ) -> Result<IUIAnimationTransition2> {
        unsafe {
            self.CreateAccelerateDecelerateTransition(
                duration,
                finalvalue,
                accelerationratio,
                decelerationratio,
            )
        }
    }
}

pub trait UIAnimationStoryboardExt {
    fn schedule(&self, time: f64) -> Result<()>;
    fn add_transition<P0, P1>(&self, variable: P0, transition: P1) -> Result<()>
    where
        P0: Param<IUIAnimationVariable2>,
        P1: Param<IUIAnimationTransition2>;
    fn add_keyframe_after_transition<P0>(&self, transition: P0) -> Result<UI_ANIMATION_KEYFRAME>
    where
        P0: Param<IUIAnimationTransition2>;
    fn add_transition_at_keyframe<P0, P1>(
        &self,
        variable: P0,
        transition: P1,
        startkeyframe: UI_ANIMATION_KEYFRAME,
    ) -> Result<()>
    where
        P0: Param<IUIAnimationVariable2>,
        P1: Param<IUIAnimationTransition2>;
}

impl UIAnimationStoryboardExt for IUIAnimationStoryboard2 {
    #[inline(always)]
    fn schedule(&self, time: f64) -> Result<()> {
        unsafe { self.Schedule(time, None) }
    }

    #[inline(always)]
    fn add_transition<P0, P1>(&self, variable: P0, transition: P1) -> Result<()>
    where
        P0: Param<IUIAnimationVariable2>,
        P1: Param<IUIAnimationTransition2>,
    {
        unsafe { self.AddTransition(variable, transition) }
    }

    #[inline(always)]
    fn add_keyframe_after_transition<P0>(&self, transition: P0) -> Result<UI_ANIMATION_KEYFRAME>
    where
        P0: Param<IUIAnimationTransition2>,
    {
        unsafe { self.AddKeyframeAfterTransition(transition) }
    }

    #[inline(always)]
    fn add_transition_at_keyframe<P0, P1>(
        &self,
        variable: P0,
        transition: P1,
        startkeyframe: UI_ANIMATION_KEYFRAME,
    ) -> Result<()>
    where
        P0: Param<IUIAnimationVariable2>,
        P1: Param<IUIAnimationTransition2>,
    {
        unsafe { self.AddTransitionAtKeyframe(variable, transition, startkeyframe) }
    }
}
