use winapi::shared::windef::HWND;
use winapi::shared::winerror::HRESULT;

use super::com_rc::*;
use super::dcomp_api::*;

/*
    //------------------------------------------------------------------
    // Set up DirectComposition
    //------------------------------------------------------------------
    


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
        let dc_dev = create_device::<IDCompositionDevice>(None)?;
        let dc_target = dc_dev.create_target_for_hwnd(hwnd, true)?;
        let dc_visual = dc_dev.create_visual()?;

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
