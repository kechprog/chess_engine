// TODO:
// - Move should be u16 to repr everything that move contains

/*-------ARCHITECTURE--------*/

// | 6 bits | 6 bits | 4 bits |
// |  From  |   To   | Type   |
// |        16 bits           |


#[derive(Debug, Clone, Copy)]
pub enum MoveType {
    Normal    = 1 << 0,
    EnPassant = 1 << 1,
    Promotion = 1 << 2,
    Castling  = 1 << 3,
}

impl From<u16> for MoveType {
    fn from(value: u16) -> Self {
        match value {
            x if x == MoveType::Normal as u16    => MoveType::Normal,
            x if x == MoveType::EnPassant as u16 => MoveType::EnPassant,
            x if x == MoveType::Promotion as u16 => MoveType::Promotion,
            x if x == MoveType::Castling as u16  => MoveType::Castling,
            _ => panic!("Invalid value for MoveType: {}", value),
        }
    }
}

#[derive(Debug, Clone, Copy)]
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
        return (self.buf & 0xF).into();
    }

    pub fn _from(&self) -> usize {
        return ((self.buf >> 10) & 0x3F) as usize;
    }

    pub fn _to(&self) -> usize {
        return ((self.buf >> 4) & 0x3F) as usize;
    }


    fn from_u8(&self) -> u8 {
        todo!()
    }
    fn to_u8(&self) -> u8 {
        todo!()
    }
}
