use std::{
    error::Error,
    fmt::Display,
    io::{Read, Write},
    marker::PhantomData,
    mem::size_of,
    net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
};

use crate::EpsilonResult;

pub struct Streamer<T, const T_SIZE: usize> {
    listener: TcpListener,
    stream: Option<TcpStream>,
    _marker: PhantomData<T>,
}

impl<T: Copy, const T_SIZE: usize> Streamer<T, T_SIZE> {
    // add results
    pub fn listen<P: ToSocketAddrs>(socket: P) -> EpsilonResult<Self> {
        Ok(Self {
            listener: TcpListener::bind(socket)?,
            stream: None,
            _marker: PhantomData,
        })
    }

    /// connectes to the next valid incomming client
    /// this will block until the next client, and return Ok(()) if succesful
    pub fn next_client(&mut self) -> EpsilonResult<()> {
        let stream = loop {
            let (mut stream, socket) = self.listener.accept()?;
            eprintln!("checking transfer type signature of {:?}", socket);

            let mut transfer_signature = [0u8; size_of::<TransferTypeSignature>()];

            stream.read_exact(&mut transfer_signature)?;

            let converter = Converter {
                data: transfer_signature,
            };

            if self.connecting_transfer_type_signature_matches_local(unsafe { converter.value }) {
                break stream;
            }
        };

        self.stream = Some(stream);

        Ok(())
    }

    fn connecting_transfer_type_signature_matches_local(
        &self,
        transfer_type_signature: TransferTypeSignature,
    ) -> bool {
        self.generate_transfer_type_signature() == transfer_type_signature
    }

    pub fn send(&mut self, item: T) -> EpsilonResult<()> {
        if let Some(stream) = &mut self.stream {
            let converter: Converter<T, T_SIZE> = Converter { value: item };

            stream.write_all(unsafe { &converter.data })?;

            Ok(())
        } else {
            Err(Box::new(StreamerError::NoClientConnected))
        }
    }
}

pub struct Client<T, const T_SIZE: usize> {
    stream: TcpStream,
    _marker: PhantomData<T>,
}

impl<T: Copy, const T_SIZE: usize> Client<T, T_SIZE> {
    // add results
    pub fn connect<P: ToSocketAddrs>(socket: P) -> EpsilonResult<Self> {
        let mut _self = Self {
            stream: TcpStream::connect(socket)?,
            _marker: PhantomData,
        };

        const SIZE: usize = size_of::<TransferTypeSignature>();
        let converter: Converter<TransferTypeSignature, SIZE> = Converter {
            value: _self.generate_transfer_type_signature(),
        };

        _self.stream.write_all(unsafe { &converter.data })?;

        Ok(_self)
    }

    pub fn recv(&mut self) -> EpsilonResult<T> {
        let mut converter: Converter<T, T_SIZE> = Converter {
            data: [0u8; T_SIZE],
        };

        self.stream.read_exact(unsafe { &mut converter.data })?;

        Ok(unsafe { converter.value })
    }
}

trait GenerateTransferTypeSignature<T> {
    fn generate_transfer_type_signature(&self) -> TransferTypeSignature {
        TransferTypeSignature {
            size: size_of::<T>(),
            align: std::mem::align_of::<T>(),
        }
    }
}

impl<T, const T_SIZE: usize> GenerateTransferTypeSignature<T> for Streamer<T, T_SIZE> {}

impl<T, const T_SIZE: usize> GenerateTransferTypeSignature<T> for Client<T, T_SIZE> {}

pub union Converter<T: Copy, const SIZE: usize> {
    pub value: T,
    pub data: [u8; SIZE],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TransferTypeSignature {
    pub size: usize,
    pub align: usize,
}

///////////////////////////////////////////////////////////////////
// Errors

#[derive(Debug, Clone, Copy)]
pub enum StreamerError {
    NoClientConnected,
}

impl Display for StreamerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            StreamerError::NoClientConnected => "no client connected",
        })
    }
}

impl Error for StreamerError {}
