use terminal_2048::game::logic::GameLogic;

#[test]
fn test_move_left() {
    let mut game = GameLogic::new();

    // Test case 1: Simple merge
    game.board = [[2, 2, 0, 0], [4, 0, 4, 0], [8, 8, 8, 8], [2, 4, 8, 16]];
    game.score = 0;
    let moved = game.move_left();
    assert!(moved);
    assert_eq!(game.board[0], [4, 0, 0, 0]);
    assert_eq!(game.board[1], [8, 0, 0, 0]);
    assert_eq!(game.board[2], [16, 16, 0, 0]);
    assert_eq!(game.board[3], [2, 4, 8, 16]);
    assert_eq!(game.score, 4 + 8 + 16 + 16);

    // Test case 2: No move possible
    game.board = [[2, 4, 8, 16], [16, 8, 4, 2], [2, 4, 8, 16], [16, 8, 4, 2]];
    let moved = game.move_left();
    assert!(!moved);
}

#[test]
fn test_move_right() {
    let mut game = GameLogic::new();
    game.board = [[2, 2, 0, 0], [4, 0, 4, 0], [8, 8, 8, 8], [16, 8, 4, 2]];
    game.score = 0;
    let moved = game.move_right();
    assert!(moved);
    assert_eq!(game.board[0], [0, 0, 0, 4]);
    assert_eq!(game.board[1], [0, 0, 0, 8]);
    assert_eq!(game.board[2], [0, 0, 16, 16]);
    assert_eq!(game.board[3], [16, 8, 4, 2]);
    assert_eq!(game.score, 4 + 8 + 16 + 16);
}

#[test]
fn test_move_up() {
    let mut game = GameLogic::new();
    game.board = [[2, 4, 8, 16], [2, 4, 8, 16], [0, 0, 0, 0], [0, 0, 0, 0]];
    game.score = 0;
    let moved = game.move_up();
    assert!(moved);
    let expected_board = [[4, 8, 16, 32], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]];
    assert_eq!(game.board, expected_board);
    assert_eq!(game.score, 4 + 8 + 16 + 32);
}

#[test]
fn test_move_down() {
    let mut game = GameLogic::new();
    game.board = [[2, 4, 8, 16], [2, 4, 8, 16], [0, 0, 0, 0], [0, 0, 0, 0]];
    game.score = 0;
    let moved = game.move_down();
    assert!(moved);
    let expected_board = [[0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [4, 8, 16, 32]];
    assert_eq!(game.board, expected_board);
    assert_eq!(game.score, 4 + 8 + 16 + 32);
}

#[test]
fn test_can_move() {
    let mut game = GameLogic::new();
    // Test case 1: Board with empty cells
    game.board = [[2, 4, 8, 16], [16, 8, 4, 2], [2, 4, 8, 16], [16, 8, 4, 0]];
    assert!(game.can_move());

    // Test case 2: Full board with possible horizontal move
    game.board = [[2, 2, 8, 16], [16, 8, 4, 2], [2, 4, 8, 16], [16, 8, 4, 2]];
    assert!(game.can_move());

    // Test case 3: Full board with possible vertical move
    game.board = [[2, 4, 8, 16], [2, 8, 4, 2], [4, 4, 8, 16], [16, 8, 4, 2]];
    assert!(game.can_move());

    // Test case 4: Full board with no possible moves
    game.board = [[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]];
    assert!(!game.can_move());
}

#[test]
fn test_win_scenario() {
    let mut game = GameLogic::new();
    game.board = [[1024, 1024, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]];
    game.score = 0;
    game.won = false;

    let moved = game.move_left();

    assert!(moved);
    assert!(game.won, "Game should be marked as won");
    assert_eq!(game.board[0], [2048, 0, 0, 0]);
    assert_eq!(game.score, 2048);
}

#[test]
fn test_game_over_scenario() {
    let mut game = GameLogic::new();
    game.board = [[2, 4, 8, 16], [16, 8, 4, 2], [2, 4, 8, 16], [16, 8, 4, 2]];

    assert!(!game.can_move(), "Game should be over (no moves possible)");

    let board_before = game.board;
    game.move_left();
    assert_eq!(
        game.board, board_before,
        "Board should not change after a move when game is over"
    );
    game.move_right();
    assert_eq!(
        game.board, board_before,
        "Board should not change after a move when game is over"
    );
    game.move_up();
    assert_eq!(
        game.board, board_before,
        "Board should not change after a move when game is over"
    );
    game.move_down();
    assert_eq!(
        game.board, board_before,
        "Board should not change after a move when game is over"
    );
}
