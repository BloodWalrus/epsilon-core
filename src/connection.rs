use std::{
    io::{self, Read, Write},
    marker::PhantomData,
    mem::{size_of, zeroed},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

#[repr(u8)]
pub enum CtrlSignal {
    Start,
    Stop,
    Reset,
}

pub struct Incomming<'a, T> {
    listener: &'a TcpListener,
    _marker: PhantomData<T>,
}

impl<'a, T> Iterator for Incomming<'a, T> {
    type Item = Stream<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.listener.accept().map(|(stream, _)| stream) {
            Ok(stream) => Some(Stream::from_tcpstream(stream)),
            Err(_) => None,
        }
    }
}

pub struct Listener<T> {
    listener: TcpListener,
    _marker: PhantomData<T>,
}

impl<T> Listener<T> {
    // add results
    pub fn listen<P: ToSocketAddrs>(socket: P) -> io::Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(socket)?,
            _marker: PhantomData,
        })
    }

    pub fn incomming(&self) -> Incomming<'_, T> {
        Incomming {
            listener: &self.listener,
            _marker: PhantomData,
        }
    }
}

pub struct Stream<T> {
    stream: TcpStream,
    _marker: PhantomData<T>,
}

impl<T> Stream<T> {
    // add results
    pub fn connect<P: ToSocketAddrs>(socket: P) -> io::Result<Self> {
        Ok(Self::from_tcpstream(TcpStream::connect(socket)?))
    }

    pub(crate) fn from_tcpstream(stream: TcpStream) -> Self {
        Self {
            stream,
            _marker: PhantomData,
        }
    }

    pub fn send(&mut self, item: &T) -> io::Result<()> {
        self.stream.set_nonblocking(false)?;
        self.stream.write_all(unsafe { as_bytes(item) })?;

        Ok(())
    }

    pub fn recv(&mut self) -> io::Result<T> {
        self.stream.set_nonblocking(false)?;
        let mut tmp: T = unsafe { zeroed() };
        self.stream.read_exact(unsafe { as_bytes_mut(&mut tmp) })?;

        Ok(tmp)
    }

    pub fn try_recv(&mut self) -> io::Result<Option<T>> {
        self.stream.set_nonblocking(true)?;
        let mut tmp: T = unsafe { zeroed() };
        match self.stream.read_exact(unsafe { as_bytes_mut(&mut tmp) }) {
            Ok(_) => Ok(Some(tmp)),
            Err(err) => match err.kind() {
                io::ErrorKind::WouldBlock => Ok(None),
                _ => Err(err),
            },
        }
    }
}

unsafe fn as_bytes<'a, T: Sized>(value: &'a T) -> &'a [u8] {
    std::slice::from_raw_parts(value as *const T as *const u8, size_of::<T>())
}

unsafe fn as_bytes_mut<'a, T: Sized>(value: &'a mut T) -> &'a mut [u8] {
    std::slice::from_raw_parts_mut(value as *mut T as *mut u8, size_of::<T>())
}
