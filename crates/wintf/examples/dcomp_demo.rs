#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ambassador::*;
use std::path::*;
use std::sync::*;
use windows::{
    core::*,
    Win32::{
        Foundation::*, Graphics::Direct2D::Common::*, Graphics::Direct2D::*, Graphics::Direct3D::*,
        Graphics::Direct3D11::*, Graphics::DirectComposition::*, Graphics::DirectWrite::*,
        Graphics::Dxgi::Common::*, Graphics::Dxgi::*, Graphics::Gdi::*,
        Graphics::Imaging::*, System::Com::*, UI::Animation::*, UI::HiDpi::*, UI::Shell::*,
        UI::WindowsAndMessaging::*,
    },
};
use windows_numerics::*;
use wintf::{com::d2d::*, com::d3d11::*, com::dcomp::*, com::wic::*, *};

const CARD_ROWS: usize = 3;
const CARD_COLUMNS: usize = 6;
const CARD_MARGIN: LxLength = LxLength::new(15.0);
const CARD_WIDTH: LxLength = LxLength::new(150.0);
const CARD_HEIGHT: LxLength = LxLength::new(210.0);
const CARD_SIZE: LxSize = LxSize::new(CARD_WIDTH.0, CARD_HEIGHT.0);

const WINDOW_WIDTH: LxLength =
    LxLength::new((CARD_WIDTH.0 + CARD_MARGIN.0) * (CARD_COLUMNS as f32) + CARD_MARGIN.0);
const WINDOW_HEIGHT: LxLength =
    LxLength::new((CARD_HEIGHT.0 + CARD_MARGIN.0) * (CARD_ROWS as f32) + CARD_MARGIN.0);
const WINDOW_SIZE: LxSize = LxSize::new(WINDOW_WIDTH.0, WINDOW_HEIGHT.0);

fn main() -> Result<()> {
    human_panic::setup_panic!();

    let mgr = WinThreadMgr::new()?;
    let window = Arc::new(DemoWindow::new()?);
    let style = WinStyle::WS_OVERLAPPED()
        .WS_CAPTION(true)
        .WS_SYSMENU(true)
        .WS_MINIMIZEBOX(true)
        .WS_VISIBLE(true)
        .WS_EX_NOREDIRECTIONBITMAP(true);

    let _ = mgr.create_window(window.clone(), "Sample Window", style)?;
    println!("spawn_normal: set");
    let move_win = window.clone();
    mgr.spawn_normal(async move {
        println!("spawn_normal: execute: hwnd={:?}", move_win.hwnd());
    })
    .detach();
    mgr.run()
}

#[derive(PartialEq)]
enum Status {
    Hidden,
    Selected,
    Matched,
}

struct Card {
    status: Status,
    value: u8,
    offset: PxPoint,
    variable: IUIAnimationVariable2,
    rotation: Option<IDCompositionRotateTransform3D>,
}

#[derive(Delegate)]
#[delegate(WinState, target = "win_state")]
struct DemoWindow {
    win_state: SimpleWinState,
    format: IDWriteTextFormat,
    image: IWICFormatConverter,
    manager: IUIAnimationManager2,
    library: IUIAnimationTransitionLibrary2,
    first: Option<usize>,
    cards: Vec<Card>,
    d3d: Option<ID3D11Device>,
    dcomp: Option<IDCompositionDevice3>,
    target: Option<IDCompositionTarget>,
}

unsafe impl Send for DemoWindow {}
unsafe impl Sync for DemoWindow {}

impl WinMessageHandler for DemoWindow {
    fn WM_CREATE(&mut self, _wparam: WPARAM, _lparam: LPARAM) -> Option<LRESULT> {
        eprintln!("WM_CREATE");
        self.create_handler().expect("WM_CREATE");
        Some(LRESULT(0))
    }

    fn WM_DESTROY(&mut self, _wparam: WPARAM, _lparam: LPARAM) -> Option<LRESULT> {
        unsafe { PostQuitMessage(0) };
        Some(LRESULT(0))
    }

    fn WM_LBUTTONUP(&mut self, _wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        self.click_handler(lparam).expect("WM_LBUTTONUP");
        Some(LRESULT(0))
    }

    fn WM_PAINT(&mut self, _wparam: WPARAM, _lparam: LPARAM) -> Option<LRESULT> {
        self.paint_handler().unwrap_or_else(|_| {
            // デバイスロスはレンダリングの失敗を引き起こす可能性がありますが、
            // 致命的とは見なされるべきではありません。
            if cfg!(debug_assertions) {
                println!("WM_PAINT failed");
            }
            self.d3d = None;
        });
        Some(LRESULT(0))
    }

    fn WM_DPICHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        self.dpi_changed_handler(wparam, lparam)
            .expect("WM_DPICHANGED");
        Some(LRESULT(0))
    }
}

impl DemoWindow {
    fn new() -> Result<Self> {
        unsafe {
            let manager: IUIAnimationManager2 =
                CoCreateInstance(&UIAnimationManager2, None, CLSCTX_INPROC_SERVER)?;

            use rand::{seq::*, *};
            let mut rng = rand::rng();
            let mut values = [b'?'; CARD_ROWS * CARD_COLUMNS];

            for i in 0..values.len() / 2 {
                let value = rng.random_range(b'A'..=b'Z');
                values[i * 2] = value;
                values[i * 2 + 1] = value + b'a' - b'A';
            }

            values.shuffle(&mut rng);
            let mut cards = Vec::new();

            for value in values {
                cards.push(Card {
                    status: Status::Hidden,
                    value,
                    offset: Default::default(),
                    variable: manager.CreateAnimationVariable(0.0)?,
                    rotation: None,
                });
            }

            if cfg!(debug_assertions) {
                println!("deck:");
                for row in 0..CARD_ROWS {
                    for column in 0..CARD_COLUMNS {
                        print!(
                            " {}",
                            char::from_u32(cards[row * CARD_COLUMNS + column].value as u32)
                                .expect("char")
                        );
                    }
                    println!();
                }
            }

            let library =
                CoCreateInstance(&UIAnimationTransitionLibrary2, None, CLSCTX_INPROC_SERVER)?;

            Ok(DemoWindow {
                win_state: Default::default(),
                format: create_text_format()?,
                image: create_image()?,
                manager,
                library,
                first: None,
                cards,
                d3d: None,
                dcomp: None,
                target: None,
            })
        }
    }

    fn create_device_resources(&mut self) -> Result<()> {
        unsafe {
            debug_assert!(self.d3d.is_none());
            let d3d = create_device_3d()?;
            let dxgi = d3d.cast()?;
            let d2d = d2d_create_device(&dxgi)?;
            self.d3d = Some(d3d);
            let desktop = dcomp_create_desktop_device(&d2d)?;
            let dcomp = desktop.cast()?;

            // 以前のターゲットを最初にリリースします。そうしないと `CreateTargetForHwnd` が HWND が占有されていることを検出します。
            self.target = None;
            let target = desktop.create_target_for_hwnd(self.hwnd(), true)?;
            let root_visual = create_visual(&dcomp)?;
            target.SetRoot(&root_visual)?;
            self.target = Some(target);

            let dc = d2d.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;

            let brush = dc.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                None,
            )?;

            let bitmap = dc.CreateBitmapFromWicBitmap(&self.image, None)?;
            let dpi = self.dpi();
            let card_size: PxSize = CARD_SIZE.into_dpi(dpi);
            let card_size = card_size.into_raw();

            for row in 0..CARD_ROWS {
                for column in 0..CARD_COLUMNS {
                    let card = &mut self.cards[row * CARD_COLUMNS + column];
                    let offset = LxPoint::from_lengths(
                        (CARD_WIDTH + CARD_MARGIN) * (column as f32) + CARD_MARGIN,
                        (CARD_HEIGHT + CARD_MARGIN) * (row as f32) + CARD_MARGIN,
                    );
                    card.offset = offset.into_dpi(dpi);

                    if card.status == Status::Matched {
                        continue;
                    }

                    let front_visual = create_visual(&dcomp)?;
                    front_visual.SetOffsetX2(card.offset.x)?;
                    front_visual.SetOffsetY2(card.offset.y)?;
                    root_visual.AddVisual(&front_visual, false, None)?;

                    let back_visual = create_visual(&dcomp)?;
                    back_visual.SetOffsetX2(card.offset.x)?;
                    back_visual.SetOffsetY2(card.offset.y)?;
                    root_visual.AddVisual(&back_visual, false, None)?;

                    let front_surface = create_surface(&dcomp, card_size)?;
                    front_visual.SetContent(&front_surface)?;
                    draw_card_front(&front_surface, card.value, &self.format, &brush, dpi)?;

                    let back_surface = create_surface(&dcomp, card_size)?;
                    back_visual.SetContent(&back_surface)?;
                    draw_card_back(&back_surface, &bitmap, card.offset, dpi)?;

                    let rotation = dcomp.CreateRotateTransform3D()?;

                    if card.status == Status::Selected {
                        rotation.SetAngle2(180.0)?;
                    }

                    rotation.SetAxisZ2(0.0)?;
                    rotation.SetAxisY2(1.0)?;
                    create_effect(&dcomp, &front_visual, &rotation, true, dpi)?;
                    create_effect(&dcomp, &back_visual, &rotation, false, dpi)?;
                    card.rotation = Some(rotation);
                }
            }

            dcomp.Commit()?;
            self.dcomp = Some(dcomp);
            Ok(())
        }
    }

    fn click_handler(&mut self, lparam: LPARAM) -> Result<()> {
        let dpi = self.dpi();
        unsafe {
            let x = lparam.0 as u16 as f32;
            let y = (lparam.0 >> 16) as f32;

            let width: PxLength = CARD_WIDTH.into_dpi(dpi);
            let height: PxLength = CARD_HEIGHT.into_dpi(dpi);
            let mut next = None;

            for (index, card) in self.cards.iter().enumerate() {
                if x > card.offset.x
                    && y > card.offset.y
                    && x < card.offset.x + width.0
                    && y < card.offset.y + height.0
                {
                    next = Some(index);
                    break;
                }
            }

            if let Some(next) = next {
                if Some(next) == self.first {
                    if cfg!(debug_assertions) {
                        println!("same card");
                    }
                    return Ok(());
                }

                if self.cards[next].status == Status::Matched {
                    if cfg!(debug_assertions) {
                        println!("previous match");
                    }
                    return Ok(());
                }

                let dcomp = self.dcomp.as_ref().expect("IDCompositionDesktopDevice");
                let stats = dcomp.GetFrameStatistics()?;

                let next_frame: f64 =
                    stats.nextEstimatedFrameTime as f64 / stats.timeFrequency as f64;

                self.manager.Update(next_frame, None)?;
                let storyboard = self.manager.CreateStoryboard()?;
                let key_frame = add_show_transition(&self.library, &storyboard, &self.cards[next])?;

                if let Some(first) = self.first.take() {
                    let final_value = if b'a' - b'A'
                        == u8::abs_diff(self.cards[first].value, self.cards[next].value)
                    {
                        self.cards[first].status = Status::Matched;
                        self.cards[next].status = Status::Matched;
                        90.0
                    } else {
                        self.cards[first].status = Status::Hidden;
                        0.0
                    };

                    add_hide_transition(
                        &self.library,
                        &storyboard,
                        key_frame,
                        final_value,
                        &self.cards[first],
                    )?;

                    add_hide_transition(
                        &self.library,
                        &storyboard,
                        key_frame,
                        final_value,
                        &self.cards[next],
                    )?;

                    storyboard.Schedule(next_frame, None)?;
                    update_animation(dcomp, &self.cards[first])?;
                    update_animation(dcomp, &self.cards[next])?;
                } else {
                    self.first = Some(next);
                    self.cards[next].status = Status::Selected;
                    storyboard.Schedule(next_frame, None)?;
                    update_animation(dcomp, &self.cards[next])?;
                }

                dcomp.Commit()?;
            } else if cfg!(debug_assertions) {
                println!("missed");
            }

            Ok(())
        }
    }

    fn paint_handler(&mut self) -> Result<()> {
        unsafe {
            if let Some(device) = &self.d3d {
                if cfg!(debug_assertions) {
                    println!("check device");
                }
                device.GetDeviceRemovedReason()?;
            } else {
                if cfg!(debug_assertions) {
                    println!("build device");
                }
                self.create_device_resources()?;
            }

            ValidateRect(Some(self.hwnd()), None).ok()
        }
    }

    fn dpi_changed_handler(&mut self, wparam: WPARAM, lparam: LPARAM) -> Result<()> {
        unsafe {
            self.set_dpi_change_message(wparam, lparam);

            if cfg!(debug_assertions) {
                println!("dpi changed: {:?}", self.dpi());
            }

            let rect = &*(lparam.0 as *const RECT);
            let size = self.effective_window_size(WINDOW_SIZE)?.into_raw();

            SetWindowPos(
                self.hwnd(),
                None,
                rect.left,
                rect.top,
                size.width,
                size.height,
                SWP_NOACTIVATE | SWP_NOZORDER,
            )?;

            self.d3d = None;
            Ok(())
        }
    }

    fn create_handler(&mut self) -> Result<()> {
        unsafe {
            let monitor = MonitorFromWindow(self.hwnd(), MONITOR_DEFAULTTONEAREST);
            let mut dpi = (0, 0);
            GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi.0, &mut dpi.1)?;
            self.set_dpi(Dpi::new(dpi.0 as f32));

            if cfg!(debug_assertions) {
                println!("initial dpi: {:?}", self.dpi());
            }

            let size = self.effective_window_size(WINDOW_SIZE)?.into_raw();

            SetWindowPos(
                self.hwnd(),
                None,
                0,
                0,
                size.width,
                size.height,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOZORDER,
            )
        }
    }
}

fn create_text_format() -> Result<IDWriteTextFormat> {
    unsafe {
        let factory: IDWriteFactory2 = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;

        let format = factory.CreateTextFormat(
            w!("Candara"),
            None,
            DWRITE_FONT_WEIGHT_NORMAL,
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            CARD_HEIGHT.0 / 2.0,
            w!("en"),
        )?;

        format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER)?;
        format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;
        Ok(format)
    }
}

fn create_image() -> Result<IWICFormatConverter> {
    let factory = wic_factory()?;
    let path = if Path::new("dcomp_demo.jpg").exists() {
        h!("dcomp_demo.jpg")
    } else {
        h!("crates/wintf/examples/dcomp_demo.jpg")
    };
    let decoder = factory.create_decoder_from_filename(
        path,
        None,
        GENERIC_READ,
        WICDecodeMetadataCacheOnDemand,
    )?;
    let source = decoder.frame(0)?;
    let image = factory.create_format_converter()?;
    image.init(
        &source,
        &GUID_WICPixelFormat32bppBGR,
        WICBitmapDitherTypeNone,
        None,
        0.0,
        WICBitmapPaletteTypeMedianCut,
    )?;
    Ok(image)
}

fn create_device_3d() -> Result<ID3D11Device> {
    d3d11_create_device(
        None,
        D3D_DRIVER_TYPE_HARDWARE,
        HMODULE::default(),
        D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        None,
        D3D11_SDK_VERSION,
        None,
        None,
    )
}

fn create_visual(dcomp: &IDCompositionDevice3) -> Result<IDCompositionVisual3> {
    unsafe {
        let visual = dcomp.CreateVisual()?;
        visual.SetBackFaceVisibility(DCOMPOSITION_BACKFACE_VISIBILITY_HIDDEN)?;
        Ok(visual.cast()?)
    }
}

fn create_surface(dcomp: &IDCompositionDevice3, size: RawSize) -> Result<IDCompositionSurface> {
    unsafe {
        dcomp.CreateSurface(
            size.width as u32,
            size.height as u32,
            DXGI_FORMAT_B8G8R8A8_UNORM,
            DXGI_ALPHA_MODE_PREMULTIPLIED,
        )
    }
}

fn add_show_transition(
    library: &IUIAnimationTransitionLibrary2,
    storyboard: &IUIAnimationStoryboard2,
    card: &Card,
) -> Result<UI_ANIMATION_KEYFRAME> {
    unsafe {
        let duration = (180.0 - card.variable.GetValue()?) / 180.0;
        let transition = create_transition(library, duration, 180.0)?;
        storyboard.AddTransition(&card.variable, &transition)?;
        storyboard.AddKeyframeAfterTransition(&transition)
    }
}

fn add_hide_transition(
    library: &IUIAnimationTransitionLibrary2,
    storyboard: &IUIAnimationStoryboard2,
    key_frame: UI_ANIMATION_KEYFRAME,
    final_value: f64,
    card: &Card,
) -> Result<()> {
    unsafe {
        let transition = create_transition(library, 1.0, final_value)?;
        storyboard.AddTransitionAtKeyframe(&card.variable, &transition, key_frame)
    }
}

fn update_animation(dcomp: &IDCompositionDevice3, card: &Card) -> Result<()> {
    unsafe {
        // 1. 空の DirectComposition アニメーションを作成
        let animation = dcomp.CreateAnimation()?;

        // 2. UI Animation 変数のカーブを DComp アニメーションへコピー
        card.variable.GetCurve(&animation)?;

        // 3. 回転トランスフォームの Angle にセット（以後 DComp 側で自動進行）
        card.rotation
            .as_ref()
            .expect("IDCompositionRotateTransform3D")
            .SetAngle(&animation)
    }
}

fn create_transition(
    library: &IUIAnimationTransitionLibrary2,
    duration: f64,
    final_value: f64,
) -> Result<IUIAnimationTransition2> {
    unsafe { library.CreateAccelerateDecelerateTransition(duration, final_value, 0.2, 0.8) }
}

fn create_effect(
    dcomp: &IDCompositionDevice3,
    visual: &IDCompositionVisual3,
    rotation: &IDCompositionRotateTransform3D,
    front: bool,
    dpi: Dpi,
) -> Result<()> {
    unsafe {
        let width: PxLength = CARD_WIDTH.into_dpi(dpi);
        let height: PxLength = CARD_HEIGHT.into_dpi(dpi);

        let pre_matrix = Matrix4x4::translation(-width.0 / 2.0, -height.0 / 2.0, 0.0)
            * Matrix4x4::rotation_y(if front { 180.0 } else { 0.0 });

        let pre_transform = dcomp.CreateMatrixTransform3D()?;
        pre_transform.SetMatrix(&pre_matrix)?;

        let post_matrix = Matrix4x4::perspective_projection(width.0 * 2.0)
            * Matrix4x4::translation(width.0 / 2.0, height.0 / 2.0, 0.0);

        let post_transform = dcomp.CreateMatrixTransform3D()?;
        post_transform.SetMatrix(&post_matrix)?;

        let transform = dcomp.CreateTransform3DGroup(&[
            pre_transform.cast().ok(),
            rotation.cast().ok(),
            post_transform.cast().ok(),
        ])?;

        visual.SetEffect(&transform)
    }
}

fn draw_card_front(
    surface: &IDCompositionSurface,
    value: u8,
    format: &IDWriteTextFormat,
    brush: &ID2D1SolidColorBrush,
    dpi: Dpi,
) -> Result<()> {
    unsafe {
        let mut offset = Default::default();
        let dc: ID2D1DeviceContext = surface.BeginDraw(None, &mut offset)?;
        dc.set_dpi(dpi);
        let offset: LxPoint = PxPoint::new(offset.x as f32, offset.y as f32).into_dpi(dpi);
        dc.SetTransform(&Matrix3x2::translation(offset.x, offset.y));

        dc.Clear(Some(&D2D1_COLOR_F {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }));

        dc.DrawText(
            &[value as _],
            format,
            &D2D_RECT_F {
                left: 0.0,
                top: 0.0,
                right: CARD_WIDTH.0,
                bottom: CARD_HEIGHT.0,
            },
            brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );

        surface.EndDraw()
    }
}

fn draw_card_back(
    surface: &IDCompositionSurface,
    bitmap: &ID2D1Bitmap1,
    offset: PxPoint,
    dpi: Dpi,
) -> Result<()> {
    unsafe {
        let mut dc_offset = Default::default();
        let dc: ID2D1DeviceContext = surface.BeginDraw(None, &mut dc_offset)?;
        let dc: ID2D1DeviceContext7 = dc.cast()?;
        dc.set_dpi(dpi);
        let dc_offset: LxPoint = PxPoint::new(dc_offset.x as f32, dc_offset.y as f32).into_dpi(dpi);
        dc.SetTransform(&Matrix3x2::translation(dc_offset.x, dc_offset.y));

        let offset: LxPoint = offset.into_dpi(dpi);
        let left = offset.x;
        let top = offset.y;

        dc.DrawBitmap(
            bitmap,
            None,
            1.0,
            D2D1_INTERPOLATION_MODE_LINEAR,
            Some(&D2D_RECT_F {
                left,
                top,
                right: left + CARD_WIDTH.0,
                bottom: top + CARD_HEIGHT.0,
            }),
            None,
        );

        surface.EndDraw()
    }
}
