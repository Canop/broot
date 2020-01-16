//! This module deals is where is defined whether broot
//! writes on stdout, on stderr or elsewhere. It also provides helper
//! structs for io.
use std::{
    fmt,
    io::{self, Write},
};

/// the type used by all GUI writing functions
pub type W = std::io::Stderr;

/// return the writer used by the application
pub fn writer() -> W {
    std::io::stderr()
}

/// RAII wrapper for writer to control state transitions.
pub struct WriteCleanup<W, F, E>
where
    W: Write,
    F: Fn(&mut W) -> Result<(), E>,
    E: fmt::Display,
{
    writer: W,
    cleanup: F,
}

impl<W, F, E> WriteCleanup<W, F, E>
where
    W: Write,
    F: Fn(&mut W) -> Result<(), E>,
    E: fmt::Display,
{
    #[inline]
    pub fn new(writer: W, cleanup: F) -> Self {
        WriteCleanup { writer, cleanup }
    }

    #[inline]
    pub fn build<E2, F2: Fn(&mut W) -> Result<(), E2>>(
        mut writer: W,
        build: F2,
        cleanup: F,
    ) -> Result<Self, E2> {
        build(&mut writer)?;
        Ok(Self::new(writer, cleanup))
    }
}

impl<W, F, E> Drop for WriteCleanup<W, F, E>
where
    W: Write,
    F: Fn(&mut W) -> Result<(), E>,
    E: fmt::Display,
{
    fn drop(&mut self) {
        if let Err(err) = (self.cleanup)(&mut self.writer) {
            warn!("Error cleaning up terminal: {}", err);
        }
    }
}

impl<W, F, E> fmt::Debug for WriteCleanup<W, F, E>
where
    W: Write + fmt::Debug,
    F: Fn(&mut W) -> Result<(), E>,
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WriteCleanup")
            .field("writer", &self.writer)
            .field("cleanup", &"<closure>")
            .finish()
    }
}

impl<W, F, E> Write for WriteCleanup<W, F, E>
where
    W: Write,
    F: Fn(&mut W) -> Result<(), E>,
    E: fmt::Display,
{
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.writer.write_vectored(bufs)
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.writer.write_all(buf)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        self.writer.write_fmt(fmt)
    }
}

#[test]
fn test_write_cleanup() -> io::Result<()> {
    use std::str;

    let mut buffer: Vec<u8> = Vec::new();

    {
        let writer = WriteCleanup::build(
            &mut buffer,
            |writer| write!(writer, "abc "),
            |writer| write!(writer, " xyz"),
        )?;

        let mut writer = WriteCleanup::build(
            writer,
            |writer| write!(writer, "123 "),
            |writer| write!(writer, " 789"),
        )?;

        write!(&mut writer, "Hello, World!")?;
    }

    let result = str::from_utf8(&buffer).unwrap();

    assert_eq!(result, "abc 123 Hello, World! 789 xyz");

    Ok(())
}
