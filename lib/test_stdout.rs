use std::{io::Write, rc::Rc, sync::RwLock};

#[derive(Clone)]
#[repr(transparent)]
pub struct TestStdout(Rc<RwLock<Vec<u8>>>);

impl TestStdout {
    pub fn new(s: impl std::ops::Deref<Target = [u8]>) -> Self {
        Self(Rc::new(RwLock::new(s.to_vec())))
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.read().unwrap().clone()
    }
}

impl Write for TestStdout {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write().unwrap().extend(buf);

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
