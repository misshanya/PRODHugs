//! Sudoku puzzle / solution generator used by the captcha service.

use rand::seq::SliceRandom;
use rand::Rng;

pub type Board = [[u8; 9]; 9];

const CELLS_TO_REMOVE: usize = 40;

/// Generate a fully solved board and a corresponding puzzle (with
/// `CELLS_TO_REMOVE` cells cleared to zero).
pub fn generate() -> (Board, Board) {
    let mut rng = rand::thread_rng();
    let mut solution: Board = [[0; 9]; 9];
    fill_board(&mut solution, &mut rng);

    let mut puzzle = solution;
    let mut left = CELLS_TO_REMOVE;
    while left > 0 {
        let row = rng.gen_range(0..9);
        let col = rng.gen_range(0..9);
        if puzzle[row][col] != 0 {
            puzzle[row][col] = 0;
            left -= 1;
        }
    }

    (puzzle, solution)
}

fn fill_board<R: Rng>(board: &mut Board, rng: &mut R) -> bool {
    for row in 0..9 {
        for col in 0..9 {
            if board[row][col] != 0 {
                continue;
            }
            let mut nums: [u8; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];
            nums.shuffle(rng);
            for n in nums {
                if is_valid(board, row, col, n) {
                    board[row][col] = n;
                    if fill_board(board, rng) {
                        return true;
                    }
                    board[row][col] = 0;
                }
            }
            return false;
        }
    }
    true
}

fn is_valid(board: &Board, row: usize, col: usize, num: u8) -> bool {
    for i in 0..9 {
        if board[row][i] == num {
            return false;
        }
        if board[i][col] == num {
            return false;
        }
    }
    let sr = (row / 3) * 3;
    let sc = (col / 3) * 3;
    for r in sr..sr + 3 {
        for c in sc..sc + 3 {
            if board[r][c] == num {
                return false;
            }
        }
    }
    true
}
