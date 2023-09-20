use std::{error::Error, fmt::Display};

use crate::{CompactMove, HnefataflError, Turn};

use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    combinator::eof,
    IResult,
};

use log::warn;

#[derive(Debug)]
pub enum CommandError {
    // TooFewBytes(got, expected)
    TooFewBytes(u8, u8),
    InvalidCommandKind(u8),
    ParseError,
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::TooFewBytes(got, expected) => {
                write!(f, "Too few bytes: got {}, expected {}", got, expected)
            }
            CommandError::InvalidCommandKind(kind) => {
                write!(f, "Invalid command kind: {}", kind)
            }
            CommandError::ParseError => write!(f, "Parse error"),
        }
    }
}

impl Error for CommandError {}

#[repr(u8)]
enum CommandKind {
    Move = 0,
    IllegalMove = 1,
    MoveList = 2,
    Username = 3,
    RequestHistory = 4,
    ColorSelect = 5,
    Reset = 6,
    Observer = 7,

    IllegalCommand = 255,
}

/// Move contains a move
/// A user sends move to server, then server sends move to everybody
///
/// IllegalMove contains an error
///
/// MoveList contains a list of moves (usually as a response to request_history)
///
/// Username contains a string, the username
/// A user sends username to server, then server sends username to everybody
///
/// RequestHistory contains no data
/// A user sends request_history to server, then server sends move_list to user
///
/// ColorSelect contains a turn
/// A response from the server to a new user, telling them which color they are
/// Could also be received midgame
///
/// Reset contains no data
/// A user sends reset to server, then server sends reset to everybody
/// When received, reset the game
///
/// Observer contains no data
/// In establishing phase, server might respond with observer instead of ColorSelect if there are
/// already two players
///
/// IllegalCommand contains no data
/// Usual response when receiving an illegal command
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Move(CompactMove),
    IllegalMove(HnefataflError),
    MoveList(Vec<CompactMove>),
    Username(String),
    RequestHistory,
    ColorSelect(Turn),
    Reset,
    Observer,

    IllegalCommand,
}

fn parse_compact_move(input: &[u8]) -> IResult<&[u8], CompactMove> {
    let mut bytes = [0; 4];

    let (input, b) = take(4usize)(input)?;

    bytes.copy_from_slice(b);

    Ok((input, CompactMove::from(bytes)))
}

/// Parse a string that is prefixed by its length.
fn parse_string(input: &[u8]) -> IResult<&[u8], String> {
    let (input, length) = take(1usize)(input)?;
    let (input, name) = take(length[0])(input)?;

    let name = unsafe { std::str::from_utf8_unchecked(name) };

    Ok((input, name.to_string()))
}

fn parse_move(input: &[u8]) -> IResult<&[u8], Command> {
    let (input, _) = tag(&[CommandKind::Move as u8])(input)?;
    let (input, compact_move) = parse_compact_move(input)?;

    Ok((input, Command::Move(compact_move)))
}

fn parse_illegal_move(input: &[u8]) -> IResult<&[u8], Command> {
    let (input, _) = tag(&[CommandKind::IllegalMove as u8])(input)?;
    let (input, error) = take(1usize)(input)?;

    let error = unsafe { std::mem::transmute(error[0]) };

    Ok((input, Command::IllegalMove(error)))
}

fn parse_move_list(input: &[u8]) -> IResult<&[u8], Command> {
    let (input, _) = tag(&[CommandKind::MoveList as u8])(input)?;
    let (mut input, num) = take(1usize)(input)?;

    let mut moves = Vec::with_capacity(num[0] as usize);

    for _ in 0..num[0] {
        let (i, m) = parse_compact_move(input)?;
        input = i;
        moves.push(m);
    }

    Ok((input, Command::MoveList(moves)))
}

fn parse_initiate(input: &[u8]) -> IResult<&[u8], Command> {
    let (input, _) = tag(&[CommandKind::Username as u8])(input)?;
    let (input, name) = parse_string(input)?;

    Ok((input, Command::Username(name)))
}

fn parse_request_history(input: &[u8]) -> IResult<&[u8], Command> {
    let (input, _) = tag(&[CommandKind::RequestHistory as u8])(input)?;

    Ok((input, Command::RequestHistory))
}

fn parse_color_select(input: &[u8]) -> IResult<&[u8], Command> {
    let (input, _) = tag(&[CommandKind::ColorSelect as u8])(input)?;
    let (input, turn) = take(1usize)(input)?;

    let turn = unsafe { std::mem::transmute(turn[0]) };

    Ok((input, Command::ColorSelect(turn)))
}

fn parse_reset(input: &[u8]) -> IResult<&[u8], Command> {
    let (input, _) = tag(&[CommandKind::Reset as u8])(input)?;

    Ok((input, Command::Reset))
}

fn parse_observer(input: &[u8]) -> IResult<&[u8], Command> {
    let (input, _) = tag(&[CommandKind::Observer as u8])(input)?;

    Ok((input, Command::Observer))
}

fn parse_illegal_command(input: &[u8]) -> IResult<&[u8], Command> {
    let (input, _) = tag(&[CommandKind::IllegalCommand as u8])(input)?;

    Ok((input, Command::IllegalCommand))
}

fn parse_command(input: &[u8]) -> IResult<&[u8], Command> {
    let (input, command) = alt((
        parse_move,
        parse_illegal_move,
        parse_move_list,
        parse_initiate,
        parse_request_history,
        parse_color_select,
        parse_reset,
        parse_observer,
        parse_illegal_command,
    ))(input)?;
    let (input, _) = eof(input)?;

    Ok((input, command))
}

impl Command {
    pub fn from_binary(bytes: &[u8]) -> Result<Command, CommandError> {
        match parse_command(bytes) {
            Ok((_, command)) => Ok(command),
            Err(e) => {
                warn!("Error parsing command: {:?}", e);
                Err(CommandError::ParseError)
            }
        }
    }

    pub fn to_binary_vec(&self) -> Vec<u8> {
        let mut bytes = [0u8; 256];
        let length = self.to_binary(&mut bytes).unwrap();
        bytes[0..length].to_vec()
    }

    pub fn to_binary(&self, bytes: &mut [u8]) -> Result<usize, CommandError> {
        match self {
            Command::Move(compact_move) => {
                if bytes.len() < 5 {
                    return Err(CommandError::TooFewBytes(bytes.len() as u8, 5));
                }
                bytes[0] = CommandKind::Move as u8;
                let b: [u8; 4] = (*compact_move).into();
                bytes[1..5].copy_from_slice(&b);
                Ok(5)
            }
            Command::IllegalMove(error) => {
                if bytes.len() < 2 {
                    return Err(CommandError::TooFewBytes(bytes.len() as u8, 2));
                }
                bytes[0] = CommandKind::IllegalMove as u8;
                bytes[1] = *error as u8;
                Ok(2)
            }
            Command::MoveList(moves) => {
                if bytes.len() < 2 + moves.len() * 4 {
                    return Err(CommandError::TooFewBytes(
                        bytes.len() as u8,
                        2 + moves.len() as u8 * 4,
                    ));
                }
                bytes[0] = CommandKind::MoveList as u8;
                bytes[1] = moves.len() as u8;
                for (i, m) in moves.iter().enumerate() {
                    let b: [u8; 4] = (*m).into();
                    bytes[2 + i * 4..2 + (i + 1) * 4].copy_from_slice(&b);
                }
                Ok(2 + moves.len() * 4)
            }
            Command::Username(name) => {
                if bytes.len() < 2 + name.len() {
                    return Err(CommandError::TooFewBytes(
                        bytes.len() as u8,
                        2 + name.len() as u8,
                    ));
                }
                bytes[0] = CommandKind::Username as u8;
                bytes[1] = name.len() as u8;
                bytes[2..2 + name.len()].copy_from_slice(name.as_bytes());
                Ok(2 + name.len())
            }
            Command::RequestHistory => {
                if bytes.is_empty() {
                    return Err(CommandError::TooFewBytes(bytes.len() as u8, 1));
                }
                bytes[0] = CommandKind::RequestHistory as u8;
                Ok(1)
            }
            Command::ColorSelect(turn) => {
                if bytes.len() < 2 {
                    return Err(CommandError::TooFewBytes(bytes.len() as u8, 2));
                }
                bytes[0] = CommandKind::ColorSelect as u8;
                bytes[1] = *turn as u8;
                Ok(2)
            }
            Command::Reset => {
                if bytes.is_empty() {
                    return Err(CommandError::TooFewBytes(bytes.len() as u8, 1));
                }
                bytes[0] = CommandKind::Reset as u8;
                Ok(1)
            }
            Command::Observer => {
                if bytes.is_empty() {
                    return Err(CommandError::TooFewBytes(bytes.len() as u8, 1));
                }
                bytes[0] = CommandKind::Observer as u8;
                Ok(1)
            }
            Command::IllegalCommand => {
                if bytes.is_empty() {
                    return Err(CommandError::TooFewBytes(bytes.len() as u8, 1));
                }
                bytes[0] = CommandKind::IllegalCommand as u8;
                Ok(1)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Move;

    use super::*;

    fn test_to_from<const N: usize>(c: Command) {
        let mut bytes = [0u8; N];
        let count = c.to_binary(&mut bytes).unwrap();
        let c2 = Command::from_binary(&bytes).unwrap();

        let bytes2 = c2.to_binary_vec();
        let c3 = Command::from_binary(&bytes2).unwrap();

        assert_eq!(count, bytes2.len());
        assert_eq!(&c2, &c3);

        assert_eq!(count, N);
        assert_eq!(c, c2);
    }

    #[test]
    fn test_moves() {
        test_to_from::<5>(Command::Move(Move::from(0, 0, 1, 0).unwrap().compact()));
        test_to_from::<2>(Command::IllegalMove(HnefataflError::IllegalMove));

        test_to_from::<{ 2 + 4 * 4 }>(Command::MoveList(vec![
            Move::from(0, 0, 1, 0).unwrap().compact(),
            Move::from(0, 0, 2, 0).unwrap().compact(),
            Move::from(0, 0, 3, 0).unwrap().compact(),
            Move::from(0, 0, 4, 0).unwrap().compact(),
        ]));
        test_to_from::<6>(Command::Username("test".to_string()));
        test_to_from::<1>(Command::RequestHistory);
        test_to_from::<2>(Command::ColorSelect(Turn::White));
        test_to_from::<1>(Command::Reset);
        test_to_from::<1>(Command::Observer);

        test_to_from::<1>(Command::IllegalCommand);
    }
}
