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
        let _dc = graphics
            .d2d
            .create_device_context(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
            .expect("DeviceContext作成失敗");
        
        println!("[TEST PASS] ID2D1DeviceContext created successfully");
    }

    #[test]
    fn test_create_visual() {
        let graphics = GraphicsCore::new().expect("GraphicsCore作成失敗");
        
        use crate::com::dcomp::DCompositionDeviceExt;
        let _visual = graphics.dcomp.create_visual().expect("Visual作成失敗");
        
        println!("[TEST PASS] IDCompositionVisual3 created successfully");
    }

    #[test]
    fn test_create_multiple_device_contexts() {
        let graphics = GraphicsCore::new().expect("GraphicsCore作成失敗");
        
        use crate::com::d2d::D2D1DeviceExt;
        
        // 複数のDeviceContextを作成できることを確認
        let _dc1 = graphics
            .d2d
            .create_device_context(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
            .expect("DeviceContext1作成失敗");
        
        let _dc2 = graphics
            .d2d
            .create_device_context(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
            .expect("DeviceContext2作成失敗");
        
        let _dc3 = graphics
            .d2d
            .create_device_context(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
            .expect("DeviceContext3作成失敗");
        
        println!("[TEST PASS] Multiple ID2D1DeviceContext created successfully");
    }

    #[test]
    fn test_create_multiple_visuals() {
        let graphics = GraphicsCore::new().expect("GraphicsCore作成失敗");
        
        use crate::com::dcomp::DCompositionDeviceExt;
        
        // 複数のVisualを作成できることを確認
        let _v1 = graphics.dcomp.create_visual().expect("Visual1作成失敗");
        let _v2 = graphics.dcomp.create_visual().expect("Visual2作成失敗");
        let _v3 = graphics.dcomp.create_visual().expect("Visual3作成失敗");
        
        println!("[TEST PASS] Multiple IDCompositionVisual3 created successfully");
    }

    #[test]
    fn test_commit() {
        let graphics = GraphicsCore::new().expect("GraphicsCore作成失敗");
        
        use crate::com::dcomp::DCompositionDeviceExt;
        
        // Commit()を呼び出せることを確認
        graphics.dcomp.commit().expect("Commit失敗");
        
        println!("[TEST PASS] IDCompositionDevice3::Commit() succeeded");
    }
}
