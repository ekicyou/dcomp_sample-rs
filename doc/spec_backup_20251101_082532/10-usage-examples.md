# 使用例

    horizontal_alignment: Alignment,
    vertical_alignment: Alignment,
    
    // レイアウトタイプ
    layout_type: LayoutType,
    
    // 計算結果（キャッシュ）
    desired_size: Size2D,
    final_rect: Rect,
}

pub enum Length {
    Auto,
    Pixels(f32),
    Percent(f32),
}

pub enum LayoutType {
    None,
    Stack(StackLayout),
    // 将来的に追加
    // Grid(GridLayout),
    // Flex(FlexLayout),
}

impl WidgetSystem {
    /// レイアウトを更新（2パス）
    pub fn update_layout(&mut self, root_id: WidgetId, available_size: Size2D) {
        // パス1: Measure（子から親へ、必要なサイズを計算）
        self.measure_widget_recursive(root_id, available_size);
        
        // パス2: Arrange（親から子へ、最終位置を決定）
        let final_rect = Rect::new(Point2D::zero(), available_size);
        self.arrange_widget_recursive(root_id, final_rect);
        
        // Visualに反映
        self.apply_layout_to_visuals();
    }
    
    fn measure_widget_recursive(&mut self, widget_id: WidgetId, available: Size2D) -> Size2D {
        let layout = self.layouts.get(widget_id);
        
        // レイアウトタイプに応じて子を計測
        let desired_size = match &layout.map(|l| &l.layout_type) {
            Some(LayoutType::Stack(_)) => self.measure_stack(widget_id),
            _ => Size2D::zero(),
        };
        
        // 制約を適用
        let constrained = self.apply_constraints(widget_id, desired_size);
        
        // 結果を保存
        if let Some(layout) = self.layouts.get_mut(widget_id) {
            layout.desired_size = constrained;
        }
        
        constrained
    }
    
    fn arrange_widget_recursive(&mut self, widget_id: WidgetId, final_rect: Rect) {
        // 自分の最終矩形を保存
        if let Some(layout) = self.layouts.get_mut(widget_id) {
            layout.final_rect = final_rect;
        }
        
        // 子を配置
        for child_id in self.children(widget_id) {
            let child_rect = self.calculate_child_rect(widget_id, child_id, final_rect);
