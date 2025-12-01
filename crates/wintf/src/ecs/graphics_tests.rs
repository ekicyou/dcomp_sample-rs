// タスク7.3: GraphicsCoreからのCOMオブジェクト作成テスト

#[cfg(test)]
mod graphics_core_tests {
    use crate::ecs::graphics::GraphicsCore;
    use windows::Win32::Graphics::Direct2D::D2D1_DEVICE_CONTEXT_OPTIONS_NONE;

    #[test]
    fn test_graphics_core_creation() {
        let _graphics = GraphicsCore::new().expect("GraphicsCore作成失敗");

        // GraphicsCoreが正常に作成されたことを確認（すべてのフィールドが初期化されている）
        println!("[TEST PASS] GraphicsCore created successfully with all valid devices");
    }

    #[test]
    fn test_create_device_context() {
        let graphics = GraphicsCore::new().expect("GraphicsCore作成失敗");

        use crate::com::d2d::D2D1DeviceExt;
        let d2d = graphics.d2d_device().expect("D2Dデバイスが無効");
        let _dc = d2d
            .create_device_context(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
            .expect("DeviceContext作成失敗");

        println!("[TEST PASS] ID2D1DeviceContext created successfully");
    }

    #[test]
    fn test_create_visual() {
        let graphics = GraphicsCore::new().expect("GraphicsCore作成失敗");

        use crate::com::dcomp::DCompositionDeviceExt;
        let dcomp = graphics.dcomp().expect("DCompositionデバイスが無効");
        let _visual = dcomp.create_visual().expect("Visual作成失敗");

        println!("[TEST PASS] IDCompositionVisual3 created successfully");
    }

    #[test]
    fn test_create_multiple_device_contexts() {
        let graphics = GraphicsCore::new().expect("GraphicsCore作成失敗");

        use crate::com::d2d::D2D1DeviceExt;
        let d2d = graphics.d2d_device().expect("D2Dデバイスが無効");

        // 複数のDeviceContextを作成できることを確認
        let _dc1 = d2d
            .create_device_context(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
            .expect("DeviceContext1作成失敗");

        let _dc2 = d2d
            .create_device_context(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
            .expect("DeviceContext2作成失敗");

        let _dc3 = d2d
            .create_device_context(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
            .expect("DeviceContext3作成失敗");

        println!("[TEST PASS] Multiple ID2D1DeviceContext created successfully");
    }

    #[test]
    fn test_create_multiple_visuals() {
        let graphics = GraphicsCore::new().expect("GraphicsCore作成失敗");

        use crate::com::dcomp::DCompositionDeviceExt;
        let dcomp = graphics.dcomp().expect("DCompositionデバイスが無効");

        // 複数のVisualを作成できることを確認
        let _v1 = dcomp.create_visual().expect("Visual1作成失敗");
        let _v2 = dcomp.create_visual().expect("Visual2作成失敗");
        let _v3 = dcomp.create_visual().expect("Visual3作成失敗");

        println!("[TEST PASS] Multiple IDCompositionVisual3 created successfully");
    }

    #[test]
    fn test_commit() {
        let graphics = GraphicsCore::new().expect("GraphicsCore作成失敗");

        use crate::com::dcomp::DCompositionDeviceExt;
        let dcomp = graphics.dcomp().expect("DCompositionデバイスが無効");

        // Commit()を呼び出せることを確認
        dcomp.commit().expect("Commit失敗");

        println!("[TEST PASS] IDCompositionDevice3::Commit() succeeded");
    }
}

// Task 3.1: HasGraphicsResources メソッドのユニットテスト
// Note: HasGraphicsResources は空マーカーに変更されたため、
// 古いテスト（needs_init, request_init, mark_initialized）は削除
// Changed<HasGraphicsResources> で再初期化トリガーを検知する設計に移行
#[cfg(test)]
mod has_graphics_resources_tests {
    use crate::ecs::graphics::HasGraphicsResources;

    #[test]
    fn test_default_is_unit_struct() {
        // 空マーカーコンポーネントとして機能することを確認
        let _res = HasGraphicsResources::default();
        // HasGraphicsResources は () と同等の空構造体
    }

    #[test]
    fn test_clone_and_partial_eq() {
        let res1 = HasGraphicsResources::default();
        let res2 = res1.clone();
        assert_eq!(res1, res2, "クローンは同一");
    }
}

// Task 3.1: SurfaceGraphicsDirty コンポーネントのユニットテスト
#[cfg(test)]
mod surface_graphics_dirty_tests {
    use crate::ecs::graphics::SurfaceGraphicsDirty;

    #[test]
    fn test_default_requested_frame_is_zero() {
        let dirty = SurfaceGraphicsDirty::default();
        assert_eq!(dirty.requested_frame, 0, "デフォルトのrequested_frameは0");
    }

    #[test]
    fn test_requested_frame_can_be_updated() {
        let mut dirty = SurfaceGraphicsDirty::default();
        dirty.requested_frame = 42;
        assert_eq!(dirty.requested_frame, 42, "requested_frameを更新できる");
    }
}
