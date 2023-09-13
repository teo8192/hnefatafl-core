use std::fmt::Display;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Piece {
    King,
    Defender,
    Attacker,
}

pub struct Board {
    board: [[Option<Piece>; 11]; 11],
}

impl Board {
    pub fn new() -> Board {
        let mut board = Board {
            board: [[None; 11]; 11],
        };

        // placing defenders
        for i in 3..=7 {
            let a = 2 - i32::abs(i - 5);

            for j in 5 - a..5 + a + 1 {
                board.place_piece(Piece::Defender, i as usize, j as usize);
            }
        }
        board.place_piece(Piece::King, 5, 5);

        // placing the attackers
        for i in 3..=7 {
            board.place_piece(Piece::Attacker, i, 0);
            board.place_piece(Piece::Attacker, i, 10);
            board.place_piece(Piece::Attacker, 0, i);
            board.place_piece(Piece::Attacker, 10, i);
        }
        board.place_piece(Piece::Attacker, 5, 1);
        board.place_piece(Piece::Attacker, 5, 9);
        board.place_piece(Piece::Attacker, 1, 5);
        board.place_piece(Piece::Attacker, 9, 5);

        board
    }

    fn place_piece(&mut self, piece: Piece, x: usize, y: usize) {
        self.board[x][y] = Some(piece);
    }

    fn remove_piece(&mut self, x: usize, y: usize) {
        self.board[x][y] = None;
    }

    pub fn move_piece(&mut self, x: usize, y: usize, new_x: usize, new_y: usize) {
        self.board[new_x][new_y] = self.board[x][y];
        self.board[x][y] = None;
    }

    pub fn get_piece(&self, x: usize, y: usize) -> Option<Piece> {
        self.board[x][y]
    }
}

// {{{ Display

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.board.iter() {
            for piece in row.iter() {
                match piece {
                    Some(Piece::King) => f.write_str("K")?,
                    Some(Piece::Defender) => f.write_str("D")?,
                    Some(Piece::Attacker) => f.write_str("A")?,
                    None => f.write_str(" ")?,
                }
            }
            f.write_str("\n")?;
        }

        Ok(())
    }
}

// }}}

// {{{ Default

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

// }}}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board() {
        let board = Board::new();

        use Piece::*;

        // {{{ A lot of asserts
        assert_eq!(board.get_piece(0, 0), None);
        assert_eq!(board.get_piece(1, 0), None);
        assert_eq!(board.get_piece(2, 0), None);
        assert_eq!(board.get_piece(3, 0), Some(Attacker));
        assert_eq!(board.get_piece(4, 0), Some(Attacker));
        assert_eq!(board.get_piece(5, 0), Some(Attacker));
        assert_eq!(board.get_piece(6, 0), Some(Attacker));
        assert_eq!(board.get_piece(7, 0), Some(Attacker));
        assert_eq!(board.get_piece(8, 0), None);
        assert_eq!(board.get_piece(9, 0), None);
        assert_eq!(board.get_piece(10, 0), None);

        assert_eq!(board.get_piece(0, 1), None);
        assert_eq!(board.get_piece(1, 1), None);
        assert_eq!(board.get_piece(2, 1), None);
        assert_eq!(board.get_piece(3, 1), None);
        assert_eq!(board.get_piece(4, 1), None);
        assert_eq!(board.get_piece(5, 1), Some(Attacker));
        assert_eq!(board.get_piece(6, 1), None);
        assert_eq!(board.get_piece(7, 1), None);
        assert_eq!(board.get_piece(8, 1), None);
        assert_eq!(board.get_piece(9, 1), None);
        assert_eq!(board.get_piece(10, 1), None);

        assert_eq!(board.get_piece(0, 2), None);
        assert_eq!(board.get_piece(1, 2), None);
        assert_eq!(board.get_piece(2, 2), None);
        assert_eq!(board.get_piece(3, 2), None);
        assert_eq!(board.get_piece(4, 2), None);
        assert_eq!(board.get_piece(5, 2), None);
        assert_eq!(board.get_piece(6, 2), None);
        assert_eq!(board.get_piece(7, 2), None);
        assert_eq!(board.get_piece(8, 2), None);
        assert_eq!(board.get_piece(9, 2), None);
        assert_eq!(board.get_piece(10, 2), None);

        assert_eq!(board.get_piece(0, 3), Some(Attacker));
        assert_eq!(board.get_piece(1, 3), None);
        assert_eq!(board.get_piece(2, 3), None);
        assert_eq!(board.get_piece(3, 3), None);
        assert_eq!(board.get_piece(4, 3), None);
        assert_eq!(board.get_piece(5, 3), Some(Defender));
        assert_eq!(board.get_piece(6, 3), None);
        assert_eq!(board.get_piece(7, 3), None);
        assert_eq!(board.get_piece(8, 3), None);
        assert_eq!(board.get_piece(9, 3), None);
        assert_eq!(board.get_piece(10, 3), Some(Attacker));

        assert_eq!(board.get_piece(0, 4), Some(Attacker));
        assert_eq!(board.get_piece(1, 4), None);
        assert_eq!(board.get_piece(2, 4), None);
        assert_eq!(board.get_piece(3, 4), None);
        assert_eq!(board.get_piece(4, 4), Some(Defender));
        assert_eq!(board.get_piece(5, 4), Some(Defender));
        assert_eq!(board.get_piece(6, 4), Some(Defender));
        assert_eq!(board.get_piece(7, 4), None);
        assert_eq!(board.get_piece(8, 4), None);
        assert_eq!(board.get_piece(9, 4), None);
        assert_eq!(board.get_piece(10, 4), Some(Attacker));

        assert_eq!(board.get_piece(0, 5), Some(Attacker));
        assert_eq!(board.get_piece(1, 5), Some(Attacker));
        assert_eq!(board.get_piece(2, 5), None);
        assert_eq!(board.get_piece(3, 5), Some(Defender));
        assert_eq!(board.get_piece(4, 5), Some(Defender));
        assert_eq!(board.get_piece(5, 5), Some(King));
        assert_eq!(board.get_piece(6, 5), Some(Defender));
        assert_eq!(board.get_piece(7, 5), Some(Defender));
        assert_eq!(board.get_piece(8, 5), None);
        assert_eq!(board.get_piece(9, 5), Some(Attacker));
        assert_eq!(board.get_piece(10, 5), Some(Attacker));

        assert_eq!(board.get_piece(0, 6), Some(Attacker));
        assert_eq!(board.get_piece(1, 6), None);
        assert_eq!(board.get_piece(2, 6), None);
        assert_eq!(board.get_piece(3, 6), None);
        assert_eq!(board.get_piece(4, 6), Some(Defender));
        assert_eq!(board.get_piece(5, 6), Some(Defender));
        assert_eq!(board.get_piece(6, 6), Some(Defender));
        assert_eq!(board.get_piece(7, 6), None);
        assert_eq!(board.get_piece(8, 6), None);
        assert_eq!(board.get_piece(9, 6), None);
        assert_eq!(board.get_piece(10, 6), Some(Attacker));

        assert_eq!(board.get_piece(0, 7), Some(Attacker));
        assert_eq!(board.get_piece(1, 7), None);
        assert_eq!(board.get_piece(2, 7), None);
        assert_eq!(board.get_piece(3, 7), None);
        assert_eq!(board.get_piece(4, 7), None);
        assert_eq!(board.get_piece(5, 7), Some(Defender));
        assert_eq!(board.get_piece(6, 7), None);
        assert_eq!(board.get_piece(7, 7), None);
        assert_eq!(board.get_piece(8, 7), None);
        assert_eq!(board.get_piece(9, 7), None);
        assert_eq!(board.get_piece(10, 7), Some(Attacker));

        assert_eq!(board.get_piece(0, 8), None);
        assert_eq!(board.get_piece(1, 8), None);
        assert_eq!(board.get_piece(2, 8), None);
        assert_eq!(board.get_piece(3, 8), None);
        assert_eq!(board.get_piece(4, 8), None);
        assert_eq!(board.get_piece(5, 8), None);
        assert_eq!(board.get_piece(6, 8), None);
        assert_eq!(board.get_piece(7, 8), None);
        assert_eq!(board.get_piece(8, 8), None);
        assert_eq!(board.get_piece(9, 8), None);
        assert_eq!(board.get_piece(10, 8), None);

        assert_eq!(board.get_piece(0, 9), None);
        assert_eq!(board.get_piece(1, 9), None);
        assert_eq!(board.get_piece(2, 9), None);
        assert_eq!(board.get_piece(3, 9), None);
        assert_eq!(board.get_piece(4, 9), None);
        assert_eq!(board.get_piece(5, 9), Some(Attacker));
        assert_eq!(board.get_piece(6, 9), None);
        assert_eq!(board.get_piece(7, 9), None);
        assert_eq!(board.get_piece(8, 9), None);
        assert_eq!(board.get_piece(9, 9), None);
        assert_eq!(board.get_piece(10, 9), None);

        assert_eq!(board.get_piece(0, 10), None);
        assert_eq!(board.get_piece(1, 10), None);
        assert_eq!(board.get_piece(2, 10), None);
        assert_eq!(board.get_piece(3, 10), Some(Attacker));
        assert_eq!(board.get_piece(4, 10), Some(Attacker));
        assert_eq!(board.get_piece(5, 10), Some(Attacker));
        assert_eq!(board.get_piece(6, 10), Some(Attacker));
        assert_eq!(board.get_piece(7, 10), Some(Attacker));
        assert_eq!(board.get_piece(8, 10), None);
        assert_eq!(board.get_piece(9, 10), None);
        assert_eq!(board.get_piece(10, 10), None);
        // }}}
    }
}
