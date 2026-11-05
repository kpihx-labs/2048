use std::fmt;
use std::str::FromStr;

pub const BOARD_SIZE: u16 = 4;
pub const MIN_TILE_WIDTH: u16 = 5;
pub const MIN_TILE_HEIGHT: u16 = 1;
pub const HEADER_HEIGHT: u16 = 4;
pub const FOOTER_HEIGHT: u16 = 2;
const BORDER_COUNT: u16 = BOARD_SIZE + 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Auto,
    Compact,
    Balanced,
    Large,
    Manual,
}

impl LayoutMode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Compact => "compact",
            Self::Balanced => "balanced",
            Self::Large => "large",
            Self::Manual => "manual",
        }
    }

    fn fixed_dimensions(self) -> Option<(u16, u16)> {
        match self {
            Self::Compact => Some((5, 1)),
            Self::Balanced => Some((11, 3)),
            Self::Large => Some((17, 5)),
            Self::Auto | Self::Manual => None,
        }
    }
}

impl FromStr for LayoutMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "auto" => Ok(Self::Auto),
            "compact" => Ok(Self::Compact),
            "balanced" => Ok(Self::Balanced),
            "large" => Ok(Self::Large),
            "manual" => Ok(Self::Manual),
            _ => Err(format!(
                "invalid layout '{value}'; expected auto, compact, balanced, large, or manual"
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutOptions {
    pub mode: LayoutMode,
    pub tile_width: Option<u16>,
    pub tile_height: Option<u16>,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            mode: LayoutMode::Auto,
            tile_width: None,
            tile_height: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardLayout {
    pub mode: LayoutMode,
    pub tile_width: u16,
    pub tile_height: u16,
    pub board_x: u16,
    pub board_y: u16,
    pub board_width: u16,
    pub board_height: u16,
    pub content_y: u16,
    pub footer_y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutError {
    pub required_width: u16,
    pub required_height: u16,
    pub actual_width: u16,
    pub actual_height: u16,
}

impl fmt::Display for LayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "terminal too small: need {}x{}, have {}x{}",
            self.required_width, self.required_height, self.actual_width, self.actual_height
        )
    }
}

impl BoardLayout {
    pub fn calculate(
        terminal_width: u16,
        terminal_height: u16,
        options: LayoutOptions,
    ) -> Result<Self, LayoutError> {
        let (tile_width, tile_height) = match options.mode {
            LayoutMode::Auto => (
                terminal_width.saturating_sub(BORDER_COUNT) / BOARD_SIZE,
                terminal_height.saturating_sub(HEADER_HEIGHT + FOOTER_HEIGHT + BORDER_COUNT)
                    / BOARD_SIZE,
            ),
            LayoutMode::Manual => (
                options.tile_width.unwrap_or(MIN_TILE_WIDTH),
                options.tile_height.unwrap_or(MIN_TILE_HEIGHT),
            ),
            fixed => fixed.fixed_dimensions().expect("fixed layout dimensions"),
        };

        let tile_width = tile_width.max(MIN_TILE_WIDTH);
        let tile_height = tile_height.max(MIN_TILE_HEIGHT);
        let board_width = tile_width * BOARD_SIZE + BORDER_COUNT;
        let board_height = tile_height * BOARD_SIZE + BORDER_COUNT;
        let required_height = HEADER_HEIGHT + board_height + FOOTER_HEIGHT;

        if terminal_width < board_width || terminal_height < required_height {
            return Err(LayoutError {
                required_width: board_width,
                required_height,
                actual_width: terminal_width,
                actual_height: terminal_height,
            });
        }

        let content_y = (terminal_height - required_height) / 2;
        let board_x = (terminal_width - board_width) / 2;
        let board_y = content_y + HEADER_HEIGHT;

        Ok(Self {
            mode: options.mode,
            tile_width,
            tile_height,
            board_x,
            board_y,
            board_width,
            board_height,
            content_y,
            footer_y: board_y + board_height,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_layout_fills_a_split_pane() {
        let layout = BoardLayout::calculate(118, 30, LayoutOptions::default()).unwrap();
        assert_eq!((layout.tile_width, layout.tile_height), (28, 4));
        assert_eq!((layout.board_width, layout.board_height), (117, 21));
        assert_eq!(layout.board_x, 0);
    }

    #[test]
    fn auto_layout_grows_with_the_terminal() {
        let small = BoardLayout::calculate(80, 24, LayoutOptions::default()).unwrap();
        let large = BoardLayout::calculate(160, 50, LayoutOptions::default()).unwrap();
        assert!(large.tile_width > small.tile_width);
        assert!(large.tile_height > small.tile_height);
    }

    #[test]
    fn manual_layout_is_centered() {
        let options = LayoutOptions {
            mode: LayoutMode::Manual,
            tile_width: Some(18),
            tile_height: Some(5),
        };
        let layout = BoardLayout::calculate(100, 40, options).unwrap();
        assert_eq!((layout.tile_width, layout.tile_height), (18, 5));
        assert_eq!(layout.board_x, (100 - 77) / 2);
    }

    #[test]
    fn reports_required_size_for_oversized_manual_layout() {
        let options = LayoutOptions {
            mode: LayoutMode::Manual,
            tile_width: Some(30),
            tile_height: Some(8),
        };
        let error = BoardLayout::calculate(80, 24, options).unwrap_err();
        assert_eq!((error.required_width, error.required_height), (125, 43));
        assert_eq!((error.actual_width, error.actual_height), (80, 24));
    }
}
