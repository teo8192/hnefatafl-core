use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Debug, Display};

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
    UpDown,
    LeftRight,
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

#[derive(Debug, PartialEq)]
pub struct Move {
    from_x: i32,
    from_y: i32,
    to_x: i32,
    to_y: i32,
}

#[derive(PartialEq)]
pub struct Board {
    board: [[Option<Piece>; 11]; 11],
    turn: Turn,
}

impl Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Board")
            .field("board", &self.board)
            .field("turn", &self.turn)
            .finish()
    }
}

impl Board {
    /// Create a new board with the pieces in their starting positions
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

    /// Create an empty board
    pub fn empty() -> Self {
        Self {
            board: [[None; 11]; 11],
            turn: Turn::Black,
        }
    }

    /// Set the turn
    pub fn set_turn(&mut self, turn: Turn) {
        self.turn = turn;
    }

    pub fn get_turn(&self) -> Turn {
        self.turn
    }

    /// Get a piece, but do not check if the coordinates are within bounds
    pub fn get_piece_unchecked(&self, x: i32, y: i32) -> Option<Piece> {
        self.board[y as usize][x as usize]
    }

    /// Get a piece, but check if the coordinates are within bounds
    /// returns None if the coordinates are out of bounds
    pub fn get_piece_checked(&self, x: i32, y: i32) -> Option<Piece> {
        if !(0..=10).contains(&x) || !(0..=10).contains(&y) {
            return None;
        }

        self.board[y as usize][x as usize]
    }

    /// Place a piece on the board
    fn place(&mut self, piece: Option<Piece>, x: i32, y: i32) {
        self.board[y as usize][x as usize] = piece;
    }

    /// place a piece on the board, but do not check if the coordinates are within bounds
    fn place_piece(&mut self, piece: Piece, x: i32, y: i32) {
        self.place(Some(piece), x, y);
    }

    /// Remove a piece from the board
    fn remove_piece(&mut self, x: i32, y: i32) {
        self.place(None, x, y);
    }

    /// Move a piece without checking if the move is valid
    pub fn move_piece_uncheced(&mut self, x: i32, y: i32, new_x: i32, new_y: i32) {
        self.place(self.get_piece_unchecked(x, y), new_x, new_y);
        self.remove_piece(x, y);
    }

    /// Move a piece, checking if the move is valid
    pub fn move_piece(
        &mut self,
        x: i32,
        y: i32,
        new_x: i32,
        new_y: i32,
    ) -> Result<Vec<Piece>, HnefataflError> {
        // Important to check if the bounds are met before trying to access the piece
        if !(0..=10).contains(&x) || !(0..=10).contains(&y) {
            return Err(HnefataflError::StartOutOfBounds);
        }
        if !(0..=10).contains(&new_x) || !(0..=10).contains(&new_y) {
            return Err(HnefataflError::TargetOutOfBounds);
        }
        // Check if bad direction (gotta be rook move)
        if x != new_x && y != new_y {
            return Err(HnefataflError::MoveNotHorVer);
        }

        let piece = self
            .get_piece_unchecked(x, y)
            .ok_or(HnefataflError::NoPieceToMove)?;

        if !self.turn.is_same_color(&piece) {
            return Err(HnefataflError::WrongPieceColor);
        }

        if piece != Piece::King && self.is_fortress(new_x, new_y) {
            return Err(HnefataflError::IsProtectedTile);
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
                if self.get_piece_unchecked(i, j).is_some() {
                    return Err(HnefataflError::PieceInTheWay);
                }
            }
        }

        self.remove_piece(x, y);
        self.place_piece(piece, new_x, new_y);

        // try capture in all directions
        let mut captures = Vec::new();
        let mut capture = |x, y, dir| {
            if let Some(p) = self.try_capture(x, y, dir) {
                captures.push(p)
            }
        };
        capture(new_x, new_y + 1, Direction::UpDown);
        capture(new_x, new_y - 1, Direction::UpDown);
        capture(new_x + 1, new_y, Direction::LeftRight);
        capture(new_x - 1, new_y, Direction::LeftRight);

        self.turn = self.turn.opposite();

        // TODO: Check for win conditions (king in the corner, king captured)

        Ok(captures)
    }

    /// Check if the tile is a fortress tile.
    ///
    /// The fortress tiles are (0,0), (0,10), (10,0), (10,10) and (5,5).
    /// Only the king may occupy a fortress.
    ///
    /// The arguments are not checked if they are within bounds
    fn is_fortress(&self, x: i32, y: i32) -> bool {
        matches!((x, y), (0, 0) | (0, 10) | (10, 0) | (10, 10) | (5, 5))
    }

    /// Checks if the specified tile is an enemy tile
    ///
    /// Returns false if the tile is out of bounds
    fn is_enemy(&self, start_piece: &Piece, x: i32, y: i32) -> bool {
        if !(0..10).contains(&x) || !(0..10).contains(&y) {
            return false;
        }

        let check_square = self.get_piece_unchecked(x, y);

        // if the king occupies a fortress, then the position is not an enemy to the white pieces
        // This choice could possibly be changed
        if let Some(piece) = check_square {
            !start_piece.is_same_color(&piece)
        } else {
            // if the square is empty, but is a fortress, then it is an enemy to all pieces
            // if it is an empty, ordinary tile, then it is not an enemy
            self.is_fortress(x, y)
        }
    }

    /// Try to capture a piece.
    /// If this piece is captured, then return the piece.
    fn try_capture(&mut self, x: i32, y: i32, direction: Direction) -> Option<Piece> {
        let p = self.get_piece_checked(x, y)?;

        let (lx, ly) = match direction {
            Direction::UpDown => (x, y - 1),
            Direction::LeftRight => (x - 1, y),
        };

        let (rx, ry) = match direction {
            Direction::UpDown => (x, y + 1),
            Direction::LeftRight => (x + 1, y),
        };

        // checking for normal capture
        if p != Piece::King && self.is_enemy(&p, lx, ly) && self.is_enemy(&p, rx, ry) {
            self.remove_piece(x, y);
            return Some(p);
        }

        // King capture
        if p == Piece::King
            && self.is_enemy(&p, x + 1, y)
            && self.is_enemy(&p, x - 1, y)
            && self.is_enemy(&p, x, y + 1)
            && self.is_enemy(&p, x, y - 1)
        {
            self.remove_piece(x, y);
            return Some(p);
        }

        None
    }

    /// Returns a list of all target tiles available from the specified tile
    /// This does check whose turn it is
    fn moves_from(&self, x: i32, y: i32) -> Vec<(i32, i32)> {
        if !(0..=10).contains(&x) || !(0..=10).contains(&y) {
            return Vec::new();
        }

        // We have already checked the bounds, no need to check again
        let p = self.get_piece_unchecked(x, y);
        if p.is_none() || !self.turn.is_same_color(&p.unwrap()) {
            return Vec::new();
        }

        // Safe to unwrap, if it is none, then we have already returned
        let p = p.unwrap();
        let mut moves = Vec::new();

        // check the square.
        // Return true if the square is occupied, false if it is empty
        // (And some logic to handle the fortress)
        let mut check_square = |x, y| {
            let check_square = self.get_piece_unchecked(x, y);

            if check_square.is_none()
                && (!self.is_fortress(x, y) || (self.is_fortress(x, y) && p == Piece::King))
            {
                moves.push((x, y));
                false
            } else {
                println!("Hit");
                true
            }
        };

        // Check up
        for i in (0..y).rev() {
            if check_square(x, i) {
                break;
            }
        }

        // Check down
        for i in (y + 1)..11 {
            if check_square(x, i) {
                break;
            }
        }

        // Check left
        for i in (0..x).rev() {
            if check_square(i, y) {
                break;
            }
        }

        // Check right
        for i in (x + 1)..11 {
            if check_square(i, y) {
                break;
            }
        }

        moves
    }

    /// Returns all available moves right now
    pub fn available_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        for x in 0..11 {
            for y in 0..11 {
                if let Some(true) = self
                    .get_piece_unchecked(x, y)
                    .map(|p| self.turn.is_same_color(&p))
                {
                    moves.extend(self.moves_from(x, y).into_iter().map(|(to_x, to_y)| Move {
                        from_x: x,
                        from_y: y,
                        to_x,
                        to_y,
                    }));
                }
            }
        }
        moves
    }
}

// {{{ Display

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Turn: {:?}", self.turn)?;
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

// {{{ Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board() {
        let board = Board::new();

        use Piece::*;

        // {{{ A lot of asserts
        assert_eq!(board.get_piece_unchecked(0, 0), None);
        assert_eq!(board.get_piece_unchecked(1, 0), None);
        assert_eq!(board.get_piece_unchecked(2, 0), None);
        assert_eq!(board.get_piece_unchecked(3, 0), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(4, 0), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(5, 0), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(6, 0), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(7, 0), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(8, 0), None);
        assert_eq!(board.get_piece_unchecked(9, 0), None);
        assert_eq!(board.get_piece_unchecked(10, 0), None);

        assert_eq!(board.get_piece_unchecked(0, 1), None);
        assert_eq!(board.get_piece_unchecked(1, 1), None);
        assert_eq!(board.get_piece_unchecked(2, 1), None);
        assert_eq!(board.get_piece_unchecked(3, 1), None);
        assert_eq!(board.get_piece_unchecked(4, 1), None);
        assert_eq!(board.get_piece_unchecked(5, 1), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(6, 1), None);
        assert_eq!(board.get_piece_unchecked(7, 1), None);
        assert_eq!(board.get_piece_unchecked(8, 1), None);
        assert_eq!(board.get_piece_unchecked(9, 1), None);
        assert_eq!(board.get_piece_unchecked(10, 1), None);

        assert_eq!(board.get_piece_unchecked(0, 2), None);
        assert_eq!(board.get_piece_unchecked(1, 2), None);
        assert_eq!(board.get_piece_unchecked(2, 2), None);
        assert_eq!(board.get_piece_unchecked(3, 2), None);
        assert_eq!(board.get_piece_unchecked(4, 2), None);
        assert_eq!(board.get_piece_unchecked(5, 2), None);
        assert_eq!(board.get_piece_unchecked(6, 2), None);
        assert_eq!(board.get_piece_unchecked(7, 2), None);
        assert_eq!(board.get_piece_unchecked(8, 2), None);
        assert_eq!(board.get_piece_unchecked(9, 2), None);
        assert_eq!(board.get_piece_unchecked(10, 2), None);

        assert_eq!(board.get_piece_unchecked(0, 3), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(1, 3), None);
        assert_eq!(board.get_piece_unchecked(2, 3), None);
        assert_eq!(board.get_piece_unchecked(3, 3), None);
        assert_eq!(board.get_piece_unchecked(4, 3), None);
        assert_eq!(board.get_piece_unchecked(5, 3), Some(Defender));
        assert_eq!(board.get_piece_unchecked(6, 3), None);
        assert_eq!(board.get_piece_unchecked(7, 3), None);
        assert_eq!(board.get_piece_unchecked(8, 3), None);
        assert_eq!(board.get_piece_unchecked(9, 3), None);
        assert_eq!(board.get_piece_unchecked(10, 3), Some(Attacker));

        assert_eq!(board.get_piece_unchecked(0, 4), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(1, 4), None);
        assert_eq!(board.get_piece_unchecked(2, 4), None);
        assert_eq!(board.get_piece_unchecked(3, 4), None);
        assert_eq!(board.get_piece_unchecked(4, 4), Some(Defender));
        assert_eq!(board.get_piece_unchecked(5, 4), Some(Defender));
        assert_eq!(board.get_piece_unchecked(6, 4), Some(Defender));
        assert_eq!(board.get_piece_unchecked(7, 4), None);
        assert_eq!(board.get_piece_unchecked(8, 4), None);
        assert_eq!(board.get_piece_unchecked(9, 4), None);
        assert_eq!(board.get_piece_unchecked(10, 4), Some(Attacker));

        assert_eq!(board.get_piece_unchecked(0, 5), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(1, 5), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(2, 5), None);
        assert_eq!(board.get_piece_unchecked(3, 5), Some(Defender));
        assert_eq!(board.get_piece_unchecked(4, 5), Some(Defender));
        assert_eq!(board.get_piece_unchecked(5, 5), Some(King));
        assert_eq!(board.get_piece_unchecked(6, 5), Some(Defender));
        assert_eq!(board.get_piece_unchecked(7, 5), Some(Defender));
        assert_eq!(board.get_piece_unchecked(8, 5), None);
        assert_eq!(board.get_piece_unchecked(9, 5), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(10, 5), Some(Attacker));

        assert_eq!(board.get_piece_unchecked(0, 6), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(1, 6), None);
        assert_eq!(board.get_piece_unchecked(2, 6), None);
        assert_eq!(board.get_piece_unchecked(3, 6), None);
        assert_eq!(board.get_piece_unchecked(4, 6), Some(Defender));
        assert_eq!(board.get_piece_unchecked(5, 6), Some(Defender));
        assert_eq!(board.get_piece_unchecked(6, 6), Some(Defender));
        assert_eq!(board.get_piece_unchecked(7, 6), None);
        assert_eq!(board.get_piece_unchecked(8, 6), None);
        assert_eq!(board.get_piece_unchecked(9, 6), None);
        assert_eq!(board.get_piece_unchecked(10, 6), Some(Attacker));

        assert_eq!(board.get_piece_unchecked(0, 7), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(1, 7), None);
        assert_eq!(board.get_piece_unchecked(2, 7), None);
        assert_eq!(board.get_piece_unchecked(3, 7), None);
        assert_eq!(board.get_piece_unchecked(4, 7), None);
        assert_eq!(board.get_piece_unchecked(5, 7), Some(Defender));
        assert_eq!(board.get_piece_unchecked(6, 7), None);
        assert_eq!(board.get_piece_unchecked(7, 7), None);
        assert_eq!(board.get_piece_unchecked(8, 7), None);
        assert_eq!(board.get_piece_unchecked(9, 7), None);
        assert_eq!(board.get_piece_unchecked(10, 7), Some(Attacker));

        assert_eq!(board.get_piece_unchecked(0, 8), None);
        assert_eq!(board.get_piece_unchecked(1, 8), None);
        assert_eq!(board.get_piece_unchecked(2, 8), None);
        assert_eq!(board.get_piece_unchecked(3, 8), None);
        assert_eq!(board.get_piece_unchecked(4, 8), None);
        assert_eq!(board.get_piece_unchecked(5, 8), None);
        assert_eq!(board.get_piece_unchecked(6, 8), None);
        assert_eq!(board.get_piece_unchecked(7, 8), None);
        assert_eq!(board.get_piece_unchecked(8, 8), None);
        assert_eq!(board.get_piece_unchecked(9, 8), None);
        assert_eq!(board.get_piece_unchecked(10, 8), None);

        assert_eq!(board.get_piece_unchecked(0, 9), None);
        assert_eq!(board.get_piece_unchecked(1, 9), None);
        assert_eq!(board.get_piece_unchecked(2, 9), None);
        assert_eq!(board.get_piece_unchecked(3, 9), None);
        assert_eq!(board.get_piece_unchecked(4, 9), None);
        assert_eq!(board.get_piece_unchecked(5, 9), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(6, 9), None);
        assert_eq!(board.get_piece_unchecked(7, 9), None);
        assert_eq!(board.get_piece_unchecked(8, 9), None);
        assert_eq!(board.get_piece_unchecked(9, 9), None);
        assert_eq!(board.get_piece_unchecked(10, 9), None);

        assert_eq!(board.get_piece_unchecked(0, 10), None);
        assert_eq!(board.get_piece_unchecked(1, 10), None);
        assert_eq!(board.get_piece_unchecked(2, 10), None);
        assert_eq!(board.get_piece_unchecked(3, 10), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(4, 10), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(5, 10), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(6, 10), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(7, 10), Some(Attacker));
        assert_eq!(board.get_piece_unchecked(8, 10), None);
        assert_eq!(board.get_piece_unchecked(9, 10), None);
        assert_eq!(board.get_piece_unchecked(10, 10), None);
        // }}}
    }

    #[test]
    fn test_move_unchecked() {
        let mut board = Board::new();

        board.move_piece_uncheced(0, 7, 5, 7);

        assert_eq!(board.get_piece_unchecked(0, 7), None);
        assert_eq!(board.get_piece_unchecked(5, 7), Some(Piece::Attacker));
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

        assert_eq!(board.move_piece(0, 7, 4, 7), Ok(vec![]));
        assert_eq!(board.get_piece_unchecked(0, 7), None);
        assert_eq!(board.get_piece_unchecked(4, 7), Some(Piece::Attacker));
    }

    #[test]
    fn normal_capture() {
        // Setup board
        let mut board = Board::empty();
        board.place_piece(Piece::Attacker, 3, 3);
        board.place_piece(Piece::Attacker, 5, 7);
        board.place_piece(Piece::Defender, 4, 3);

        // Expected board
        let mut expected_board = Board::empty();
        expected_board.place_piece(Piece::Attacker, 3, 3);
        expected_board.place_piece(Piece::Attacker, 5, 3);
        expected_board.set_turn(Turn::White);

        let expected_captures = vec![Piece::Defender];

        // Make move
        let captured = board.move_piece(5, 7, 5, 3).unwrap();

        // Test
        assert_eq!(board, expected_board);
        assert_eq!(captured, expected_captures);
    }

    #[test]
    fn capturing_with_fortress_assistance() {
        // Setup board
        let mut board = Board::empty();
        board.place_piece(Piece::Attacker, 1, 0);
        board.place_piece(Piece::Defender, 2, 3);
        board.set_turn(Turn::White);

        // Expected board
        let mut expected_board = Board::empty();
        expected_board.place_piece(Piece::Defender, 2, 0);
        expected_board.set_turn(Turn::Black);

        let expected_captures = vec![Piece::Attacker];

        // Make move
        let captured = board.move_piece(2, 3, 2, 0).unwrap();

        assert_eq!(board, expected_board);
        assert_eq!(captured, expected_captures);
    }

    #[test]
    fn king_being_captured() {
        // try a king capture with the fortress
        // Setup board
        let mut board = Board::empty();
        board.set_turn(Turn::Black);

        board.place_piece(Piece::King, 4, 5);

        // Stationary attackers
        board.place_piece(Piece::Attacker, 4, 4);
        board.place_piece(Piece::Attacker, 4, 6);

        // The sneaky attacker
        board.place_piece(Piece::Attacker, 1, 5);

        // Expected board
        let mut expected_board = Board::empty();
        expected_board.place_piece(Piece::Attacker, 4, 4);
        expected_board.place_piece(Piece::Attacker, 4, 6);
        expected_board.place_piece(Piece::Attacker, 3, 5);
        expected_board.set_turn(Turn::White);

        let expected_captures = vec![Piece::King];

        // Make move
        let captured = board.move_piece(1, 5, 3, 5).unwrap();

        assert_eq!(board, expected_board);
        assert_eq!(captured, expected_captures);
    }

    #[test]
    fn available_moves_from_king_include_fortress() {
        let mut board = Board::empty();
        board.set_turn(Turn::White);
        board.place_piece(Piece::King, 0, 5);
        board.place_piece(Piece::Attacker, 3, 5);
        board.place_piece(Piece::Defender, 0, 9);

        let mut expected_moves = vec![
            (0, 0),
            (0, 1),
            (0, 2),
            (0, 3),
            (0, 4),
            (0, 6),
            (0, 7),
            (0, 8),
            (1, 5),
            (2, 5),
        ];

        let available_moves = board.moves_from(0, 5);

        assert_eq!(available_moves.len(), expected_moves.len());

        for expected_move in expected_moves.drain(..) {
            assert!(available_moves.contains(&expected_move));
        }
    }

    #[test]
    fn available_moves_from_defender_exclude_fortress() {
        let mut board = Board::empty();
        board.set_turn(Turn::White);
        board.place_piece(Piece::King, 0, 5);
        board.place_piece(Piece::Attacker, 3, 5);
        board.place_piece(Piece::Defender, 0, 9);

        let mut expected_moves = vec![
            (0, 8),
            (0, 7),
            (0, 6),
            (1, 9),
            (2, 9),
            (3, 9),
            (4, 9),
            (5, 9),
            (6, 9),
            (7, 9),
            (8, 9),
            (9, 9),
            (10, 9),
        ];

        let available_moves = board.moves_from(0, 9);

        assert_eq!(available_moves.len(), expected_moves.len());

        for expected_move in expected_moves.drain(..) {
            assert!(available_moves.contains(&expected_move));
        }
    }

    #[test]
    fn available_moves_for_defender() {
        let mut board = Board::empty();
        board.set_turn(Turn::White);
        board.place_piece(Piece::King, 0, 5);
        board.place_piece(Piece::Attacker, 3, 5);
        board.place_piece(Piece::Defender, 0, 9);

        let expected_moves_defender = vec![
            (0, 8),
            (0, 7),
            (0, 6),
            (1, 9),
            (2, 9),
            (3, 9),
            (4, 9),
            (5, 9),
            (6, 9),
            (7, 9),
            (8, 9),
            (9, 9),
            (10, 9),
        ];
        let expected_moves_king = vec![
            (0, 0),
            (0, 1),
            (0, 2),
            (0, 3),
            (0, 4),
            (0, 6),
            (0, 7),
            (0, 8),
            (1, 5),
            (2, 5),
        ];

        let expected_moves = expected_moves_defender
            .into_iter()
            .map(|(to_x, to_y)| Move {
                from_x: 0,
                from_y: 9,
                to_x,
                to_y,
            })
            .chain(expected_moves_king.into_iter().map(|(to_x, to_y)| Move {
                from_x: 0,
                from_y: 5,
                to_x,
                to_y,
            }))
            .collect::<Vec<_>>();

        let available_moves = board.available_moves();

        assert_eq!(available_moves.len(), expected_moves.len());

        for expected_move in expected_moves {
            assert!(available_moves.contains(&expected_move));
        }
    }
}
// }}}
