use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::DirectComposition::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Animation::*;
use windows_numerics::*;

pub fn dcomp_create_desktop_device<P0>(renderingdevice: P0) -> Result<IDCompositionDesktopDevice>
where
    P0: Param<IUnknown>,
{
    unsafe { DCompositionCreateDevice3(renderingdevice) }
}

pub trait DCompositionDesktopDeviceExt {
    /// CreateTargetForHwnd
    fn create_target_for_hwnd(&self, hwnd: HWND, topmost: bool) -> Result<IDCompositionTarget>;
}

impl DCompositionDesktopDeviceExt for IDCompositionDesktopDevice {
    #[inline(always)]
    fn create_target_for_hwnd(&self, hwnd: HWND, topmost: bool) -> Result<IDCompositionTarget> {
        unsafe { self.CreateTargetForHwnd(hwnd, topmost) }
    }
}

pub trait IDCompositionTargetExt {
    /// SetRoot
    fn set_root<P0>(&self, visual: P0) -> Result<()>
    where
        P0: Param<IDCompositionVisual>;
}

impl IDCompositionTargetExt for IDCompositionTarget {
    #[inline(always)]
    fn set_root<P0>(&self, visual: P0) -> Result<()>
    where
        P0: Param<IDCompositionVisual>,
    {
        unsafe { self.SetRoot(visual) }
    }
}

pub trait IDCompositionVisualExt {
    /// SetBackFaceVisibility
    fn set_backface_visibility(&self, visibility: DCOMPOSITION_BACKFACE_VISIBILITY) -> Result<()>;
    /// SetOffsetX
    fn set_offset_x(&self, offset: f32) -> Result<()>;
    /// SetOffsetY
    fn set_offset_y(&self, offset: f32) -> Result<()>;
    /// AddVisual
    fn add_visual<P0, P1>(&self, visual: P0, insertabove: bool, referencevisual: P1) -> Result<()>
    where
        P0: Param<IDCompositionVisual>,
        P1: Param<IDCompositionVisual>;
    /// SetContent
    fn set_content<P0>(&self, content: P0) -> Result<()>
    where
        P0: Param<IUnknown>;
    /// SetEffect
    fn set_effect<P0>(&self, effect: P0) -> Result<()>
    where
        P0: Param<IDCompositionEffect>;
}

impl IDCompositionVisualExt for IDCompositionVisual3 {
    #[inline(always)]
    fn set_backface_visibility(&self, visibility: DCOMPOSITION_BACKFACE_VISIBILITY) -> Result<()> {
        unsafe { self.SetBackFaceVisibility(visibility) }
    }

    #[inline(always)]
    fn set_offset_x(&self, offset: f32) -> Result<()> {
        unsafe { self.SetOffsetX2(offset) }
    }

    #[inline(always)]
    fn set_offset_y(&self, offset: f32) -> Result<()> {
        unsafe { self.SetOffsetY2(offset) }
    }

    #[inline(always)]
    fn add_visual<P0, P1>(&self, visual: P0, insertabove: bool, referencevisual: P1) -> Result<()>
    where
        P0: Param<IDCompositionVisual>,
        P1: Param<IDCompositionVisual>,
    {
        unsafe { self.AddVisual(visual, insertabove, referencevisual) }
    }

    #[inline(always)]
    fn set_content<P0>(&self, content: P0) -> Result<()>
    where
        P0: Param<IUnknown>,
    {
        unsafe { self.SetContent(content) }
    }

    #[inline(always)]
    fn set_effect<P0>(&self, effect: P0) -> Result<()>
    where
        P0: Param<IDCompositionEffect>,
    {
        unsafe { self.SetEffect(effect) }
    }
}

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

pub trait IDCompositionRotateTransform3DExt {
    /// SetAngle
    fn set_angle<P0>(&self, animation: P0) -> Result<()>
    where
        P0: Param<IDCompositionAnimation>;
    /// SetAngle2
    fn set_angle2(&self, angle: f32) -> Result<()>;
    /// SetAxisZ2
    fn set_axis_z(&self, z: f32) -> Result<()>;
    /// SetAxisY2
    fn set_axis_y(&self, y: f32) -> Result<()>;
}

impl IDCompositionRotateTransform3DExt for IDCompositionRotateTransform3D {
    #[inline(always)]
    fn set_angle<P0>(&self, animation: P0) -> Result<()>
    where
        P0: Param<IDCompositionAnimation>,
    {
        unsafe { self.SetAngle(animation) }
    }

    #[inline(always)]
    fn set_angle2(&self, angle: f32) -> Result<()> {
        unsafe { self.SetAngle2(angle) }
    }

    #[inline(always)]
    fn set_axis_z(&self, z: f32) -> Result<()> {
        unsafe { self.SetAxisZ2(z) }
    }

    #[inline(always)]
    fn set_axis_y(&self, y: f32) -> Result<()> {
        unsafe { self.SetAxisY2(y) }
    }
}

pub trait IDCompositionMatrixTransform3DExt {
    /// SetMatrix
    fn set_matrix(&self, matrix: &Matrix4x4) -> Result<()>;
}

impl IDCompositionMatrixTransform3DExt for IDCompositionMatrixTransform3D {
    #[inline(always)]
    fn set_matrix(&self, matrix: &Matrix4x4) -> Result<()> {
        unsafe { self.SetMatrix(matrix) }
    }
}

pub trait IDCompositionDeviceExt {
    /// CreateVisual
    fn create_visual(&self) -> Result<IDCompositionVisual3>;
    /// Commit
    fn commit(&self) -> Result<()>;
    /// CreateSurface
    fn create_surface(
        &self,
        width: u32,
        height: u32,
        pixelformat: DXGI_FORMAT,
        alphamode: DXGI_ALPHA_MODE,
    ) -> Result<IDCompositionSurface>;
    /// GetFrameStatistics
    fn get_frame_statistics(&self) -> Result<DCOMPOSITION_FRAME_STATISTICS>;
    /// CreateAnimation
    fn create_animation(&self) -> Result<IDCompositionAnimation>;
    /// CreateMatrixTransform3D
    fn create_matrix_transform_3d(&self) -> Result<IDCompositionMatrixTransform3D>;
    /// CreateTransform3DGroup
    fn create_transform_3d_group(
        &self,
        transforms: &[Option<IDCompositionTransform3D>],
    ) -> Result<IDCompositionTransform3D>;
    fn create_rotate_transform_3d(&self) -> Result<IDCompositionRotateTransform3D>;
}

impl IDCompositionDeviceExt for IDCompositionDevice3 {
    #[inline(always)]
    fn create_visual(&self) -> Result<IDCompositionVisual3> {
        unsafe { self.CreateVisual()?.cast() }
    }

    #[inline(always)]
    fn commit(&self) -> Result<()> {
        unsafe { self.Commit() }
    }

    #[inline(always)]
    fn create_surface(
        &self,
        width: u32,
        height: u32,
        pixelformat: DXGI_FORMAT,
        alphamode: DXGI_ALPHA_MODE,
    ) -> Result<IDCompositionSurface> {
        unsafe { self.CreateSurface(width, height, pixelformat, alphamode) }
    }

    #[inline(always)]
    fn get_frame_statistics(&self) -> Result<DCOMPOSITION_FRAME_STATISTICS> {
        unsafe { self.GetFrameStatistics() }
    }

    #[inline(always)]
    fn create_animation(&self) -> Result<IDCompositionAnimation> {
        unsafe { self.CreateAnimation() }
    }

    #[inline(always)]
    fn create_matrix_transform_3d(&self) -> Result<IDCompositionMatrixTransform3D> {
        unsafe { self.CreateMatrixTransform3D() }
    }

    #[inline(always)]
    fn create_transform_3d_group(
        &self,
        transforms: &[Option<IDCompositionTransform3D>],
    ) -> Result<IDCompositionTransform3D> {
        unsafe { self.CreateTransform3DGroup(transforms) }
    }

    #[inline(always)]
    fn create_rotate_transform_3d(&self) -> Result<IDCompositionRotateTransform3D> {
        unsafe { self.CreateRotateTransform3D() }
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
