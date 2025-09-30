#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::*;
use windows::core::*;
use wintf::*;

fn main() -> Result<()> {
    human_panic::setup_panic!();

    let mgr = WinThreadMgr::new()?;
    let _weak = Arc::downgrade(&mgr);
    mgr.run()?;
    Ok(())
}
