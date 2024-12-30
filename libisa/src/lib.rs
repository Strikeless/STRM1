pub mod instruction;

pub type Word = u16;
pub type WordSigned = i16;

pub const BYTES_PER_WORD: usize = 2;

pub type Register = usize;
pub type Immediate = Word;

pub const REGISTER_COUNT: usize = 16;

pub fn word_to_bytes(word: Word) -> [u8; BYTES_PER_WORD] {
    [((word & 0xFF00) >> 8) as u8, (word & 0x00FF) as u8]
}

pub fn bytes_to_word(bytes: [u8; BYTES_PER_WORD]) -> Word {
    (bytes[0] as u16) << 8 | (bytes[1] as u16)
}
