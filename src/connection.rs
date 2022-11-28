use std::{
    io,
    mem::{size_of, transmute},
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
};

#[repr(u8)]
pub enum CtrlSignal {
    Start,
    Stop,
    Reset,
}

#[repr(C)]
pub struct DataPacket {
    pub index: u64,
    pub data: [u8; 120],
}

pub struct Ctrl {
    socket: UdpSocket,
}

impl Ctrl {
    pub fn new(socket: SocketAddr, target: SocketAddr) -> io::Result<Self> {
        let tmp = UdpSocket::bind(socket)?;
        tmp.connect(target)?;
        Ok(Self { socket: tmp })
    }

    pub fn try_recv(&mut self) -> io::Result<Option<CtrlSignal>> {
        self.socket.set_nonblocking(true)?;
        let mut tmp = [0u8; 1];

        let result = self.socket.recv(&mut tmp);
        match result {
            Ok(_) => Ok(Some(unsafe { transmute(tmp) })),
            Err(err) => match err.kind() {
                io::ErrorKind::WouldBlock => Ok(None),
                _ => Err(err),
            },
        }
    }

    pub fn send(&mut self, signal: CtrlSignal) -> io::Result<()> {
        self.socket.set_nonblocking(false)?;
        let tmp = unsafe { transmute::<CtrlSignal, [u8; 1]>(signal) };
        self.socket.send(&tmp)?;
        Ok(())
    }
}

pub struct DataBus {
    socket: UdpSocket,
}

impl DataBus {
    pub fn new(socket: SocketAddr, target: SocketAddr) -> io::Result<Self> {
        let tmp = UdpSocket::bind(socket)?;
        tmp.connect(target)?;
        tmp.set_nonblocking(false)?;
        Ok(Self { socket: tmp })
    }

    pub fn recv(&mut self) -> io::Result<DataPacket> {
        let mut tmp = [0u8; size_of::<DataPacket>()];
        self.socket.recv(&mut tmp)?;
        Ok(unsafe { transmute(tmp) })
    }

    pub fn send(&mut self, signal: DataPacket) -> io::Result<()> {
        let tmp = unsafe { transmute::<DataPacket, [u8; size_of::<DataPacket>()]>(signal) };
        self.socket.send(&tmp)?;
        Ok(())
    }
}
