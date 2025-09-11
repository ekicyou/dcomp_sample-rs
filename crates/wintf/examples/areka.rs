#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::*;
use windows::{
    core::*,
    Win32::{
        Foundation::*, Graphics::Direct2D::Common::*, Graphics::Direct2D::*, Graphics::Direct3D::*,
        Graphics::Direct3D11::*, Graphics::DirectComposition::*, Graphics::DirectWrite::*,
        Graphics::Dxgi::Common::*, Graphics::Dxgi::*, Graphics::Gdi::*, Graphics::Imaging::D2D::*,
        Graphics::Imaging::*, System::Com::*, UI::Animation::*, UI::HiDpi::*, UI::Shell::*,
        UI::WindowsAndMessaging::*,
    },
};
use windows_numerics::*;
use wintf::*;

fn main() -> Result<()> {
    human_panic::setup_panic!();

    let mgr = WinThreadMgr::new()?;
    let weak = Arc::downgrade(&mgr);
    mgr.run()?;
    Ok(())
}
