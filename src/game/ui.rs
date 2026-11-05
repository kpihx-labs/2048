use crate::game::display::{label_lines, LabelLine};
use crate::game::layout::{BoardLayout, LayoutError, LayoutOptions, BOARD_SIZE};
use crate::game::logic::GameLogic;
use crate::scores::{self, HighScores};
use crossterm::{
    cursor::MoveTo,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind,
    },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameOverAction {
    Restart,
    Quit,
}

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
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        enable_raw_mode()?;

        let result = self.run_loop(&mut stdout);
        let raw_result = disable_raw_mode();
        let screen_result = execute!(stdout, DisableMouseCapture, LeaveAlternateScreen);
        result.and(raw_result).and(screen_result)
    }

    fn run_loop(&mut self, stdout: &mut Stdout) -> std::io::Result<()> {
        self.show_welcome_screen(stdout)?;

        loop {
            self.draw_board(stdout)?;

            if self.logic.game_over {
                match self.show_final_score_screen(stdout)? {
                    GameOverAction::Restart => {
                        self.logic = GameLogic::new();
                        continue;
                    }
                    GameOverAction::Quit => break,
                }
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
        self.draw_grid(stdout, layout, &self.logic.board)?;
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

    fn draw_grid(
        &self,
        stdout: &mut Stdout,
        layout: BoardLayout,
        board: &[[u16; 4]; 4],
    ) -> std::io::Result<()> {
        for row in 0..=BOARD_SIZE {
            self.draw_border(stdout, layout, row)?;
            if row == BOARD_SIZE {
                break;
            }
            for inner_y in 0..layout.tile_height {
                let y = layout.board_y + row * (layout.tile_height + 1) + inner_y + 1;
                queue!(stdout, MoveTo(layout.board_x, y), Print("│"))?;
                for column in 0..BOARD_SIZE {
                    self.draw_tile_row(
                        stdout,
                        layout,
                        board,
                        row as usize,
                        column as usize,
                        inner_y,
                    )?;
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
        board: &[[u16; 4]; 4],
        row: usize,
        column: usize,
        inner_y: u16,
    ) -> std::io::Result<()> {
        let value = board[row][column];
        let (foreground, background) = tile_colors(value);
        let labels = if value == 0 {
            Vec::new()
        } else {
            label_lines(value, layout.tile_width, layout.tile_height)
        };
        let content = tile_row_content(layout.tile_width, inner_y, &labels);
        let x = layout.board_x + 1 + column as u16 * (layout.tile_width + 1);
        let y = layout.board_y + row as u16 * (layout.tile_height + 1) + inner_y + 1;

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

    fn show_final_score_screen(&mut self, stdout: &mut Stdout) -> std::io::Result<GameOverAction> {
        if scores::is_new_high_score(&self.high_scores, self.logic.score) {
            scores::add_high_score(&mut self.high_scores, self.logic.score, &self.logic.board);
        }

        loop {
            self.draw_board(stdout)?;
            let (width, height) = terminal::size()?;
            if let Ok(layout) = BoardLayout::calculate(width, height, self.layout_options) {
                self.draw_game_over_overlay(stdout, layout)?;
            }
            stdout.flush()?;

            match event::read()? {
                Event::Resize(_, _) => continue,
                Event::Key(key) => match key.code {
                    KeyCode::Char('r') | KeyCode::Enter => return Ok(GameOverAction::Restart),
                    KeyCode::Char('h') => self.show_high_scores(stdout)?,
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(GameOverAction::Quit),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn draw_game_over_overlay(
        &self,
        stdout: &mut Stdout,
        layout: BoardLayout,
    ) -> std::io::Result<()> {
        let width = layout.board_width.saturating_sub(2).min(48);
        if width < 20 || layout.board_height < 7 {
            return Ok(());
        }
        let height = 7;
        let x = layout.board_x + layout.board_width.saturating_sub(width) / 2;
        let y = layout.board_y + layout.board_height.saturating_sub(height) / 2;
        let inner = width.saturating_sub(2) as usize;
        let lines = [
            center_text("GAME OVER", inner),
            center_text(&format!("SCORE {}", self.logic.score), inner),
            center_text("R / Enter  restart", inner),
            center_text("H          history", inner),
            center_text("Q / Esc    quit", inner),
        ];

        queue!(
            stdout,
            SetBackgroundColor(Color::DarkRed),
            SetForegroundColor(Color::White),
            SetAttribute(Attribute::Bold),
            MoveTo(x, y),
            Print(format!("┌{}┐", "─".repeat(inner)))
        )?;
        for (row, line) in lines.iter().enumerate() {
            queue!(
                stdout,
                MoveTo(x, y + row as u16 + 1),
                Print(format!("│{}│", line))
            )?;
        }
        queue!(
            stdout,
            MoveTo(x, y + height - 1),
            Print(format!("└{}┘", "─".repeat(inner))),
            SetAttribute(Attribute::Reset),
            ResetColor
        )?;
        Ok(())
    }

    fn show_high_scores(&self, stdout: &mut Stdout) -> std::io::Result<()> {
        let mut selected = 0_usize;
        let mut offset = 0_usize;

        loop {
            let (width, height) = terminal::size()?;
            queue!(stdout, Clear(ClearType::All))?;
            let table_width = 60_u16.min(width);
            let x = width.saturating_sub(table_width) / 2;
            let title_y = 1_u16.min(height.saturating_sub(1));
            let list_y = title_y + 2;
            let visible_rows = height.saturating_sub(6).max(1) as usize;
            let (start, end) = score_window(
                self.high_scores.scores.len(),
                selected,
                visible_rows,
                &mut offset,
            );
            queue!(
                stdout,
                MoveTo(x, title_y),
                SetAttribute(Attribute::Bold),
                SetForegroundColor(Color::Yellow),
                Print("PERFORMANCE HISTORY"),
                SetAttribute(Attribute::Reset)
            )?;

            if self.high_scores.scores.is_empty() {
                queue!(
                    stdout,
                    MoveTo(x, list_y),
                    Print("No completed ranked run yet.")
                )?;
            } else {
                for (row, index) in (start..end).enumerate() {
                    let entry = &self.high_scores.scores[index];
                    let line = truncate(
                        &format!(
                            "{:>2}. {:>8} pts  tile {:>5}  {}",
                            index + 1,
                            entry.score,
                            entry.highest_tile,
                            entry.date
                        ),
                        table_width as usize,
                    );
                    queue!(
                        stdout,
                        MoveTo(x, list_y + row as u16),
                        SetBackgroundColor(if index == selected {
                            Color::DarkYellow
                        } else {
                            Color::Reset
                        }),
                        SetForegroundColor(if index == selected {
                            Color::Black
                        } else if index < 3 {
                            Color::Yellow
                        } else {
                            Color::White
                        }),
                        Print(format!("{line:<width$}", width = table_width as usize)),
                        ResetColor
                    )?;
                }
            }
            queue!(
                stdout,
                MoveTo(x, height.saturating_sub(2)),
                SetForegroundColor(Color::DarkGrey),
                Print(truncate(
                    "↑↓ / wheel select   Enter / click snapshot   Esc back",
                    table_width as usize
                )),
                ResetColor
            )?;
            stdout.flush()?;

            match event::read()? {
                Event::Resize(_, _) => continue,
                Event::Key(key) => match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        selected = selected.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        selected =
                            (selected + 1).min(self.high_scores.scores.len().saturating_sub(1));
                    }
                    KeyCode::PageUp => selected = selected.saturating_sub(visible_rows),
                    KeyCode::PageDown => {
                        selected = (selected + visible_rows)
                            .min(self.high_scores.scores.len().saturating_sub(1));
                    }
                    KeyCode::Enter if !self.high_scores.scores.is_empty() => {
                        self.show_score_snapshot(stdout, selected)?;
                    }
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('h') => return Ok(()),
                    _ => {}
                },
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollUp => selected = selected.saturating_sub(1),
                    MouseEventKind::ScrollDown => {
                        selected =
                            (selected + 1).min(self.high_scores.scores.len().saturating_sub(1));
                    }
                    MouseEventKind::Down(MouseButton::Left)
                        if mouse.column >= x
                            && mouse.column < x + table_width
                            && mouse.row >= list_y
                            && mouse.row < list_y + (end - start) as u16 =>
                    {
                        selected = start + (mouse.row - list_y) as usize;
                        self.show_score_snapshot(stdout, selected)?;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn show_score_snapshot(&self, stdout: &mut Stdout, start: usize) -> std::io::Result<()> {
        let mut selected = start;
        loop {
            self.draw_score_snapshot(stdout, selected)?;
            match event::read()? {
                Event::Resize(_, _) => continue,
                Event::Key(key) => match key.code {
                    KeyCode::Left | KeyCode::Up | KeyCode::Char('k') => {
                        selected = selected.saturating_sub(1);
                    }
                    KeyCode::Right | KeyCode::Down | KeyCode::Char('j') => {
                        selected =
                            (selected + 1).min(self.high_scores.scores.len().saturating_sub(1));
                    }
                    KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => return Ok(()),
                    _ => {}
                },
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollUp => selected = selected.saturating_sub(1),
                    MouseEventKind::ScrollDown => {
                        selected =
                            (selected + 1).min(self.high_scores.scores.len().saturating_sub(1));
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn draw_score_snapshot(&self, stdout: &mut Stdout, selected: usize) -> std::io::Result<()> {
        let (width, height) = terminal::size()?;
        queue!(stdout, Clear(ClearType::All))?;
        let Some(entry) = self.high_scores.scores.get(selected) else {
            return stdout.flush();
        };
        let layout = match BoardLayout::calculate(width, height, self.layout_options) {
            Ok(layout) => layout,
            Err(error) => {
                self.draw_too_small(stdout, error)?;
                return stdout.flush();
            }
        };

        let title = truncate(
            &format!(
                "RUN {}/{}  |  SCORE {}  |  TILE {}  |  {}",
                selected + 1,
                self.high_scores.scores.len(),
                entry.score,
                entry.highest_tile,
                entry.date
            ),
            layout.board_width as usize,
        );
        let title_x = layout.board_x + layout.board_width.saturating_sub(title.len() as u16) / 2;
        let controls = truncate(
            "←↑ previous   →↓ next   wheel browse   Enter/Esc back",
            layout.board_width as usize,
        );
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
        self.draw_grid(stdout, layout, &entry.board)?;

        let footer = if entry.board.iter().flatten().all(|value| *value == 0) {
            "Legacy score: no final-board snapshot was stored."
        } else {
            "Responsive reconstruction of the final frozen board."
        };
        let footer = truncate(footer, layout.board_width as usize);
        let footer_x = layout.board_x + layout.board_width.saturating_sub(footer.len() as u16) / 2;
        queue!(
            stdout,
            MoveTo(footer_x, layout.footer_y + 1),
            SetForegroundColor(Color::DarkGrey),
            Print(footer),
            ResetColor
        )?;
        stdout.flush()
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

fn center_text(value: &str, width: usize) -> String {
    let value = truncate(value, width);
    let value_width = value.chars().count();
    let left = width.saturating_sub(value_width) / 2;
    let right = width.saturating_sub(value_width + left);
    format!("{}{}{}", " ".repeat(left), value, " ".repeat(right))
}

fn score_window(
    total: usize,
    selected: usize,
    visible: usize,
    offset: &mut usize,
) -> (usize, usize) {
    if total == 0 {
        *offset = 0;
        return (0, 0);
    }
    let visible = visible.max(1);
    if selected < *offset {
        *offset = selected;
    } else if selected >= *offset + visible {
        *offset = selected + 1 - visible;
    }
    *offset = (*offset).min(total.saturating_sub(visible));
    (*offset, (*offset + visible).min(total))
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

    #[test]
    fn centers_overlay_text_without_changing_width() {
        let line = center_text("GAME OVER", 20);
        assert_eq!(line.chars().count(), 20);
        assert_eq!(line.trim(), "GAME OVER");
    }

    #[test]
    fn score_window_follows_selection_and_remains_bounded() {
        let mut offset = 0;
        assert_eq!(score_window(10, 0, 3, &mut offset), (0, 3));
        assert_eq!(score_window(10, 5, 3, &mut offset), (3, 6));
        assert_eq!(score_window(10, 9, 3, &mut offset), (7, 10));
        assert_eq!(score_window(10, 1, 3, &mut offset), (1, 4));
    }
}
