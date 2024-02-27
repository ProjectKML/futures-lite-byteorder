use std::{
    future::Future,
    io::ErrorKind,
    marker::PhantomData,
    mem,
    pin::Pin,
    task::{Context, Poll},
};

use byteorder::ByteOrder;
use futures_lite::{io, ready, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

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
            n: usize,
        }

        impl<W: AsyncWrite + Unpin + ?Sized> Future for $future<'_, W> {
            type Output = io::Result<()>;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
                let Self { writer, buf, n } = &mut *self;

                while *n < buf.len() {
                    *n = ready!(Pin::new(&mut **writer).poll_write(cx, &buf[*n..]))?;

                    if *n == 0 {
                        return Poll::Ready(Err(ErrorKind::WriteZero.into()))
                    }
                }

                Poll::Ready(Ok(()))
            }
        }
    };
}

macro_rules! write_impl_8 {
    ($future: ident, $name: ident, $ty: ty) => {
        fn $name(&mut self, value: $ty) -> $future<'_, Self>
        where
            Self: Unpin,
        {
            $future {
                writer: self,
                buf: [value as u8],
            }
        }
    };
}

macro_rules! write_impl {
    ($future: ident, $name: ident, $ty: ty) => {
        fn $name<T: ByteOrder>(&mut self, value: $ty) -> $future<'_, Self>
        where
            Self: Unpin,
        {
            let mut future = $future {
                writer: self,
                buf: [0; mem::size_of::<$ty>()],
                n: 0,
            };
            T::$name(&mut future.buf, value);

            future
        }
    };
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
    write_impl_8!(WriteU8Future, write_u8, u8);
    write_impl_8!(WriteI8Future, write_i8, i8);

    write_impl!(WriteU16Future, write_u16, u16);
    write_impl!(WriteI16Future, write_i16, i16);
    write_impl!(WriteU32Future, write_u32, u32);
    write_impl!(WriteI32Future, write_i32, i32);
    write_impl!(WriteU64Future, write_u64, u64);
    write_impl!(WriteI64Future, write_i64, i64);
}

impl<W: AsyncWriteExt> AsyncByteOrderWrite for W {}

macro_rules! read_future_8 {
    ($future: ident, $ty: ty) => {
        pub struct $future<'a, R: Unpin + ?Sized> {
            reader: &'a mut R,
            buf: [u8; 1],
        }

        impl<R: AsyncRead + Unpin + ?Sized> Future for $future<'_, R> {
            type Output = io::Result<$ty>;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
                let Self { reader, buf } = &mut *self;

                let n = ready!(Pin::new(&mut *reader).poll_read(cx, buf))?;

                if n == 0 {
                    return Poll::Ready(Err(ErrorKind::UnexpectedEof.into()))
                }

                Poll::Ready(Ok((buf[0] as $ty)))
            }
        }
    };
}

macro_rules! read_future {
    ($future: ident, $name: ident, $ty: ty) => {
        pub struct $future<'a, R: Unpin + ?Sized, T: ByteOrder> {
            reader: &'a mut R,
            buf: [u8; mem::size_of::<$ty>()],
            n: usize,
            phantom: PhantomData<&'a T>,
        }

        impl<R: AsyncRead + Unpin + ?Sized, T: ByteOrder> Future for $future<'_, R, T> {
            type Output = io::Result<$ty>;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
                let Self { reader, buf, n, .. } = &mut *self;

                while *n < buf.len() {
                    *n = ready!(Pin::new(&mut **reader).poll_read(cx, &mut buf[*n..]))?;

                    if *n == 0 {
                        return Poll::Ready(Err(ErrorKind::UnexpectedEof.into()))
                    }
                }

                Poll::Ready(Ok(T::$name(buf)))
            }
        }
    };
}

macro_rules! read_impl_8 {
    ($future: ident, $name: ident, $ty: ty) => {
        fn $name(&mut self) -> $future<'_, Self>
        where
            Self: Unpin,
        {
            $future {
                reader: self,
                buf: Default::default(),
            }
        }
    };
}

macro_rules! read_impl {
    ($future: ident, $name: ident, $ty: ty) => {
        fn $name<T: ByteOrder>(&mut self) -> $future<'_, Self, T>
        where
            Self: Unpin,
        {
            $future {
                reader: self,
                buf: Default::default(),
                n: 0,
                phantom: PhantomData,
            }
        }
    };
}

read_future_8!(ReadU8Future, u8);
read_future_8!(ReadI8Future, i8);

read_future!(ReadU16Future, read_u16, u16);
read_future!(ReadI16Future, read_i16, i16);
read_future!(ReadU32Future, read_u32, u32);
read_future!(ReadI32Future, read_i32, i32);
read_future!(ReadU64Future, read_u64, u64);
read_future!(ReadI64Future, read_i64, i64);

pub trait AsyncByteOrderRead: AsyncReadExt {
    read_impl_8!(ReadU8Future, read_u8, u8);
    read_impl_8!(ReadI8Future, read_i8, i8);

    read_impl!(ReadU16Future, read_u16, u16);
    read_impl!(ReadI16Future, read_i16, i16);
    read_impl!(ReadU32Future, read_u32, u32);
    read_impl!(ReadI32Future, read_i32, i32);
    read_impl!(ReadU64Future, read_u64, u64);
    read_impl!(ReadI64Future, read_i64, i64);
}

impl<R: AsyncReadExt> AsyncByteOrderRead for R {}
