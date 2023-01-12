use std::io;

use tokio::io::{AsyncRead, AsyncReadExt};

pub struct LineReader<T: AsyncRead> {
    reader: T,
    buffer: Vec<u8>,
}
/// suboptimal (to say the least) implementation of an asynchronous line reader.
///
/// It will not throw a new  line at EOF if the last char is not an `\n`
///
/// DO NOT USE IT IN REAL LIFE
impl<T: AsyncRead + Unpin> LineReader<T> {
    pub fn new(reader: T) -> LineReader<T> {
        Self {
            reader,
            buffer: Vec::with_capacity(4096),
        }
    }

    /// returns a new line
    pub async fn next_line(&mut self) -> Result<Option<String>, io::Error> {
        let mut buf = [0u8; 4096];
        loop {
            // search in our internal buffer for strings, if no string is found read more data!
            for i in 0..self.buffer.len() {
                if self.buffer[i] == '\n' as u8 {
                    // found a string!
                    let ret = String::from_utf8(self.buffer[0..i].to_vec())
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                    self.buffer = self.buffer[(i + 1)..].to_vec();

                    return Ok(Some(ret));
                }
            }

            let read = self.reader.read(&mut buf).await?;
            if read == 0 {
                return Ok(None);
            }
            self.buffer.extend_from_slice(&buf[0..read]);
        }
    }
}
