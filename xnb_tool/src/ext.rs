use byteorder::ReadBytesExt;

trait MyReadBytesExt: ReadBytesExt {
    fn read_7bit_encoded_i32(&mut self) -> std::io::Result<i32>;
}

impl<R: ReadBytesExt> MyReadBytesExt for R {
    fn read_7bit_encoded_i32(&mut self) -> std::io::Result<i32> {
        let mut result: i32 = 0;
        let mut bits_read = 0;

        loop {
            let byte = self.read_u8()? as i32;
            result |= (byte & 0x7f) << bits_read;
            bits_read += 7;

            if byte & 0x80 == 0 {
                break;
            }
        }

        Ok(result)
    }
}
