// Move representation using compact 16-bit encoding
//
// This allows efficient storage and comparison of moves while encoding
// all necessary information (source, destination, and move type).

/*-------ARCHITECTURE--------*/

// | 6 bits | 6 bits | 4 bits |
// |  From  |   To   | Type   |
// |        16 bits           |


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveType {
    Normal           = 1,
    EnPassant        = 2,
    PromotionQueen   = 4,
    PromotionRook    = 5,
    PromotionBishop  = 6,
    PromotionKnight  = 7,
    Castling         = 8,
}

impl MoveType {
    pub fn is_promotion(&self) -> bool {
        matches!(self,
            MoveType::PromotionQueen |
            MoveType::PromotionRook |
            MoveType::PromotionBishop |
            MoveType::PromotionKnight
        )
    }
}

impl From<u16> for MoveType {
    fn from(value: u16) -> Self {
        match value {
            1 => MoveType::Normal,
            2 => MoveType::EnPassant,
            4 => MoveType::PromotionQueen,
            5 => MoveType::PromotionRook,
            6 => MoveType::PromotionBishop,
            7 => MoveType::PromotionKnight,
            8 => MoveType::Castling,
            _ => panic!("Invalid value for MoveType: {}", value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    buf: u16,
}

impl Move {
    pub fn new(from: u8, to: u8, move_type: MoveType) -> Move {
        let mut buf = 0u16;
        buf |= (from as u16) << 10;
        buf |= (to as u16) << 4;
        buf |= move_type as u16;

        Self { buf }
    }

    pub fn move_type(&self) -> MoveType {
        (self.buf & 0xF).into()
    }

    pub fn _from(&self) -> usize {
        ((self.buf >> 10) & 0x3F) as usize
    }

    pub fn _to(&self) -> usize {
        ((self.buf >> 4) & 0x3F) as usize
    }
}
