#[cfg(test)]
mod tests {
    use wintf::ecs::widget::text::label::{Label, TextDirection};
    use wintf::ecs::TextLayoutMetrics;

    #[test]
    fn test_text_direction_enum() {
        let direction = TextDirection::default();
        assert_eq!(direction, TextDirection::HorizontalLeftToRight);

        let vertical = TextDirection::VerticalRightToLeft;
        assert_eq!(vertical, TextDirection::VerticalRightToLeft);
    }

    #[test]
    fn test_label_has_direction() {
        let label = Label {
            text: "Test".to_string(),
            direction: TextDirection::VerticalRightToLeft,
            ..Default::default()
        };
        assert_eq!(label.direction, TextDirection::VerticalRightToLeft);
    }

    #[test]
    fn test_text_layout_metrics() {
        let metrics = TextLayoutMetrics {
            width: 100.0,
            height: 200.0,
        };
        assert_eq!(metrics.width, 100.0);
        assert_eq!(metrics.height, 200.0);
    }
}
