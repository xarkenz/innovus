pub const BOARD_SIZE: usize = 10;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum PieceType {
    Bomb = b'B',
    Marshal = b'1',
    General = b'2',
    Colonel = b'3',
    Major = b'4',
    Captain = b'5',
    Lieutenant = b'6',
    Sergeant = b'7',
    Miner = b'8',
    Scout = b'9',
    Spy = b'S',
    Flag = b'F',
}

impl PieceType {
    pub fn can_defeat(self, other: Self) -> bool {
        todo!()
    }
}

fn main() {

}