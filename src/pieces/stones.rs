pub const WHITE_STONE: char = '⚫';
pub const BLACK_STONE: char = '⚪';
pub const EMPTY_STONE: char = '.';

#[derive(Eq, PartialEq)]
pub enum Stones{
    White = 2,
    Black = 1,
    Empty = 0
}


impl From<u8> for Stones{
    fn from(x: u8) -> Self {
        match x {
            2 => Stones::White,
            1 => Stones::Black,
            0 => Stones::Empty,
            _ => panic!("Error int the convertion from u8 to Stone")
        }
    }
}
