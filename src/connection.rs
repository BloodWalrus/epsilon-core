use std::{
    io::{Read, Write},
    marker::PhantomData,
    net::{SocketAddr, TcpStream, UdpSocket},
};

enum Protocol {
    Tcp(TcpStream),
    Udp(UdpSocket),
}

pub struct Connection<const N: usize, T> {
    stream: Protocol,
    _marker: PhantomData<T>,
}

impl<const N: usize, T> Connection<N, T> {
    pub fn new_tcp(stream: TcpStream) -> Self {
        Self {
            stream: Protocol::Tcp(stream),
            _marker: PhantomData,
        }
    }

    pub fn send(&mut self, frame: [T; N]) -> std::io::Result<()> {
        match &mut self.stream {
            Protocol::Tcp(stream) => {
                // cast &[T] into &[u8]
                let tmp = unsafe {
                    std::slice::from_raw_parts(
                        frame.as_ptr() as *const u8,
                        std::mem::size_of::<[T; N]>(),
                    )
                };
                stream.write_all(tmp)?;
                Ok(())
            }
            Protocol::Udp(_) => todo!(),
        }
    }

    pub fn recv(&mut self) -> std::io::Result<Box<[T; N]>> {
        match &mut self.stream {
            Protocol::Tcp(stream) => {
                // create empty boxed [T; N]
                let tmp = Box::new(unsafe { std::mem::zeroed::<[T; N]>() });

                // cast &[T] into &[u8] and pass it to read_exact for it to read into
                stream.read_exact(unsafe {
                    std::slice::from_raw_parts_mut(
                        tmp.as_ptr() as *mut u8,
                        std::mem::size_of::<[T; N]>(),
                    )
                })?;

                Ok(tmp)
            }
            Protocol::Udp(_) => todo!(),
        }
    }
}
