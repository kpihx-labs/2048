#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelLine {
    pub x: u16,
    pub y: u16,
    pub text: String,
}

const DIGITS: [[&str; 3]; 10] = [
    ["███", "█ █", "███"],
    [" ██", "  █", "  █"],
    ["███", " ██", "██ "],
    ["███", " ██", "███"],
    ["█ █", "███", "  █"],
    ["██ ", " ██", "███"],
    ["██ ", "███", "███"],
    ["███", "  █", "  █"],
    ["███", "███", "███"],
    ["███", "███", "  █"],
];

pub fn label_lines(value: u16, width: u16, height: u16) -> Vec<LabelLine> {
    let value = value.to_string();
    let big_width = value.chars().count() as u16 * 4 - 1;

    if height >= 3 && big_width <= width {
        let y = (height - 3) / 2;
        let x = (width - big_width) / 2;
        return (0..3)
            .map(|row| LabelLine {
                x,
                y: y + row as u16,
                text: value
                    .chars()
                    .map(|digit| DIGITS[digit.to_digit(10).expect("decimal digit") as usize][row])
                    .collect::<Vec<_>>()
                    .join(" "),
            })
            .collect();
    }

    let text_width = value.chars().count() as u16;
    vec![LabelLine {
        x: width.saturating_sub(text_width) / 2,
        y: height / 2,
        text: value,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_big_digits_when_the_tile_can_hold_them() {
        let lines = label_lines(2048, 20, 5);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].text.chars().count(), 15);
        assert_eq!(lines[0].y, 1);
    }

    #[test]
    fn falls_back_to_a_centered_plain_number() {
        let lines = label_lines(2048, 5, 1);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "2048");
        assert_eq!(lines[0].x, 0);
    }
}
