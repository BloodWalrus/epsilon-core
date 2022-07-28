use std::{
    io::{Read, Write},
    marker::PhantomData,
    net::TcpStream,
};

pub struct Connection<const N: usize, T> {
    stream: TcpStream,
    _marker: PhantomData<T>,
}

impl<const N: usize, T> Connection<N, T> {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _marker: PhantomData,
        }
    }

    pub fn send(&mut self, frame: [T; N]) -> std::io::Result<()> {
        // cast &[T] into &[u8]
        let tmp = unsafe {
            std::slice::from_raw_parts(frame.as_ptr() as *const u8, std::mem::size_of::<[T; N]>())
        };
        self.stream.write_all(tmp)?;
        Ok(())
    }

    pub fn recv(&mut self) -> std::io::Result<Box<[T; N]>> {
        // create empty boxed [T; N]
        let tmp = Box::new(unsafe { std::mem::zeroed::<[T; N]>() });

        // cast &[T] into &[u8] and pass it to read_exact for it to read into
        self.stream.read_exact(unsafe {
            std::slice::from_raw_parts_mut(tmp.as_ptr() as *mut u8, std::mem::size_of::<[T; N]>())
        })?;

        Ok(tmp)
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.stream.peer_addr()
    }
}
