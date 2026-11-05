use crate::game::display::{label_lines, LabelLine};
use crate::game::layout::{BoardLayout, LayoutError, LayoutOptions, BOARD_SIZE};
use crate::game::logic::GameLogic;
use crate::scores::{self, HighScores};
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode},
    execute, queue,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    terminal::{
        self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::io::{stdout, Stdout, Write};

pub struct GameUI {
    logic: GameLogic,
    high_scores: HighScores,
    layout_options: LayoutOptions,
}

impl GameUI {
    pub fn new() -> Self {
        Self::with_layout(LayoutOptions::default())
    }

    pub fn with_layout(layout_options: LayoutOptions) -> Self {
        Self {
            logic: GameLogic::new(),
            high_scores: scores::load_high_scores(),
            layout_options,
        }
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        enable_raw_mode()?;

        let result = self.run_loop(&mut stdout);
        let raw_result = disable_raw_mode();
        let screen_result = execute!(stdout, LeaveAlternateScreen);
        result.and(raw_result).and(screen_result)
    }

    fn run_loop(&mut self, stdout: &mut Stdout) -> std::io::Result<()> {
        self.show_welcome_screen(stdout)?;

        loop {
            self.draw_board(stdout)?;

            if self.logic.game_over {
                self.show_final_score_screen(stdout)?;
                self.logic = GameLogic::new();
                self.show_welcome_screen(stdout)?;
                continue;
            }

            match event::read()? {
                Event::Key(key_event) => {
                    let moved = match key_event.code {
                        KeyCode::Char('w') | KeyCode::Up => self.logic.move_up(),
                        KeyCode::Char('s') | KeyCode::Down => self.logic.move_down(),
                        KeyCode::Char('a') | KeyCode::Left => self.logic.move_left(),
                        KeyCode::Char('d') | KeyCode::Right => self.logic.move_right(),
                        KeyCode::Char('h') => {
                            self.show_high_scores(stdout)?;
                            false
                        }
                        KeyCode::Char('r') => {
                            self.logic = GameLogic::new();
                            false
                        }
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        _ => false,
                    };

                    if moved {
                        self.logic.add_random_tile();
                    }
                    if !self.logic.can_move() {
                        self.logic.game_over = true;
                    }
                }
                Event::Resize(_, _) => continue,
                _ => {}
            }
        }

        Ok(())
    }

    fn show_welcome_screen(&self, stdout: &mut Stdout) -> std::io::Result<()> {
        loop {
            let (width, height) = terminal::size()?;
            queue!(stdout, Clear(ClearType::All))?;

            let lines = [
                "2048",
                "Combine equal tiles. Build the 2048 tile.",
                "WASD / arrows: move   H: scores   R: restart   Q: quit",
                "Press any key to start",
            ];
            let start_y = height.saturating_sub(lines.len() as u16) / 2;
            for (index, line) in lines.iter().enumerate() {
                let x = width.saturating_sub(line.chars().count() as u16) / 2;
                let color = if index == 0 {
                    Color::Yellow
                } else if index == lines.len() - 1 {
                    Color::DarkGrey
                } else {
                    Color::Cyan
                };
                queue!(
                    stdout,
                    MoveTo(x, start_y + index as u16),
                    SetForegroundColor(color),
                    SetAttribute(if index == 0 {
                        Attribute::Bold
                    } else {
                        Attribute::Reset
                    }),
                    Print(line),
                    ResetColor
                )?;
            }
            stdout.flush()?;

            match event::read()? {
                Event::Resize(_, _) => continue,
                Event::Key(_) => return Ok(()),
                _ => {}
            }
        }
    }

    fn draw_board(&self, stdout: &mut Stdout) -> std::io::Result<()> {
        let (width, height) = terminal::size()?;
        queue!(stdout, Clear(ClearType::All))?;

        let layout = match BoardLayout::calculate(width, height, self.layout_options) {
            Ok(layout) => layout,
            Err(error) => {
                self.draw_too_small(stdout, error)?;
                stdout.flush()?;
                return Ok(());
            }
        };

        self.draw_header(stdout, layout)?;
        self.draw_grid(stdout, layout)?;
        self.draw_footer(stdout, layout)?;
        stdout.flush()
    }

    fn draw_header(&self, stdout: &mut Stdout, layout: BoardLayout) -> std::io::Result<()> {
        let best = self
            .high_scores
            .scores
            .first()
            .map_or(0, |entry| entry.score);
        let title = format!(
            "2048  |  SCORE {}  |  BEST {}  |  {} {}x{}",
            self.logic.score,
            best,
            layout.mode.name(),
            layout.tile_width,
            layout.tile_height
        );
        let title = truncate(&title, layout.board_width as usize);
        let title_x = layout.board_x + layout.board_width.saturating_sub(title.len() as u16) / 2;
        let controls = "WASD/arrows move  H scores  R restart  Q quit";
        let controls = truncate(controls, layout.board_width as usize);
        let controls_x =
            layout.board_x + layout.board_width.saturating_sub(controls.len() as u16) / 2;

        queue!(
            stdout,
            MoveTo(title_x, layout.content_y),
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Yellow),
            Print(title),
            SetAttribute(Attribute::Reset),
            MoveTo(controls_x, layout.content_y + 1),
            SetForegroundColor(Color::Cyan),
            Print(controls),
            MoveTo(layout.board_x, layout.content_y + 2),
            Print("─".repeat(layout.board_width as usize)),
            ResetColor
        )?;
        Ok(())
    }

    fn draw_grid(&self, stdout: &mut Stdout, layout: BoardLayout) -> std::io::Result<()> {
        for row in 0..=BOARD_SIZE {
            self.draw_border(stdout, layout, row)?;
            if row == BOARD_SIZE {
                break;
            }
            for inner_y in 0..layout.tile_height {
                let y = layout.board_y + row * (layout.tile_height + 1) + inner_y + 1;
                queue!(stdout, MoveTo(layout.board_x, y), Print("│"))?;
                for column in 0..BOARD_SIZE {
                    self.draw_tile_row(stdout, layout, row as usize, column as usize, inner_y, y)?;
                    queue!(stdout, Print("│"))?;
                }
            }
        }
        Ok(())
    }

    fn draw_border(
        &self,
        stdout: &mut Stdout,
        layout: BoardLayout,
        row: u16,
    ) -> std::io::Result<()> {
        let (left, middle, right) = if row == 0 {
            ('┌', '┬', '┐')
        } else if row == BOARD_SIZE {
            ('└', '┴', '┘')
        } else {
            ('├', '┼', '┤')
        };
        let y = layout.board_y + row * (layout.tile_height + 1);
        let mut line = String::new();
        line.push(left);
        for column in 0..BOARD_SIZE {
            line.push_str(&"─".repeat(layout.tile_width as usize));
            line.push(if column + 1 == BOARD_SIZE {
                right
            } else {
                middle
            });
        }
        queue!(
            stdout,
            MoveTo(layout.board_x, y),
            SetForegroundColor(Color::DarkGrey),
            Print(line),
            ResetColor
        )?;
        Ok(())
    }

    fn draw_tile_row(
        &self,
        stdout: &mut Stdout,
        layout: BoardLayout,
        row: usize,
        column: usize,
        inner_y: u16,
        y: u16,
    ) -> std::io::Result<()> {
        let value = self.logic.board[row][column];
        let (foreground, background) = tile_colors(value);
        let labels = if value == 0 {
            Vec::new()
        } else {
            label_lines(value, layout.tile_width, layout.tile_height)
        };
        let content = tile_row_content(layout.tile_width, inner_y, &labels);
        let x = layout.board_x + 1 + column as u16 * (layout.tile_width + 1);

        queue!(
            stdout,
            MoveTo(x, y),
            SetBackgroundColor(background),
            SetForegroundColor(foreground),
            SetAttribute(Attribute::Bold),
            Print(content),
            SetAttribute(Attribute::Reset),
            ResetColor
        )?;
        Ok(())
    }

    fn draw_footer(&self, stdout: &mut Stdout, layout: BoardLayout) -> std::io::Result<()> {
        let message = if self.logic.won && !self.logic.game_over {
            "You reached 2048. Keep going."
        } else if self.logic.game_over {
            "No moves remain."
        } else {
            "The board follows this pane and redraws when it is resized."
        };
        let message = truncate(message, layout.board_width as usize);
        let x = layout.board_x + layout.board_width.saturating_sub(message.len() as u16) / 2;
        queue!(
            stdout,
            MoveTo(x, layout.footer_y + 1),
            SetForegroundColor(if self.logic.won {
                Color::Green
            } else {
                Color::DarkGrey
            }),
            Print(message),
            ResetColor
        )?;
        Ok(())
    }

    fn draw_too_small(&self, stdout: &mut Stdout, error: LayoutError) -> std::io::Result<()> {
        let lines = [
            "Terminal too small for this layout".to_string(),
            format!(
                "Need {}x{}, have {}x{}",
                error.required_width,
                error.required_height,
                error.actual_width,
                error.actual_height
            ),
            "Resize the pane or choose --layout compact".to_string(),
        ];
        let start_y = error.actual_height.saturating_sub(lines.len() as u16) / 2;
        for (index, line) in lines.iter().enumerate() {
            let x = error.actual_width.saturating_sub(line.len() as u16) / 2;
            queue!(
                stdout,
                MoveTo(x, start_y + index as u16),
                SetForegroundColor(if index == 0 { Color::Red } else { Color::White }),
                Print(line),
                ResetColor
            )?;
        }
        Ok(())
    }

    fn show_final_score_screen(&mut self, stdout: &mut Stdout) -> std::io::Result<()> {
        if scores::is_new_high_score(&self.high_scores, self.logic.score) {
            scores::add_high_score(&mut self.high_scores, self.logic.score, &self.logic.board);
        }
        self.show_high_scores(stdout)
    }

    fn show_high_scores(&self, stdout: &mut Stdout) -> std::io::Result<()> {
        loop {
            let (width, height) = terminal::size()?;
            queue!(stdout, Clear(ClearType::All))?;
            let table_width = 48_u16.min(width);
            let x = width.saturating_sub(table_width) / 2;
            let mut y = height.saturating_sub(self.high_scores.scores.len() as u16 + 5) / 2;
            queue!(
                stdout,
                MoveTo(x, y),
                SetAttribute(Attribute::Bold),
                SetForegroundColor(Color::Yellow),
                Print("HIGH SCORES"),
                SetAttribute(Attribute::Reset)
            )?;
            y += 2;

            if self.high_scores.scores.is_empty() {
                queue!(stdout, MoveTo(x, y), Print("No scores yet."))?;
                y += 1;
            } else {
                for (index, entry) in self.high_scores.scores.iter().enumerate() {
                    queue!(
                        stdout,
                        MoveTo(x, y),
                        SetForegroundColor(if index < 3 {
                            Color::Yellow
                        } else {
                            Color::White
                        }),
                        Print(format!(
                            "{:>2}. {:>8} pts  tile {:>5}  {}",
                            index + 1,
                            entry.score,
                            entry.highest_tile,
                            entry.date
                        )),
                        ResetColor
                    )?;
                    y += 1;
                }
            }
            queue!(
                stdout,
                MoveTo(x, y + 1),
                SetForegroundColor(Color::DarkGrey),
                Print("Press any key to continue"),
                ResetColor
            )?;
            stdout.flush()?;

            match event::read()? {
                Event::Resize(_, _) => continue,
                Event::Key(_) => return Ok(()),
                _ => {}
            }
        }
    }
}

impl Default for GameUI {
    fn default() -> Self {
        Self::new()
    }
}

fn tile_row_content(width: u16, y: u16, labels: &[LabelLine]) -> String {
    if let Some(label) = labels.iter().find(|line| line.y == y) {
        let right = width.saturating_sub(label.x + label.text.chars().count() as u16);
        format!(
            "{}{}{}",
            " ".repeat(label.x as usize),
            label.text,
            " ".repeat(right as usize)
        )
    } else {
        " ".repeat(width as usize)
    }
}

fn tile_colors(value: u16) -> (Color, Color) {
    match value {
        0 => (
            Color::DarkGrey,
            Color::Rgb {
                r: 45,
                g: 45,
                b: 52,
            },
        ),
        2 => (Color::Black, Color::White),
        4 => (
            Color::Black,
            Color::Rgb {
                r: 237,
                g: 224,
                b: 200,
            },
        ),
        8 => (
            Color::White,
            Color::Rgb {
                r: 242,
                g: 177,
                b: 121,
            },
        ),
        16 => (
            Color::White,
            Color::Rgb {
                r: 245,
                g: 149,
                b: 99,
            },
        ),
        32 => (
            Color::White,
            Color::Rgb {
                r: 246,
                g: 124,
                b: 95,
            },
        ),
        64 => (
            Color::White,
            Color::Rgb {
                r: 246,
                g: 94,
                b: 59,
            },
        ),
        128 => (
            Color::White,
            Color::Rgb {
                r: 237,
                g: 207,
                b: 114,
            },
        ),
        256 => (
            Color::White,
            Color::Rgb {
                r: 237,
                g: 204,
                b: 97,
            },
        ),
        512 => (
            Color::White,
            Color::Rgb {
                r: 237,
                g: 200,
                b: 80,
            },
        ),
        1024 => (
            Color::White,
            Color::Rgb {
                r: 237,
                g: 197,
                b: 63,
            },
        ),
        2048 => (
            Color::White,
            Color::Rgb {
                r: 237,
                g: 194,
                b: 46,
            },
        ),
        _ => (
            Color::White,
            Color::Rgb {
                r: 112,
                g: 91,
                b: 151,
            },
        ),
    }
}

fn truncate(value: &str, max_width: usize) -> String {
    value.chars().take(max_width).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_a_label_without_changing_tile_width() {
        let labels = vec![LabelLine {
            x: 2,
            y: 1,
            text: "2048".to_string(),
        }];
        let row = tile_row_content(10, 1, &labels);
        assert_eq!(row.chars().count(), 10);
        assert_eq!(row, "  2048    ");
    }
}
