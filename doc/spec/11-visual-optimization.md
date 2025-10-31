# ビジュアルツリーの最適化

            self.arrange_widget_recursive(child_id, child_rect);
        }
    }
    
    fn apply_layout_to_visuals(&mut self) {
        for (widget_id, layout) in &self.layouts {
            if let Some(visual) = self.visuals.get_mut(widget_id) {
                visual.offset = layout.final_rect.origin;
                visual.size = layout.final_rect.size;
                
                // DirectCompositionに反映
                visual.dcomp_visual.SetOffsetX(layout.final_rect.origin.x).unwrap();
                visual.dcomp_visual.SetOffsetY(layout.final_rect.origin.y).unwrap();
            }
        }
    }
}
```

## ヒットテストシステム

### ヒットテストの実装

**Visualの有無に関わらず、Widgetツリーでヒットテストを行う**

```rust
impl WidgetSystem {
    /// 座標からWidgetを検索
    pub fn hit_test(&self, point: Point2D) -> Option<WidgetId> {
        // ルートから深さ優先探索（Z順序を考慮）
        self.hit_test_recursive(self.root_id, point)
    }
    
    fn hit_test_recursive(&self, widget_id: WidgetId, point: Point2D) -> Option<WidgetId> {
        // レイアウト情報から矩形を取得
        let layout = self.layouts.get(widget_id)?;
        
        // この矩形内か？
        if !layout.final_rect.contains(point) {
            return None;
        }
        
        // 子を逆順で検索（後に追加した子が上に表示される）
        let children: Vec<_> = self.children(widget_id).collect();
        for child_id in children.iter().rev() {
            // 子の座標系に変換
            let local_point = self.to_local_coordinates(*child_id, point);
            if let Some(hit) = self.hit_test_recursive(*child_id, local_point) {
                return Some(hit);
            }
        }
        
        // 子でヒットしなければ、インタラクティブなWidgetならヒット
        if self.is_interactive(widget_id) {
            Some(widget_id)
        } else {
            None // 透過（親に伝播）
        }
    }
