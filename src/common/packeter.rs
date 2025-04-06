use std::{
    cell::RefMut,
    io::{Read, Write},
    ops::DerefMut,
};

type PacketSize = u32;

pub(crate) struct PacketReader {
    buffer: Vec<u8>,
}

impl Default for PacketReader {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
        }
    }
}

pub(crate) struct PacketReaderIter<'a> {
    reader: &'a mut PacketReader,
}

impl<'a> Iterator for PacketReaderIter<'a> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.buffer.len() >= size_of::<PacketSize>() {
            let size = PacketSize::from_be_bytes(
                self.reader.buffer[..size_of::<PacketSize>()]
                    .try_into()
                    .unwrap(),
            ) as usize;

            if self.reader.buffer.len() >= size_of::<PacketSize>() + size {
                self.reader.buffer.drain(0..size_of::<PacketSize>());
                Some(self.reader.buffer.drain(0..size).collect())
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl PacketReader {
    pub(crate) fn read<'a, R: Read>(
        &'a mut self,
        read: &mut R,
    ) -> Result<PacketReaderIter<'a>, std::io::Error> {
        Self::ignore_would_block(std::io::copy(read, &mut self.buffer), 0)?;
        Ok(PacketReaderIter { reader: self })
    }

    pub(crate) fn read_ref<'a, R: Read>(
        &'a mut self,
        mut read: RefMut<R>,
    ) -> Result<PacketReaderIter<'a>, std::io::Error> {
        Self::ignore_would_block(std::io::copy(read.deref_mut(), &mut self.buffer), 0)?;
        Ok(PacketReaderIter { reader: self })
    }

    fn ignore_would_block<T>(r: std::io::Result<T>, default: T) -> std::io::Result<T> {
        match r {
            Ok(ok) => Ok(ok),
            Err(err) => {
                if err.kind() == std::io::ErrorKind::WouldBlock {
                    Ok(default)
                } else {
                    Err(err)
                }
            }
        }
    }

    #[cfg(test)]
    fn bytes_stored(&self) -> usize {
        self.buffer.len()
    }
}

pub(crate) struct PacketWriter {}

impl PacketWriter {
    pub fn write<W: Write>(write: &mut W, data: &[u8]) -> Result<(), std::io::Error> {
        let size = write.write(&PacketSize::to_be_bytes(data.len() as u32))?;
        assert_eq!(size, size_of::<PacketSize>());
        let size = write.write(data)?;
        assert_eq!(size, data.len());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, io::Read};

    use super::{PacketReader, PacketWriter};

    #[test]
    fn simple_read_write() {
        let mut pipe: VecDeque<u8> = Default::default();
        let mut reader: PacketReader = Default::default();

        PacketWriter::write(&mut pipe, &[1, 2, 3, 4]).unwrap();
        PacketWriter::write(&mut pipe, &[5, 6, 7, 8]).unwrap();
        PacketWriter::write(&mut pipe, &[9, 0, 1, 2]).unwrap();

        let packets: Vec<_> = reader.read(&mut pipe).unwrap().collect();
        assert_eq!(
            packets,
            vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![9, 0, 1, 2]]
        );
        assert_eq!(reader.bytes_stored(), 0);
    }

    #[test]
    fn partial_read_write() {
        let mut input_pipe: VecDeque<u8> = Default::default();
        let mut output_pipe: VecDeque<u8> = Default::default();
        let mut reader: PacketReader = Default::default();

        PacketWriter::write(&mut input_pipe, &[1, 2, 3, 4]).unwrap();
        PacketWriter::write(&mut input_pipe, &[5, 6, 7, 8]).unwrap();
        PacketWriter::write(&mut input_pipe, &[9, 0, 1, 2]).unwrap();

        std::io::copy(&mut input_pipe.by_ref().take(10), &mut output_pipe).unwrap();

        let packets: Vec<_> = reader.read(&mut output_pipe).unwrap().collect();
        assert_eq!(packets, vec![vec![1, 2, 3, 4]]);
        assert_eq!(reader.bytes_stored(), 2);

        std::io::copy(&mut input_pipe, &mut output_pipe).unwrap();

        let packets: Vec<_> = reader.read(&mut output_pipe).unwrap().collect();
        assert_eq!(packets, vec![vec![5, 6, 7, 8], vec![9, 0, 1, 2]]);
        assert_eq!(reader.bytes_stored(), 0);
    }
}
