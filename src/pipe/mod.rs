//! A type which enables communication between threads by providing an
//! implementation of [`Read`], [`BufRead`], and [`Write`] on a shared `Vec`.
//! The result of calling any function declared by [`Read`] or [`BufRead`] while
//! a function declared by [`BufRead`] is blocking in another thread is
//! dependent on whether the new call can be satisfied immediately. If it can,
//! the pipe will fulfill it immediately. Otherwise, the new call will also
//! block and the order in which the two calls get satisfied is partially
//! dependent on which thread gets notified first.
//!
//! [`BufRead`]: /std/io/trait.BufRead.html
//! [`Read`]: /std/io/trait.Read.html
//! [`Write`]: /std/io/trait.Write.html

use std::{
    io::{self, BufRead, Read, Write},
    sync::{atomic::{AtomicU32, Ordering}, Arc, Condvar, Mutex},
};

/// Create a pipe.
pub fn mk_pipe() -> (PipeRead, PipeWrite) {
    let ret = Pipe::default();
    (PipeRead::new(ret.clone()), PipeWrite::new(ret))
}

fn index_of<T>(value: T, buf: &[T]) -> Option<usize>
where
  T: PartialEq,
{
    buf.iter().position(|v| &value == v)
}

#[derive(Clone, Default)]
struct Pipe {
    bytes: Arc<(Mutex<Vec<u8>>, Condvar)>,
    readers: Arc<AtomicU32>,
    writers: Arc<AtomicU32>,
}

impl Pipe {
    fn has_read_end(&self) -> bool {
        self.readers.load(Ordering::SeqCst) > 0
    }

    fn has_write_end(&self) -> bool {
        self.writers.load(Ordering::SeqCst) > 0
    }
}

impl Read for Pipe {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let (bytes_lock, condvar) = &*self.bytes;
        let bytes = bytes_lock.lock().unwrap();
        // Wait for data so that a wrapper around a `Read` that expects "no
        // available data" to mean "EOF reached" won't decide that the pipe is
        // dead.
        let mut bytes = condvar.wait_while(bytes, |bytes| bytes.is_empty()).unwrap();
        let len = usize::min(buf.len(), bytes.len());
        let mut bytes = bytes.drain(..len);
        for byte in buf.iter_mut() {
            *byte = bytes.next().unwrap();
        }
        // Inform any other threads that may be waiting on access to the pipe
        // through the `Condvar` that it is available.
        condvar.notify_one();
        Ok(len)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let (bytes_lock, condvar) = &*self.bytes;
        let bytes = bytes_lock.lock().unwrap();
        let condition = |bytes: &mut Vec<_>| {
            self.has_write_end() && bytes.len() < std::usize::MAX - buf.len()
        };
        let mut bytes = condvar.wait_while(bytes, condition).unwrap();
        // Either the pipe can no longer receive data or the pipe contains
        // enough data that `buf` can be filled completely.
        let len = usize::min(std::usize::MAX - buf.len(), bytes.len());
        let bytes = bytes.drain(..len);
        buf.extend(bytes);
        condvar.notify_one();
        Ok(len)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        let (bytes_lock, condvar) = &*self.bytes;
        let mut bytes = bytes_lock.lock().unwrap();
        let condition = |bytes: &mut Vec<_>| {
            self.has_write_end() && bytes.len() < std::usize::MAX - buf.len()
        };
        bytes = condvar.wait_while(bytes, condition).unwrap();
        let len = usize::min(std::usize::MAX - buf.len(), bytes.len());
        let s = std::str::from_utf8(&bytes[..len])
            .map_err(|e| {
                condvar.notify_one();
                io::Error::new(io::ErrorKind::InvalidData, e)
            })?;
        buf.push_str(s);
        bytes.drain(..len);
        condvar.notify_one();
        Ok(len)
    }

    fn read_exact(&mut self, mut buf: &mut [u8]) -> io::Result<()> {
        let (bytes_lock, condvar) = &*self.bytes;
        let mut bytes = bytes_lock.lock().unwrap();
        let len = buf.len();
        let condition = |bytes: &mut Vec<_>| bytes.len() < len;
        while condition(&mut bytes) {
            // Can't read exactly `buf.len()` bytes from `bytes`, but if
            // there's still a live writer, then more bytes may come in the
            // future.
            if !self.has_write_end() {
                // There is no longer a write end to this pipe and no way
                // to create a new one is exposed, so exactly `buf.len()`
                // bytes can never be read from this pipe.
                condvar.notify_one();
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Pipe: no writers"));
            }
            bytes = condvar.wait(bytes).unwrap();
        }
        buf.write_all(&bytes[..len])?;
        bytes.drain(..len);
        condvar.notify_one();
        Ok(())
    }
}

impl BufRead for Pipe {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        unimplemented!("<Pipe as BufRead>::fill_buf")
    }

    fn consume(&mut self, amt: usize) {
        self.bytes.0.lock().unwrap().drain(..amt);
    }

    fn read_until(
        &mut self,
        byte: u8,
        buf: &mut Vec<u8>
    ) -> io::Result<usize> {
        if buf.len() == std::usize::MAX {
            return Ok(0);
        }
        let (bytes_lock, condvar) = &*self.bytes;
        let mut bytes = bytes_lock.lock().unwrap();
        let max_read = std::usize::MAX - buf.len();
        let mut next_check = 0;
        let condition = |bytes: &mut Vec<_>| {
            if self.has_write_end() {
                match index_of(byte, &bytes[next_check..]) {
                    Some(idx) => {
                        next_check = usize::min(idx, max_read - 1);
                        false
                    }
                    None => {
                        next_check = usize::min(bytes.len(), max_read - 1);
                        next_check == max_read - 1
                    }
                }
            } else {
                false
            }
        };
        bytes = condvar.wait_while(bytes, condition).unwrap();
        if self.has_write_end() || bytes.len() > 0 && bytes[next_check] == byte {
            // Either `bytes[next_check]` is `byte` or `next_check` is
            // `max_read`.
            let ret = if bytes[next_check] == byte {
                buf.reserve(next_check + 1);
                buf.extend(bytes.drain(..=next_check));
                Ok(next_check + 1)
            } else {
                buf.reserve(max_read);
                buf.extend(bytes.drain(..max_read));
                Ok(max_read)
            };
            condvar.notify_one();
            ret
        } else {
            // There's never going to be any more data, so drain as much data
            // as possible into `buf`.
            let len = usize::min(max_read, bytes.len());
            buf.extend(bytes.drain(..len));
            condvar.notify_one();
            Ok(len)
        }
    }

    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        let (len, s) = {
            let init = buf.len();
            let mut buf = buf.to_string().into_bytes();
            let len = self.read_until('\n' as u8, &mut buf)?;
            buf.drain(..init);
            let s = String::from_utf8(buf).map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, e)
            })?;
            (len, s)
        };
        buf.push_str(&s);
        Ok(len)
    }
}

impl Write for Pipe {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let (bytes_lock, condvar) = &*self.bytes;
        let mut bytes = bytes_lock.lock().unwrap();
        let condition = |bytes: &mut Vec<_>| bytes.len() >= std::usize::MAX;
        bytes = condvar.wait_while(bytes, condition).unwrap();
        if !self.has_read_end() {
            condvar.notify_one();
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "Pipe: no readers"))
        } else {
            let len = usize::min(std::usize::MAX - bytes.len(), buf.len());
            bytes.reserve(len);
            bytes.extend_from_slice(buf);
            condvar.notify_one();
            Ok(len)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        let (bytes_lock, condvar) = &*self.bytes;
        let mut bytes = bytes_lock.lock().unwrap();
        let condition = |bytes: &mut Vec<_>| bytes.len() > std::usize::MAX - buf.len();
        bytes = condvar.wait_while(bytes, condition).unwrap();
        if !self.has_read_end() {
            condvar.notify_one();
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "Pipe: no readers"))
        } else {
            condvar.notify_one();
            bytes.write_all(buf)
        }
    }
}

/// The read end of a pipe.
pub struct PipeRead {
    inner: Pipe,
}

impl PipeRead {
    fn new(inner: Pipe) -> Self {
        inner.readers.fetch_add(1, Ordering::SeqCst);
        Self { inner }
    }
}

impl Clone for PipeRead {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone())
    }
}

impl Drop for PipeRead {
    fn drop(&mut self) {
        self.inner.readers.fetch_sub(1, Ordering::SeqCst);
    }
}

impl Read for PipeRead {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.inner.read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.inner.read_exact(buf)
    }
}

impl BufRead for PipeRead {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt)
    }

    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner.read_until(byte, buf)
    }

    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        self.inner.read_line(buf)
    }
}

/// The write end of a pipe.
pub struct PipeWrite {
    inner: Pipe,
}

impl PipeWrite {
    fn new(inner: Pipe) -> Self {
        inner.writers.fetch_add(1, Ordering::SeqCst);
        Self { inner }
    }
}

impl Clone for PipeWrite {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone())
    }
}

impl Drop for PipeWrite {
    fn drop(&mut self) {
        self.inner.writers.fetch_sub(1, Ordering::SeqCst);
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
        let (mut a_to_b_read, mut a_to_b_write) = mk_pipe();
        let (mut b_to_a_read, mut b_to_a_write) = mk_pipe();
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
        thread_a
            .join()
            .expect("Failed to join thread_test::thread_a");
        thread_b
            .join()
            .expect("Failed to join thread_test::thread_b");
    }

    #[test]
    fn test_close_read() {
        let (_, mut a_to_b_write) = mk_pipe();
        assert_eq!(
            write!(a_to_b_write, "Hello")
                .expect_err("Write to pipe with no readers succeeded")
                .kind(),
            io::ErrorKind::BrokenPipe,
        );
    }

    #[test]
    fn test_close_write() {
        let (mut a_to_b_read, mut a_to_b_write) = mk_pipe();
        let _ = write!(a_to_b_write, "Hi").unwrap();
        std::mem::drop(a_to_b_write);
        let mut buf = [0u8; 5];
        assert_eq!(
            a_to_b_read
                .read_exact(&mut buf)
                .expect_err(
                    format!(
                        "Constant-size read from pipe with insufficient {}",
                        "data succeeded",
                    )
                    .as_ref()
                )
                .kind(),
            io::ErrorKind::UnexpectedEof,
        );
    }
}
