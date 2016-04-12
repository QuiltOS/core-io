#![no_std]

extern crate void;

use core::fmt;

use void::{unreachable, Void};

pub mod slice;
pub mod cursor;
pub mod wrapper;

#[macro_export]
macro_rules! try {
	($expr:expr) => (match $expr {
		::core::result::Result::Ok(val) => val,
		::core::result::Result::Err(err) => {
			return ::core::result::Result::Err(::core::convert::From::from(err))
		}
	})
}

#[derive(Copy, Clone, Debug)]
pub struct EndOfFile;

impl From<Void> for EndOfFile {
	fn from(v: Void) -> EndOfFile {
		unreachable(v)
	}
}

#[derive(Copy, Clone, Debug)]
pub struct OutOfBounds;

impl From<Void> for OutOfBounds {
	fn from(v: Void) -> OutOfBounds {
		unreachable(v)
	}
}

pub trait Read {
	type Err;

	fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Err>;

	fn read_all<E>(&mut self, mut buf: &mut [u8]) -> Result<(), E>
		where E: From<Self::Err> + From<EndOfFile>
	{
		while buf.len() > 0 {
			match try!(self.read(&mut buf)) {
				0 => return Err(E::from(EndOfFile)),
				n => {
					let tmp = buf;
					buf = &mut tmp[n..]
				}
			}
		}
		Ok(())
	}
}

pub trait Write {
	type Err;

	fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Err>;

	fn write_all<E>(&mut self, mut buf: &[u8]) -> Result<(), E>
		where E: From<Self::Err> + From<EndOfFile>
	{
		while buf.len() > 0 {
			match try!(self.write(buf)) {
				0 => return Err(E::from(EndOfFile)),
				n => buf = &buf[n..]
			}
		}
		Ok(())
	}

	fn write_fmt<E>(&mut self, fmt: fmt::Arguments) -> Result<(), E>
		where E: From<Self::Err> + From<EndOfFile>
	{
		// Create a shim which translates a Write to a fmt::Write and saves
		// off I/O errors. instead of discarding them
		struct Adaptor<'a, T: ?Sized + 'a, E> {
			inner: &'a mut T,
			result: Result<(), E>,
		}

		impl<'a, T: ?Sized, F> fmt::Write for Adaptor<'a, T, F>
			where T: Write,
			      F: From<EndOfFile> + From<T::Err>
		{
			fn write_str(&mut self, s: &str) -> fmt::Result {
				match self.inner.write_all(s.as_bytes()) {
					Ok(()) => Ok(()),
					Err(e) => {
						self.result = Err(e);
						Err(fmt::Error)
					}
				}
			}
		}

		let mut output = Adaptor { inner: self, result: Ok(()) };
		let _ = fmt::write(&mut output, fmt);
		output.result
	}
}

pub enum SeekFrom {
	Start(u64),
	End(i64),
	Current(i64),
}

pub trait Seek {
	type Err;

	fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Err>;

	fn tell(&mut self) -> Result<u64, Self::Err> {
		self.seek(SeekFrom::Current(0))
	}
}
