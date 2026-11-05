use rand::Rng;

pub struct GameLogic {
    pub board: [[u16; 4]; 4],
    pub score: u32,
    pub game_over: bool,
    pub won: bool,
}

impl GameLogic {
    pub fn new() -> Self {
        let board = [[0; 4]; 4];
        let mut logic = Self {
            board,
            score: 0,
            game_over: false,
            won: false,
        };
        logic.add_random_tile();
        logic.add_random_tile();
        logic
    }

    pub fn add_random_tile(&mut self) {
        let mut empty_cells = Vec::new();
        for r in 0..4 {
            for c in 0..4 {
                if self.board[r][c] == 0 {
                    empty_cells.push((r, c));
                }
            }
        }

        if let Some(&(r, c)) = empty_cells.get(rand::thread_rng().gen_range(0..empty_cells.len())) {
            self.board[r][c] = if rand::thread_rng().gen_bool(0.9) {
                2
            } else {
                4
            };
        }
    }

    pub fn move_left(&mut self) -> bool {
        let mut moved = false;
        for i in 0..4 {
            let row = self.board[i];
            let row_without_zeros: Vec<u16> = row.iter().filter(|&&c| c != 0).cloned().collect();
            let mut merged_row: Vec<u16> = Vec::new();
            let mut skip = false;

            for j in 0..row_without_zeros.len() {
                if skip {
                    skip = false;
                    continue;
                }
                if j < row_without_zeros.len() - 1
                    && row_without_zeros[j] == row_without_zeros[j + 1]
                {
                    let new_tile = row_without_zeros[j] * 2;
                    merged_row.push(new_tile);
                    self.score += new_tile as u32;
                    if new_tile == 2048 {
                        self.won = true;
                    }
                    skip = true;
                } else {
                    merged_row.push(row_without_zeros[j]);
                }
            }

            let mut new_row = [0; 4];
            for (idx, &val) in merged_row.iter().enumerate() {
                new_row[idx] = val;
            }

            if self.board[i] != new_row {
                moved = true;
                self.board[i] = new_row;
            }
        }
        moved
    }

    pub fn move_right(&mut self) -> bool {
        let original_board = self.board;
        for r in 0..4 {
            self.board[r].reverse();
        }
        self.move_left();
        for r in 0..4 {
            self.board[r].reverse();
        }
        self.board != original_board
    }

    fn transpose(&mut self) {
        for r in 0..4 {
            for c in r..4 {
                let temp = self.board[r][c];
                self.board[r][c] = self.board[c][r];
                self.board[c][r] = temp;
            }
        }
    }

    pub fn move_up(&mut self) -> bool {
        let original_board = self.board;
        self.transpose();
        self.move_left();
        self.transpose();
        self.board != original_board
    }

    pub fn move_down(&mut self) -> bool {
        let original_board = self.board;
        self.transpose();
        self.move_right();
        self.transpose();
        self.board != original_board
    }

    pub fn can_move(&self) -> bool {
        for r in 0..4 {
            for c in 0..4 {
                if self.board[r][c] == 0 {
                    return true;
                }
                if c < 3 && self.board[r][c] == self.board[r][c + 1] {
                    return true;
                }
                if r < 3 && self.board[r][c] == self.board[r + 1][c] {
                    return true;
                }
            }
        }
        false
    }
}

impl Default for GameLogic {
    fn default() -> Self {
        Self::new()
    }
}
