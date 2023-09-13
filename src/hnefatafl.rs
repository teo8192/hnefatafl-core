use std::cmp::Ordering;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum HnefataflError {
    NoPieceToMove,
    PieceInTheWay,
    StartOutOfBounds,
    TargetOutOfBounds,
    MoveNotHorVer,
    WrongPieceColor,
    IsProtectedTile,
    OtherError(String),
}

// {{{ impls for error

impl Display for HnefataflError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HnefataflError::NoPieceToMove => f.write_str("No piece to move"),
            HnefataflError::PieceInTheWay => f.write_str("Piece in the way"),
            HnefataflError::TargetOutOfBounds => f.write_str("Target of move out of bounds"),
            HnefataflError::StartOutOfBounds => f.write_str("Start of move out of bounds"),
            HnefataflError::MoveNotHorVer => f.write_str("Move is not horizontal nor vertical"),
            HnefataflError::WrongPieceColor => f.write_str("Trying to move the wrong piece color"),
            HnefataflError::IsProtectedTile => {
                f.write_str("Trying to move a soldier to a protected tile")
            }
            HnefataflError::OtherError(s) => f.write_str(s),
        }
    }
}

impl Error for HnefataflError {}

// }}}

enum Direction {
    UpDown, LeftRight
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Turn {
    White,
    Black,
}

trait Color {
    fn color(&self) -> Turn;
    fn is_same_color<C: Color>(&self, other: &C) -> bool {
        self.color() == other.color()
    }

    fn opposite(&self) -> Turn {
        match self.color() {
            Turn::White => Turn::Black,
            Turn::Black => Turn::White,
        }
    }
}

impl Color for Turn {
    fn color(&self) -> Turn {
        *self
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Piece {
    King,
    Defender,
    Attacker,
}

impl Color for Piece {
    fn color(&self) -> Turn {
        match self {
            Piece::King => Turn::White,
            Piece::Defender => Turn::White,
            Piece::Attacker => Turn::Black,
        }
    }
}

pub struct Board {
    board: [[Option<Piece>; 11]; 11],
    turn: Turn,
}

impl Board {
    pub fn new() -> Self {
        let mut board = Self::empty();

        // placing defenders
        for i in 3..=7 {
            let a = 2 - i32::abs(i - 5);

            for j in 5 - a..5 + a + 1 {
                board.place_piece(Piece::Defender, i, j);
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

    pub fn empty() -> Self {
        Self {
            board: [[None; 11]; 11],
            turn: Turn::Black,
        }
    }

    pub fn get_piece(&self, x: i32, y: i32) -> Option<Piece> {
        self.board[y as usize][x as usize]
    }

    fn place(&mut self, piece: Option<Piece>, x: i32, y: i32) {
        self.board[y as usize][x as usize] = piece;
    }

    fn place_piece(&mut self, piece: Piece, x: i32, y: i32) {
        self.place(Some(piece), x, y);
    }

    fn remove_piece(&mut self, x: i32, y: i32) {
        self.place(None, x, y);
    }

    pub fn move_piece_uncheced(&mut self, x: i32, y: i32, new_x: i32, new_y: i32) {
        self.place(self.get_piece(x, y), new_x, new_y);
        self.remove_piece(x, y);
    }

    pub fn move_piece(
        &mut self,
        x: i32,
        y: i32,
        new_x: i32,
        new_y: i32,
    ) -> Result<(), HnefataflError> {
        // Important to check ig the bounds are met before trying to access the piece
        if !(0..=10).contains(&x) || !(0..=10).contains(&y) {
            return Err(HnefataflError::StartOutOfBounds);
        }
        if !(0..=10).contains(&new_x) || !(0..=10).contains(&new_y) {
            return Err(HnefataflError::TargetOutOfBounds);
        }
        // Check if diagonal before accessing memory
        if x != new_x && y != new_y {
            return Err(HnefataflError::MoveNotHorVer);
        }

        let piece = self.get_piece(x, y).ok_or(HnefataflError::NoPieceToMove)?;

        if !self.turn.is_same_color(&piece) {
            return Err(HnefataflError::WrongPieceColor);
        }

        if piece != Piece::King {
            match (new_x, new_y) {
                (0, 0) | (10, 0) | (0, 10) | (10, 10) | (5, 5) => {
                    return Err(HnefataflError::IsProtectedTile);
                }
                _ => {}
            }
        }
        use Ordering::*;

        let (start_x, end_x, start_y, end_y) = match (x.cmp(&new_x), y.cmp(&new_y)) {
            (Less, Equal) => (x + 1, new_x, y, y),
            (Greater, Equal) => (new_x, x - 1, y, y),
            (Equal, Less) => (x, x, y + 1, new_y),
            (Equal, Greater) => (x, x, new_y, y - 1),
            (a, b) => Err(HnefataflError::OtherError(format!(
                "Unknown move: ({:?}, {:?})",
                a, b
            )))?,
        };

        for i in start_x..=end_x {
            for j in start_y..=end_y {
                if self.get_piece(i, j).is_some() {
                    return Err(HnefataflError::PieceInTheWay);
                }
            }
        }

        self.remove_piece(x, y);
        self.place_piece(piece, new_x, new_y);

        // TODO: Check for captures

        self.turn = self.turn.opposite();

        Ok(())
    }

    /// Try to capture a piece.
    /// If this piece is captured, then return the piece.
    fn try_capture(&mut self, x: i32, y: i32, direction: Direction) -> Option<Piece> {
        let p = self.get_piece(x, y)?;

        // TODO: Check for bounds of the board???

        let (lx, ly) = match direction {
            Direction::UpDown => (x, y - 1),
            Direction::LeftRight => (x - 1, y),
        };
        let lp = self.get_piece(lx, ly)?;

        let (rx, ry) = match direction {
            Direction::UpDown => (x, y + 1),
            Direction::LeftRight => (x + 1, y),
        };
        let rp = self.get_piece(rx, ry)?;

        if p != Piece::King && !p.is_same_color(&lp) && !p.is_same_color(&rp) {
            self.remove_piece(x, y);
            return Some(p);
        }

        // TODO: Check if the piece has the back to one of the five special tiles
        // TODO: Check for king capture

        None
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

    #[test]
    fn test_move_unchecked() {
        let mut board = Board::new();

        board.move_piece_uncheced(0, 7, 5, 7);

        assert_eq!(board.get_piece(0, 7), None);
        assert_eq!(board.get_piece(5, 7), Some(Piece::Attacker));
    }

    #[test]
    fn test_move() {
        let mut board = Board::new();

        // {{{ Errors on move
        assert_eq!(
            board.move_piece(1, 7, 4, 7),
            Err(HnefataflError::NoPieceToMove)
        );

        assert_eq!(
            board.move_piece(0, 3, 0, 0),
            Err(HnefataflError::IsProtectedTile)
        );

        assert_eq!(
            board.move_piece(0, 7, 3, 9),
            Err(HnefataflError::MoveNotHorVer)
        );

        assert_eq!(
            board.move_piece(0, 7, -2, 7),
            Err(HnefataflError::TargetOutOfBounds)
        );

        assert_eq!(
            board.move_piece(0, 7, 0, 11),
            Err(HnefataflError::TargetOutOfBounds)
        );

        assert_eq!(
            board.move_piece(-4, 7, 3, 7),
            Err(HnefataflError::StartOutOfBounds)
        );

        assert_eq!(
            board.move_piece(0, 19, 0, 7),
            Err(HnefataflError::StartOutOfBounds)
        );

        assert_eq!(
            board.move_piece(0, 7, 5, 7),
            Err(HnefataflError::PieceInTheWay)
        );

        assert_eq!(
            board.move_piece(3, 5, 2, 5),
            Err(HnefataflError::WrongPieceColor)
        );
        // }}}

        assert_eq!(board.move_piece(0, 7, 4, 7), Ok(()));
        assert_eq!(board.get_piece(0, 7), None);
        assert_eq!(board.get_piece(4, 7), Some(Piece::Attacker));
    }
}
