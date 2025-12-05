//! BitmapSource システム
//!
//! 非同期画像読み込み、パス解決、D2D描画システム。

use super::resource::{BitmapSourceGraphics, BitmapSourceResource};
use super::task_pool::WintfTaskPool;
use crate::com::d2d::D2D1CommandListExt;
use crate::com::wic::{WICBitmapDecoderExt, WICFormatConverterExt, WICImagingFactoryExt};
use crate::ecs::graphics::{format_entity_name, GraphicsCommandList, GraphicsCore};
use crate::ecs::layout::Arrangement;
use bevy_ecs::name::Name;
use bevy_ecs::prelude::*;
use std::path::{Path, PathBuf};
use tracing::{trace, warn};
use windows::core::{Interface, Result};
use windows::Win32::Foundation::GENERIC_READ;
use windows::Win32::Graphics::Direct2D::Common::D2D1_PIXEL_FORMAT;
use windows::Win32::Graphics::Direct2D::{
    ID2D1DeviceContext, D2D1_BITMAP_OPTIONS_NONE, D2D1_BITMAP_PROPERTIES1,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Imaging::D2D::IWICImagingFactory2;
use windows::Win32::Graphics::Imaging::{
    GUID_WICPixelFormat32bppPBGRA, IWICBitmapSource, WICBitmapDitherTypeNone,
    WICBitmapPaletteTypeMedianCut, WICDecodeMetadataCacheOnDemand,
};

// ============================================================
// パス解決
// ============================================================

/// パス解決: 実行ファイル基準
///
/// wintfシステム全体の思想として、相対パスは実行ファイルの
/// ディレクトリを基準とする。カレントディレクトリは実行時に
/// 変動する可能性があるため、使用しない。
pub fn resolve_path(path: &str) -> std::io::Result<PathBuf> {
    let path = Path::new(path);

    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        // 実行ファイルのディレクトリを基準に解決
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "exe directory not found")
        })?;
        Ok(exe_dir.join(path))
    }
}

/// テスト用アセットパス解決（CARGO_MANIFEST_DIR基準）
#[cfg(test)]
pub fn test_asset_path(name: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("tests")
        .join("assets")
        .join(name)
}

// ============================================================
// WIC画像読み込み
// ============================================================

/// WICで画像を読み込み、PBGRA32形式のBitmapSourceを返す
///
/// # Arguments
/// * `factory` - WICファクトリ
/// * `path` - 画像ファイルパス
///
/// # Returns
/// PBGRA32形式に変換されたIWICBitmapSource
pub fn load_bitmap_source(factory: &IWICImagingFactory2, path: &Path) -> Result<IWICBitmapSource> {
    use windows::core::HSTRING;

    // パスをHSTRINGに変換
    let path_str = path.to_string_lossy();
    let path_hstring = HSTRING::from(path_str.as_ref());

    // デコーダー作成
    let decoder = factory.create_decoder_from_filename(
        &path_hstring,
        None,
        GENERIC_READ,
        WICDecodeMetadataCacheOnDemand,
    )?;

    // 最初のフレームを取得
    let frame = decoder.frame(0)?;

    // PBGRA32に変換（αチャネルがなくても100%不透明として変換）
    let converter = factory.create_format_converter()?;
    converter.init(
        &frame.cast::<IWICBitmapSource>()?,
        &GUID_WICPixelFormat32bppPBGRA,
        WICBitmapDitherTypeNone,
        None,
        0.0,
        WICBitmapPaletteTypeMedianCut,
    )?;

    converter.cast()
}

// ============================================================
// ECSコマンド
// ============================================================

/// 画像読み込み完了時にBitmapSourceResourceを挿入するCommand
pub struct InsertBitmapSourceResource {
    pub entity: Entity,
    pub source: IWICBitmapSource,
}

// IWICBitmapSourceはthread-free marshaling対応
unsafe impl Send for InsertBitmapSourceResource {}

impl Command for InsertBitmapSourceResource {
    fn apply(self, world: &mut World) {
        // エンティティが存在するか確認（読み込み中にdespawnされた場合の対応）
        if let Ok(mut entity_ref) = world.get_entity_mut(self.entity) {
            entity_ref.insert(BitmapSourceResource::new(self.source));
        }
    }
}

// ============================================================
// 描画システム
// ============================================================

/// BitmapSourceResourceからD2D Bitmapを生成し、GraphicsCommandListに描画コマンドを出力
///
/// - BitmapSourceGraphics.is_valid() == falseの場合、D2D Bitmapを生成
/// - GraphicsCommandListに描画コマンドを出力（OFFSET(0,0)から描画）
pub fn draw_bitmap_sources(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &BitmapSourceResource,
            &mut BitmapSourceGraphics,
            &Arrangement,
            Option<&GraphicsCommandList>,
            Option<&Name>,
        ),
        Or<(Changed<BitmapSourceResource>, Changed<Arrangement>)>,
    >,
    graphics_core: Option<Res<GraphicsCore>>,
) {
    let Some(graphics_core) = graphics_core else {
        warn!("GraphicsCore not available, skipping draw_bitmap_sources");
        return;
    };

    for (entity, resource, mut graphics, arrangement, cmd_list_opt, name) in query.iter_mut() {
        let entity_name = format_entity_name(entity, name);
        trace!(
            entity = %entity_name,
            width = arrangement.size.width,
            height = arrangement.size.height,
            "Drawing bitmap source"
        );

        // D2D Bitmap生成（未生成または無効化されている場合）
        if !graphics.is_valid() {
            let dc = match graphics_core.device_context() {
                Some(dc) => dc,
                None => {
                    warn!(entity = %entity_name, "DeviceContext not available");
                    continue;
                }
            };

            match create_d2d_bitmap(dc, resource.source()) {
                Ok(bitmap) => {
                    graphics.set_bitmap(bitmap);
                }
                Err(e) => {
                    warn!(
                        entity = %entity_name,
                        error = ?e,
                        "Failed to create D2D bitmap"
                    );
                    continue;
                }
            }
        }

        // GraphicsCommandList生成
        let dc = match graphics_core.device_context() {
            Some(dc) => dc,
            None => continue,
        };

        let command_list = match unsafe { dc.CreateCommandList() } {
            Ok(cl) => cl,
            Err(e) => {
                warn!(
                    entity = %entity_name,
                    error = ?e,
                    "Failed to create CommandList"
                );
                continue;
            }
        };

        // DeviceContextのターゲットをCommandListに設定
        unsafe {
            dc.SetTarget(&command_list);
        }

        // 描画命令を記録
        unsafe {
            dc.BeginDraw();

            if let Some(bitmap) = graphics.bitmap() {
                // OFFSET(0,0)から描画
                use crate::com::d2d::D2D1DeviceContextExt;
                dc.draw_image(bitmap);
            }

            let _ = dc.EndDraw(None, None);
        }

        // CommandListを閉じる
        if let Err(err) = command_list.close() {
            warn!(
                entity = %entity_name,
                error = ?err,
                "Failed to close CommandList"
            );
            continue;
        }

        // GraphicsCommandListをエンティティに挿入/更新
        let new_cmd_list = GraphicsCommandList::new(command_list);
        match cmd_list_opt {
            Some(existing) if *existing != new_cmd_list => {
                commands.entity(entity).insert(new_cmd_list);
            }
            None => {
                commands.entity(entity).insert(new_cmd_list);
            }
            _ => {}
        }
    }
}

/// WIC BitmapSourceからD2D Bitmapを作成
fn create_d2d_bitmap(
    dc: &ID2D1DeviceContext,
    source: &IWICBitmapSource,
) -> Result<windows::Win32::Graphics::Direct2D::ID2D1Bitmap1> {
    use std::mem::ManuallyDrop;
    use windows::Win32::Graphics::Direct2D::ID2D1ColorContext;

    let props = D2D1_BITMAP_PROPERTIES1 {
        pixelFormat: D2D1_PIXEL_FORMAT {
            format: DXGI_FORMAT_B8G8R8A8_UNORM,
            alphaMode: windows::Win32::Graphics::Direct2D::Common::D2D1_ALPHA_MODE_PREMULTIPLIED,
        },
        dpiX: 96.0,
        dpiY: 96.0,
        bitmapOptions: D2D1_BITMAP_OPTIONS_NONE,
        colorContext: ManuallyDrop::new(None::<ID2D1ColorContext>),
    };

    unsafe { dc.CreateBitmapFromWicBitmap(source, Some(&props)) }
}

// ============================================================
// drain_task_pool_commands システム
// ============================================================

/// WintfTaskPoolからコマンドをドレインしてWorldに適用
///
/// Inputスケジュールで実行される。
///
/// # 重要な設計
/// コマンド実行中も`WintfTaskPool`がWorld内に存在する必要がある。
/// なぜなら、コマンド内で`BitmapSource`をspawnした場合、
/// `on_bitmap_source_add`フックが`WintfTaskPool.spawn()`を呼び出すため。
/// リソースが取り除かれていると、画像読み込みタスクが起動されない。
pub fn drain_task_pool_commands(world: &mut World) {
    // まずコマンドをVecに収集（WintfTaskPoolをWorld内に保持したまま）
    let commands: Vec<_> = if let Some(task_pool) = world.get_resource::<WintfTaskPool>() {
        task_pool.drain_commands()
    } else {
        Vec::new()
    };

    // コマンドを実行（WintfTaskPoolはWorld内に存在する）
    for cmd in commands {
        cmd(world);
    }
}

// ============================================================
// αマスク生成システム
// ============================================================

use super::alpha_mask::AlphaMask;
use crate::com::wic::WICBitmapSourceExt;
use crate::ecs::layout::{HitTest, HitTestMode};
use tracing::error;

/// αマスク生成完了時にBitmapSourceResourceにαマスクを設定するCommand
struct SetAlphaMaskCommand {
    entity: Entity,
    mask: AlphaMask,
}

impl Command for SetAlphaMaskCommand {
    fn apply(self, world: &mut World) {
        // エンティティが存在し、BitmapSourceResourceを持っているか確認
        if let Ok(mut entity_ref) = world.get_entity_mut(self.entity) {
            if let Some(mut resource) = entity_ref.get_mut::<BitmapSourceResource>() {
                resource.set_alpha_mask(self.mask);
            }
        }
    }
}

/// αマスク生成システム
///
/// BitmapSourceResourceが追加された時、HitTestMode::AlphaMaskの場合のみ
/// 非同期でαマスクを生成する。
///
/// # Trigger
/// - `Added<BitmapSourceResource>` + `With<HitTest>`
/// - `HitTestMode::AlphaMask` の場合のみ実行
///
/// # Flow
/// 1. WIC BitmapSource からピクセルデータを取得（同期）
/// 2. 非同期タスクでAlphaMask::from_pbgra32() でマスク生成
/// 3. Command 経由で BitmapSourceResource.alpha_mask に設定
pub fn generate_alpha_mask_system(
    query: Query<(Entity, &BitmapSourceResource, &HitTest), Added<BitmapSourceResource>>,
    task_pool: Option<Res<WintfTaskPool>>,
) {
    let Some(task_pool) = task_pool else {
        return;
    };

    for (entity, resource, hit_test) in query.iter() {
        // HitTestMode::AlphaMask 以外はスキップ
        if hit_test.mode != HitTestMode::AlphaMask {
            continue;
        }

        // 既にαマスクが生成済みの場合はスキップ
        if resource.alpha_mask().is_some() {
            continue;
        }

        // 同期でピクセルデータを取得（IWICBitmapSourceはSendでないため）
        let source = resource.source();

        // 画像サイズを取得
        let (width, height) = match source.get_size() {
            Ok(size) => size,
            Err(e) => {
                error!(entity = ?entity, error = ?e, "Failed to get bitmap size for alpha mask");
                continue;
            }
        };

        // ピクセルデータを取得
        let stride = width * 4; // PBGRA32 = 4 bytes/pixel
        let buffer_size = (stride * height) as usize;
        let mut buffer = vec![0u8; buffer_size];

        if let Err(e) = source.copy_pixels(None, stride, &mut buffer) {
            error!(entity = ?entity, error = ?e, "Failed to copy pixels for alpha mask");
            continue;
        }

        // 非同期でαマスクを生成（ピクセルデータはSend可能）
        task_pool.spawn(move |tx| async move {
            // αマスクを生成
            let mask = AlphaMask::from_pbgra32(&buffer, width, height, stride);

            // Commandを送信してBitmapSourceResourceにαマスクを設定
            let cmd: super::task_pool::BoxedCommand = Box::new(move |world: &mut World| {
                SetAlphaMaskCommand { entity, mask }.apply(world);
            });
            let _ = tx.send(cmd);
        });
    }
}
