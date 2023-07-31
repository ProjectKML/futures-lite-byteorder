use std::{future::Future, io::ErrorKind, mem, pin::Pin, task::{Context, Poll}};
use byteorder::ByteOrder;

use futures_lite::{io, ready, AsyncReadExt, AsyncWrite, AsyncWriteExt};

macro_rules! write_future_8 {
    ($future: ident, $ty: ty) => {
        pub struct $future<'a, W: Unpin + ?Sized> {
            writer: &'a mut W,
            buf: [u8; 1],
        }

        impl<W: AsyncWrite + Unpin + ?Sized> Future for $future<'_, W> {
            type Output = io::Result<()>;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
                let Self { writer, buf } = &mut *self;

                let n = ready!(Pin::new(&mut **writer).poll_write(cx, buf))?;

                if n == 0 {
                    return Poll::Ready(Err(ErrorKind::WriteZero.into()))
                }

                Poll::Ready(Ok(()))
            }
        }
    };
}

macro_rules! write_future {
    ($future: ident, $ty: ty) => {
        pub struct $future<'a, W: Unpin + ?Sized> {
            writer: &'a mut W,
            buf: [u8; mem::size_of::<$ty>()],
            n: usize
        }

        impl<W: AsyncWrite + Unpin + ?Sized> Future for $future<'_, W> {
            type Output = io::Result<()>;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
                let Self { writer, buf, n } = &mut *self;

                while *n < buf.len() {
                    *n = ready!(Pin::new(&mut **writer).poll_write(cx, &buf[*n..]))?;

                    if *n == 0 {
                        return Poll::Ready(Err(ErrorKind::WriteZero.into()));
                    }
                }

                Poll::Ready(Ok(()))
            }
        }
    }
}

macro_rules! write_impl {
    ($future: ident, $name: ident, $ty: ty) => {
        fn $name<T: ByteOrder>(&mut self, value: $ty) -> $future<Self>
        where
            Self: Unpin
        {
            let mut future = $future {
                writer: self,
                buf: [0; mem::size_of::<$ty>()],
                n: 0
            };
            T::$name(&mut future.buf, value);

            future
        }
    }
}

write_future_8!(WriteU8Future, u8);
write_future_8!(WriteI8Future, i8);

write_future!(WriteU16Future, u16);
write_future!(WriteI16Future, i16);
write_future!(WriteU32Future, u32);
write_future!(WriteI32Future, i32);
write_future!(WriteU64Future, u64);
write_future!(WriteI64Future, i64);

pub trait AsyncByteOrderWrite: AsyncWriteExt {
    fn write_u8(&mut self, value: u8) -> WriteU8Future<Self>
    where
        Self: Unpin,
    {
        WriteU8Future {
            writer: self,
            buf: [value],
        }
    }

    fn write_i8(&mut self, value: i8) -> WriteI8Future<Self>
    where
        Self: Unpin,
    {
        WriteI8Future {
            writer: self,
            buf: [value as _]
        }
    }

    write_impl!(WriteU16Future, write_u16, u16);
    write_impl!(WriteI16Future, write_i16, i16);
    write_impl!(WriteU32Future, write_u32, u32);
    write_impl!(WriteI32Future, write_i32, i32);
    write_impl!(WriteU64Future, write_u64, u64);
    write_impl!(WriteI64Future, write_i64, i64);
}

pub struct ReadU8Future {

}

macro_rules! read_future_8 {

}

macro_rules! read_future {

}

macro_rules! read_impl {

}

pub trait AsyncByteOrderReader: AsyncReadExt {

}
