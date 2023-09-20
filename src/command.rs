use std::{error::Error, fmt::Display};

use crate::{CompactMove, HnefataflError};

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
    Initiate = 3,
    RequestHistory = 4,
    IllegalCommand = 255,
}

#[derive(Clone)]
pub enum Command {
    Move(CompactMove),
    IllegalMove(HnefataflError),
    MoveList(Vec<CompactMove>),
    Initiate(String),
    RequestHistory,
    IllegalCommand,
}

fn parse_move(input: &[u8]) -> IResult<&[u8], Command> {
    let mut bytes = [0; 4];

    let (input, _) = tag(&[CommandKind::Move as u8])(input)?;
    let (input, b) = take(4usize)(input)?;

    bytes.copy_from_slice(b);

    Ok((input, Command::Move(CompactMove::from(bytes))))
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
        let (i, m) = parse_move(input)?;
        input = i;
        moves.push(m);
    }

    Ok((input, Command::MoveList(Vec::new())))
}

fn parse_initiate(input: &[u8]) -> IResult<&[u8], Command> {
    let (input, _) = tag(&[CommandKind::Initiate as u8])(input)?;
    let (input, length) = take(1usize)(input)?;
    let (input, name) = take(length[0])(input)?;

    let name = unsafe { std::str::from_utf8_unchecked(name) };

    Ok((input, Command::Initiate(name.to_string())))
}

fn parse_request_history(input: &[u8]) -> IResult<&[u8], Command> {
    let (input, _) = tag(&[CommandKind::RequestHistory as u8])(input)?;

    Ok((input, Command::RequestHistory))
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

    pub fn to_binary(&self, bytes: &mut [u8]) -> Result<(), CommandError> {
        match self {
            Command::Move(compact_move) => {
                if bytes.len() < 5 {
                    return Err(CommandError::TooFewBytes(bytes.len() as u8, 5));
                }
                bytes[0] = CommandKind::Move as u8;
                let b: [u8; 4] = (*compact_move).into();
                bytes[1..5].copy_from_slice(&b);
                Ok(())
            }
            Command::IllegalMove(error) => {
                if bytes.len() < 2 {
                    return Err(CommandError::TooFewBytes(bytes.len() as u8, 2));
                }
                bytes[0] = CommandKind::IllegalMove as u8;
                bytes[1] = *error as u8;
                Ok(())
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
                Ok(())
            }
            Command::Initiate(name) => {
                if bytes.len() < 2 + name.len() {
                    return Err(CommandError::TooFewBytes(
                        bytes.len() as u8,
                        2 + name.len() as u8,
                    ));
                }
                bytes[0] = CommandKind::Initiate as u8;
                bytes[1] = name.len() as u8;
                bytes[2..2 + name.len()].copy_from_slice(name.as_bytes());
                Ok(())
            }
            Command::RequestHistory => {
                if bytes.is_empty() {
                    return Err(CommandError::TooFewBytes(bytes.len() as u8, 1));
                }
                bytes[0] = CommandKind::RequestHistory as u8;
                Ok(())
            }
            Command::IllegalCommand => {
                if bytes.is_empty() {
                    return Err(CommandError::TooFewBytes(bytes.len() as u8, 1));
                }
                bytes[0] = CommandKind::IllegalCommand as u8;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Move;

    use super::*;

    #[test]
    fn test_command() {
        let mut bytes = [0u8; 5];
        let command = Command::Move(Move::from(0, 0, 1, 0).unwrap().compact());
        command.to_binary(&mut bytes).unwrap();
        assert_eq!(bytes, [0, 0, 5, 0, 0]);

        let command = Command::IllegalMove(HnefataflError::IllegalMove);
        bytes = [0u8; 5];

        command.to_binary(&mut bytes).unwrap();
        assert_eq!(bytes, [1, HnefataflError::IllegalMove as u8, 0, 0, 0]);
    }
}
