use std::io::Cursor;

use byteorder::{BigEndian, ByteOrder, NativeEndian, ReadBytesExt, WriteBytesExt};

type HostEndian = NativeEndian;
type GuestEndian = BigEndian;

pub trait EndianRewriteExt: AsRef<[u8]> {
    fn endian_rewrite_to_host(&self) -> Vec<u8> {
        self.endian_rewrite::<GuestEndian, HostEndian>()
    }

    fn endian_rewrite_to_guest(&self) -> Vec<u8> {
        self.endian_rewrite::<HostEndian, GuestEndian>()
    }

    fn endian_rewrite<EF, ET>(&self) -> Vec<u8>
    where
        EF: ByteOrder,
        ET: ByteOrder,
    {
        let mut reader = Cursor::new(self);
        let mut writer = Vec::new();

        while let Ok(word) = reader.read_u16::<EF>() {
            writer.write_u16::<ET>(word).unwrap();
        }

        writer
    }
}

impl<T> EndianRewriteExt for T where T: AsRef<[u8]> {}
