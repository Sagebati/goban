use crate::{SgfError, SgfErrorKind};
use std::ops::Not;

/// Indicates what color the token is related to
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Color {
    Black,
    White,
}

impl Not for Color {
    type Output = Color;
    fn not(self) -> Color {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

/// Enum describing all possible SGF Properties
#[derive(Debug, PartialEq, Clone)]
pub enum SgfToken {
    Add { color: Color, coordinate: (u8, u8) },
    Move { color: Color, coordinate_or_pass: Option<(u8, u8)> },
    Time { color: Color, time: u32 },
    PlayerName { color: Color, name: String },
    PlayerRank { color: Color, rank: String },
    Komi(f32),
    Event(String),
    Copyright(String),
    GameName(String),
    Place(String),
    Date(String),
    Size(u32, u32),
    TimeLimit(u32),
    Comment(String),
    Unknown((String, String)),
    Invalid((String, String)),
    Square { coordinate: (u8, u8) },
    Triangle { coordinate: (u8, u8) },
    Label { label: String, coordinate: (u8, u8) },
}

impl SgfToken {
    /// Converts a `identifier` and `value` pair to a SGF token
    ///
    /// Returns `SgfToken::Unknown((identifier, value))` for tokens without a matching identifier
    ///
    /// Returns `SgfToken::Invalid((identifier, value))` for tokens with a matching identifier, but invalid value
    ///
    /// ```rust
    /// use sgf_parser::*;
    ///
    /// let token = SgfToken::from_pair("B", "aa");
    /// assert_eq!(token, SgfToken::Move { color: Color::Black, coordinate_or_pass: Some((1, 1)) });
    ///
    /// let token = SgfToken::from_pair("B", "");
    /// assert_eq!(token, SgfToken::Move { color: Color::Black, coordinate_or_pass: None });
    ///
    /// let token = SgfToken::from_pair("B", "not_coord");
    /// assert_eq!(token, SgfToken::Invalid(("B".to_string(), "not_coord".to_string())));
    ///
    /// let token = SgfToken::from_pair("FOO", "aa");
    /// assert_eq!(token, SgfToken::Unknown(("FOO".to_string(), "aa".to_string())));
    /// ```
    pub fn from_pair(base_ident: &str, value: &str) -> SgfToken {
        let ident = base_ident
            .chars()
            .filter(|c| c.is_uppercase())
            .collect::<String>();
        let token: Option<SgfToken> = match ident.as_ref() {
            "LB" => split_label_text(value).and_then(|(coord, label)| {
                str_to_coordinates(coord)
                    .ok()
                    .map(|coordinate| SgfToken::Label {
                        label: label[1..].to_string(),
                        coordinate,
                    })
            }),
            "SQ" => str_to_coordinates(value)
                .ok()
                .map(|coordinate| SgfToken::Square { coordinate }),
            "TR" => str_to_coordinates(value)
                .ok()
                .map(|coordinate| SgfToken::Triangle { coordinate }),
            "AB" => str_to_coordinates(value)
                .ok()
                .map(|coordinate| SgfToken::Add {
                    color: Color::Black,
                    coordinate,
                }),
            "B" => move_str_to_coord(value)
                .ok()
                .map(|coordinate| SgfToken::Move {
                    color: Color::Black,
                    coordinate_or_pass: coordinate,
                }),
            "BL" => value.parse().ok().map(|time| SgfToken::Time {
                color: Color::Black,
                time,
            }),
            "PB" => Some(SgfToken::PlayerName {
                color: Color::Black,
                name: value.to_string(),
            }),
            "BR" => Some(SgfToken::PlayerRank {
                color: Color::Black,
                rank: value.to_string(),
            }),
            "AW" => str_to_coordinates(value)
                .ok()
                .map(|coordinate| SgfToken::Add {
                    color: Color::White,
                    coordinate,
                }),
            "W" => move_str_to_coord(value)
                .ok()
                .map(|coordinate| SgfToken::Move {
                    color: Color::White,
                    coordinate_or_pass: coordinate,
                }),
            "WL" => value.parse().ok().map(|time| SgfToken::Time {
                color: Color::White,
                time,
            }),
            "PW" => Some(SgfToken::PlayerName {
                color: Color::White,
                name: value.to_string(),
            }),
            "WR" => Some(SgfToken::PlayerRank {
                color: Color::White,
                rank: value.to_string(),
            }),
            "KM" => value.parse().ok().map(SgfToken::Komi),
            "SZ" => {
                if let Some((width, height)) = split_size_text(value) {
                    Some(SgfToken::Size(width, height))
                } else {
                    value.parse().ok().map(|size| SgfToken::Size(size, size))
                }
            }
            "TM" => value.parse().ok().map(SgfToken::TimeLimit),
            "EV" => Some(SgfToken::Event(value.to_string())),
            "C" => Some(SgfToken::Comment(value.to_string())),
            "GN" => Some(SgfToken::GameName(value.to_string())),
            "CR" => Some(SgfToken::Copyright(value.to_string())),
            "DT" => Some(SgfToken::Date(value.to_string())),
            "PC" => Some(SgfToken::Place(value.to_string())),
            _ => Some(SgfToken::Unknown((
                base_ident.to_string(),
                value.to_string(),
            ))),
        };
        match token {
            Some(token) => token,
            _ => SgfToken::Invalid((base_ident.to_string(), value.to_string())),
        }
    }

    /// Checks if the token is a root token as defined by the SGF spec.
    ///
    /// Root tokens can only occur in the root of a gametree collection, and they are invalid
    /// anywhere else
    ///
    /// ```
    /// use sgf_parser::*;
    ///
    /// let token = SgfToken::from_pair("B", "aa");
    /// assert!(!token.is_root_token());
    ///
    /// let token = SgfToken::from_pair("SZ", "19");
    /// assert!(token.is_root_token());
    /// ```
    pub fn is_root_token(&self) -> bool {
        use SgfToken::*;
        match self {
            Size(_, _) => true,
            _ => false,
        }
    }
}

impl Into<String> for &SgfToken {
    fn into(self) -> String {
        match self {
            SgfToken::Label { label, coordinate } => {
                let value = coordinate_to_str(Some(*coordinate));
                format!("LB[{}:{}]", value, label)
            }
            SgfToken::Square { coordinate } => {
                let value = coordinate_to_str(Some(*coordinate));
                format!("SQ[{}]", value)
            }
            SgfToken::Triangle { coordinate } => {
                let value = coordinate_to_str(Some(*coordinate));
                format!("TR[{}]", value)
            }
            SgfToken::Add { color, coordinate } => {
                let token = match color {
                    Color::Black => "AB",
                    Color::White => "AW",
                };
                let value = coordinate_to_str(Some(*coordinate));
                format!("{}[{}]", token, value)
            }
            SgfToken::Move { color, coordinate_or_pass } => {
                let token = match color {
                    Color::Black => "B",
                    Color::White => "W",
                };
                let value = coordinate_to_str(*coordinate_or_pass);
                format!("{}[{}]", token, value)
            }
            SgfToken::Time { color, time } => {
                let token = match color {
                    Color::Black => "BL",
                    Color::White => "WL",
                };
                format!("{}[{}]", token, time)
            }
            SgfToken::PlayerName { color, name } => {
                let token = match color {
                    Color::Black => "PB",
                    Color::White => "PW",
                };
                format!("{}[{}]", token, name)
            }
            SgfToken::PlayerRank { color, rank } => {
                let token = match color {
                    Color::Black => "BR",
                    Color::White => "WR",
                };
                format!("{}[{}]", token, rank)
            }
            SgfToken::Komi(komi) => format!("KM[{}]", komi),
            SgfToken::Size(width, height) if width == height => format!("SZ[{}]", width),
            SgfToken::Size(width, height) => format!("SZ[{}:{}]", width, height),
            SgfToken::TimeLimit(time) => format!("TM[{}]", time),
            SgfToken::Event(value) => format!("EV[{}]", value),
            SgfToken::Comment(value) => format!("C[{}]", value),
            SgfToken::GameName(value) => format!("GN[{}]", value),
            SgfToken::Copyright(value) => format!("CR[{}]", value),
            SgfToken::Date(value) => format!("DT[{}]", value),
            SgfToken::Place(value) => format!("PC[{}]", value),
            _ => panic!(),
        }
    }
}

impl Into<String> for SgfToken {
    fn into(self) -> String {
        (&self).into()
    }
}

/// Splits size input text (NN:MM) to corresponding width and height
fn split_size_text(input: &str) -> Option<(u32, u32)> {
    let index = input.find(':')?;
    let (width_part, height_part) = input.split_at(index);
    let width: u32 = width_part.parse().ok()?;
    let height: u32 = height_part[1..].parse().ok()?;
    Some((width, height))
}


/// Converts goban coordinates to string representation
/// Pass None to simulate a pass
fn coordinate_to_str(coordinate: Option<(u8, u8)>) -> String {
    if let Some(coord) = coordinate {
        /* let conv = |n| {
            // skips 'I' as a valid coordinate
            n + if n >= 9 { 97 } else { 96 }
        }; */
        let x = (coord.0+ 96) as char;
        let y = (coord.1 + 96) as char;
        [x, y].iter().collect()
    } else {
        String::new() // Empty string for and pass
    }
}

/// If possible, splits a label text into coordinate and label pair
fn split_label_text(input: &str) -> Option<(&str, &str)> {
    if input.len() >= 4 {
        Some(input.split_at(2))
    } else {
        None
    }
}

fn move_str_to_coord(input: &str) -> Result<Option<(u8, u8)>, SgfError> {
    if input.is_empty() {
        Ok(None)
    } else {
        match str_to_coordinates(input) {
            Ok(t) => Ok(Some(t)),
            Err(e) => Err(e)
        }
    }
}

/// Converts a string describing goban coordinates to numeric coordinates
fn str_to_coordinates(input: &str) -> Result<(u8, u8), SgfError> {
    if input.len() != 2 {
        Err(SgfErrorKind::ParseError.into())
    } else {
        let coords = input
            .to_lowercase()
            .as_bytes()
            .iter()
            .map(|c| convert_u8_to_coordinate(*c))
            .collect::<Vec<_>>();
        Ok((coords[0], coords[1]))
    }
}

/// Converts a u8 char to numeric coordinates
///
#[inline]
fn convert_u8_to_coordinate(c: u8) -> u8 {
    c - 96
}
