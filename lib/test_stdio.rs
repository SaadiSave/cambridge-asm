use std::string::FromUtf8Error;
use std::{
    io,
    sync::{Arc, RwLock},
};

#[derive(Clone)]
#[repr(transparent)]
pub struct TestStdio(Arc<RwLock<Vec<u8>>>);

impl TestStdio {
    pub fn new(s: impl std::ops::Deref<Target = [u8]>) -> Self {
        Self(Arc::new(RwLock::new(s.to_vec())))
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.read().unwrap().clone()
    }

    pub fn try_to_string(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.to_vec())
    }
}

impl io::Write for TestStdio {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write().unwrap().extend(buf);

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Read for TestStdio {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        use io::{Error, ErrorKind::UnexpectedEof, Write};

        let written = {
            let r_lock = self.0.read().unwrap();

            if r_lock.is_empty() {
                return Err(Error::new(UnexpectedEof, "Input is empty"));
            }

            buf.write(&r_lock)?
        };

        self.0.write().unwrap().drain(0..written);
        Ok(written)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        use io::{
            Error,
            ErrorKind::{Other, UnexpectedEof},
        };
        use std::cmp::Ordering;

        let bytes_written = self.read(buf)?;

        match bytes_written.cmp(&buf.len()) {
            Ordering::Less => Err(Error::new(UnexpectedEof, "Insufficient input")),
            Ordering::Equal => Ok(()),
            Ordering::Greater => Err(Error::new(Other, "I don't know what happened")),
        }
    }
}
