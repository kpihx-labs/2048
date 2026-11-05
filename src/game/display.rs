use ratatui_core::{buffer::Buffer, layout::Rect, widgets::Widget};
use tui_big_text::{BigText, PixelSize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelLine {
    pub x: u16,
    pub y: u16,
    pub text: String,
}

pub fn label_lines(value: u16, width: u16, height: u16) -> Vec<LabelLine> {
    let value = value.to_string();

    if let Some((pixel_size, rendered_height)) = best_pixel_size(&value, width, height) {
        return render_big_text(&value, width, height, pixel_size, rendered_height);
    }

    let text_width = value.chars().count() as u16;
    vec![LabelLine {
        x: width.saturating_sub(text_width) / 2,
        y: height / 2,
        text: value,
    }]
}

fn best_pixel_size(value: &str, width: u16, height: u16) -> Option<(PixelSize, u16)> {
    let glyphs = value.chars().count() as u16;
    if height >= 8 && width >= glyphs * 8 {
        Some((PixelSize::Full, 8))
    } else if height >= 4 && width >= glyphs * 4 {
        Some((PixelSize::Quadrant, 4))
    } else {
        None
    }
}

fn render_big_text(
    value: &str,
    width: u16,
    height: u16,
    pixel_size: PixelSize,
    rendered_height: u16,
) -> Vec<LabelLine> {
    let area = Rect::new(0, 0, width, rendered_height);
    let mut buffer = Buffer::empty(area);
    let widget = BigText::builder()
        .pixel_size(pixel_size)
        .centered()
        .lines(vec![value.to_string().into()])
        .build();
    widget.render(area, &mut buffer);

    let y_offset = (height - rendered_height) / 2;
    (0..rendered_height)
        .map(|y| LabelLine {
            x: 0,
            y: y_offset + y,
            text: (0..width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_quadrant_pixel_digits_in_split_pane_tiles() {
        let lines = label_lines(2048, 28, 4);
        assert_eq!(lines.len(), 4);
        assert!(lines.iter().all(|line| line.text.chars().count() == 28));
        assert!(lines.iter().any(|line| line.text.contains('█')));
        assert!(lines.iter().any(|line| {
            line.text
                .chars()
                .any(|character| matches!(character, '▘' | '▝' | '▖' | '▗'))
        }));
    }

    #[test]
    fn uses_full_pixel_digits_when_tiles_are_tall_enough() {
        let lines = label_lines(2, 8, 8);
        assert_eq!(lines.len(), 8);
        assert!(lines.iter().all(|line| line.text.chars().count() == 8));
        assert!(lines.iter().any(|line| line.text.contains('█')));
    }

    #[test]
    fn renders_every_multi_digit_game_value_without_clipping() {
        for (value, width) in [(128, 12), (1024, 16), (2048, 16)] {
            let lines = label_lines(value, width, 4);
            assert_eq!(lines.len(), 4);
            assert!(lines
                .iter()
                .all(|line| line.text.chars().count() == width as usize));
            assert!(lines.iter().any(|line| line.text.trim().len() > 1));
        }
    }

    #[test]
    fn falls_back_to_a_centered_plain_number() {
        let lines = label_lines(2048, 5, 1);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "2048");
        assert_eq!(lines[0].x, 0);
    }
}
