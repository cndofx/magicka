use byteorder::ReadBytesExt;

pub trait MyReadBytesExt: ReadBytesExt {
    fn read_bool(&mut self) -> std::io::Result<bool>;
    fn read_7bit_encoded_i32(&mut self) -> std::io::Result<i32>;
    fn read_7bit_length_string(&mut self) -> std::io::Result<String>;
    // fn read_null_terminated_string(&mut self) -> std::io::Result<String>;
}

impl<R: ReadBytesExt> MyReadBytesExt for R {
    fn read_bool(&mut self) -> std::io::Result<bool> {
        Ok(self.read_u8()? != 0)
    }

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

    fn read_7bit_length_string(&mut self) -> std::io::Result<String> {
        let len = self.read_7bit_encoded_i32()? as usize;
        let mut s = String::with_capacity(len);
        for _ in 0..len {
            s.push(self.read_u8()? as char);
        }
        Ok(s)
    }

    // fn read_null_terminated_string(&mut self) -> std::io::Result<String> {
    //     let mut s = String::new();
    //     loop {
    //         let c = self.read_u8()? as char;
    //         if c == '\0' {
    //             break;
    //         }
    //         s.push(c);
    //     }
    //     Ok(s)
    // }
}
