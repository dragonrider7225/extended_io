use std::{
    io::{self, Read, Write},
    sync::{atomic::{AtomicU32, Ordering}, Arc, Mutex},
    thread,
};

/// A type to asynchronously transfer data between threads.
#[derive(Clone, Default)]
pub struct Pipe {
    bytes: Arc<Mutex<Vec<u8>>>,
    readers: Arc<AtomicU32>,
    writers: Arc<AtomicU32>,
}

impl Pipe {
    /// Create a pipe.
    pub fn new() -> (impl Read, impl Write) {
        let ret = Self::default();
        (PipeRead::new(ret.clone()), PipeWrite::new(ret))
    }
}

impl Read for Pipe {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut bytes = self.bytes.lock().unwrap();
        let len = usize::min(buf.len(), bytes.len());
        let mut bytes = bytes.drain(..len);
        for i in 0..len {
            buf[i] = bytes.next().unwrap();
        }
        Ok(len)
    }

    fn read_exact(&mut self, mut buf: &mut [u8]) -> io::Result<()> {
        loop {
            let mut bytes = self.bytes.lock().unwrap();
            let len = buf.len();
            if bytes.len() < len {
                if self.writers.load(Ordering::SeqCst) == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Pipe: no writers",
                    ));
                } else {
                    std::mem::drop(bytes);
                    thread::yield_now();
                }
            } else {
                buf.write_all(&bytes[..len])?;
                bytes.drain(..len);
                return Ok(());
            }
        }
    }
}

impl Write for Pipe {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut bytes = self.bytes.lock().unwrap();
        let len = usize::min(std::usize::MAX - bytes.len(), buf.len());
        bytes.reserve(len);
        bytes.extend_from_slice(buf);
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        if self.readers.load(Ordering::SeqCst) == 0 {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "Pipe: no readers"))
        } else {
            self.bytes.lock().unwrap().write_all(buf)
        }
    }
}

#[derive(Clone)]
struct PipeRead {
    inner: Pipe,
}

impl PipeRead {
    fn new(inner: Pipe) -> Self {
        inner.readers.fetch_add(1, Ordering::SeqCst);
        Self { inner }
    }
}

impl Read for PipeRead {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.inner.read_exact(buf)
    }
}

#[derive(Clone)]
struct PipeWrite {
    inner: Pipe,
}

impl PipeWrite {
    fn new(inner: Pipe) -> Self {
        inner.writers.fetch_add(1, Ordering::SeqCst);
        Self { inner }
    }
}

impl Write for PipeWrite {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inner.write_all(buf)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::thread;

    #[test]
    fn test_pipe() {
        let (mut a_to_b_read, mut a_to_b_write) = Pipe::new();
        let (mut b_to_a_read, mut b_to_a_write) = Pipe::new();
        let thread_a = thread::Builder::new()
            .name("thread_test::thread_a".to_string())
            .spawn(move || {
                write!(a_to_b_write, "Hello").unwrap();
                let mut buf = [0u8; 5];
                b_to_a_read.read_exact(&mut buf).unwrap();
                assert_eq!(b"World", &buf);
            })
            .expect("Failed to create thread_test::thread_a");
        let thread_b = thread::Builder::new()
            .name("pipe_test::thread_b".to_string())
            .spawn(move || {
                let mut buf = [0u8; 5];
                a_to_b_read.read_exact(&mut buf).unwrap();
                assert_eq!(b"Hello", &buf);
                write!(b_to_a_write, "World").unwrap();
            })
            .expect("Failed to create thread_test::thread_b");
        thread_a.join().expect("Failed to join thread_test::thread_a");
        thread_b.join().expect("Failed to join thread_test::thread_b");
    }
}
