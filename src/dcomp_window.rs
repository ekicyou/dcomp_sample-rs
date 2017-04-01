use winapi::shared::windef::HWND;
use winapi::shared::winerror::HRESULT;
use winapi::um::dcomp::*;

use super::com_rc::*;
use super::dcomp_api::*;

/*
    //------------------------------------------------------------------
    // Set up DirectComposition
    //------------------------------------------------------------------
    
    // Create the DirectComposition device
    ThrowIfFailed(DCompositionCreateDevice(
        nullptr,
        IID_PPV_ARGS(m_dcompDevice.ReleaseAndGetAddressOf())));

    // Create a DirectComposition target associated with the window (pass in hWnd here)
    ThrowIfFailed(m_dcompDevice->CreateTargetForHwnd(
        Win32Application::GetHwnd(),
        true,
        m_dcompTarget.ReleaseAndGetAddressOf()));

    // Create a DirectComposition "visual"
    ThrowIfFailed(m_dcompDevice->CreateVisual(m_dcompVisual.ReleaseAndGetAddressOf()));

    // Associate the visual with the swap chain
    ThrowIfFailed(m_dcompVisual->SetContent(swapChain.Get()));

    // Set the visual as the root of the DirectComposition target's composition tree
    ThrowIfFailed(m_dcompTarget->SetRoot(m_dcompVisual.Get()));
    ThrowIfFailed(m_dcompDevice->Commit());

    //------------------------------------------------------------------
    // DirectComposition setup end
    //------------------------------------------------------------------
*/



pub trait DCompWindow {
    fn hwnd(&self) -> HWND;
    fn create_dev(&self) -> Result<(), HRESULT> {
        let hwnd = self.hwnd();
        let dev_dcomp = create_device::<IDCompositionDevice>(None)?;

        Ok(())
    }
}

pub struct HWndProxy {
    hwnd: HWND,
}

impl HWndProxy {
    pub fn new(hwnd: HWND) -> HWndProxy {
        HWndProxy { hwnd: hwnd }
    }
}

impl DCompWindow for HWndProxy {
    fn hwnd(&self) -> HWND {
        self.hwnd
    }
}
