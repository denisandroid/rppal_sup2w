use embedded_hal::{
    delay::DelayUs,
    spi::{self, ErrorType, SpiBus, SpiDevice, Operation},
};
use embedded_hal_nb::spi::FullDuplex;
use std::io;

use super::{super::hal::Delay, Error, Spi};

impl ErrorType for Spi {
    type Error = Error;
}

impl spi::Error for Error {
    fn kind(&self) -> spi::ErrorKind {
        spi::ErrorKind::Other
    }
}

/// `SpiBus<u8>` trait implementation for `embedded-hal` v1.0.0.
impl SpiBus<u8> for Spi {
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        Spi::read(self, words)?;
        Ok(())
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        Spi::write(self, words)?;
        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        Spi::transfer(self, read, write)?;
        Ok(())
    }

    fn transfer_in_place(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let write_buffer = buffer.to_vec();
        self.transfer(buffer, &write_buffer)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// `Transfer<u8>` trait implementation for `embedded-hal` v0.2.7.
impl embedded_hal_0::blocking::spi::Transfer<u8> for Spi {
    type Error = Error;

    fn transfer<'a>(&mut self, buffer: &'a mut [u8]) -> Result<&'a [u8], Self::Error> {
        let write_buffer = buffer.to_vec();
        SpiBus::transfer(self, buffer, &write_buffer)?;
        Ok(buffer)
    }
}

/// `Write<u8>` trait implementation for `embedded-hal` v0.2.7.
impl embedded_hal_0::blocking::spi::Write<u8> for Spi {
    type Error = Error;

    fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        SpiBus::write(self, buffer)
    }
}

/// `FullDuplex<u8>` trait implementation for `embedded-hal` v1.0.0
impl FullDuplex<u8> for Spi {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        if let Some(last_read) = self.last_read.take() {
            Ok(last_read)
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn write(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        let mut read_buffer: [u8; 1] = [0];

        Spi::transfer(self, &mut read_buffer, &[byte])?;
        self.last_read = Some(read_buffer[0]);

        Ok(())
    }
}

/// `FullDuplex<u8>` trait implementation for `embedded-hal` v0.2.7.
impl embedded_hal_0::spi::FullDuplex<u8> for Spi {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        FullDuplex::read(self)
    }

    fn send(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        FullDuplex::write(self, byte)
    }
}

/// Simple implementation of [embedded_hal::spi::SpiDevice]
///
/// You only need this when using the `embedded_hal` Spi trait interface.
///
/// Slave-select is currently handled at the bus level.
/// This no-op device implementation can be used to satisfy the trait.
// TODO: The underlying crate::spi::Spi shall be split up to support proper slave-select handling here.
pub struct SimpleHalSpiDevice<B> {
    bus: B,
}

impl<B: SpiBus<u8>> SimpleHalSpiDevice<B> {
    pub fn new(bus: B) -> SimpleHalSpiDevice<B> {
        SimpleHalSpiDevice { bus }
    }
}

impl<B: SpiBus<u8>> SpiDevice<u8> for SimpleHalSpiDevice<B> {
    fn transaction(
        &mut self,
        operations: &mut [Operation<'_, u8>]
    ) -> Result<(), Error> {
        for op in operations {
            match op {
                Operation::Read(read) => {
                    self.bus.read(read).map_err(|_| {
                        Error::Io(io::Error::new(
                            io::ErrorKind::Other,
                            "SimpleHalSpiDevice read transaction error",
                        ))
                    })?;
                }
                Operation::Write(write) => {
                    self.bus.write(write).map_err(|_| {
                        Error::Io(io::Error::new(
                            io::ErrorKind::Other,
                            "SimpleHalSpiDevice write transaction error",
                        ))
                    })?;
                }
                Operation::Transfer(read, write) => {
                    self.bus.transfer(read, write).map_err(|_| {
                        Error::Io(io::Error::new(
                            io::ErrorKind::Other,
                            "SimpleHalSpiDevice read/write transaction error",
                        ))
                    })?;
                }
                Operation::TransferInPlace(words) => {
                    self.bus.transfer_in_place(words).map_err(|_| {
                        Error::Io(io::Error::new(
                            io::ErrorKind::Other,
                            "SimpleHalSpiDevice in-place read/write transaction error",
                        ))
                    })?;
                }
                Operation::DelayUs(us) => {
                    Delay::new().delay_us(*us);
                }
            }
        }
    	Ok(())
    }
}

impl<B: SpiBus<u8>> ErrorType for SimpleHalSpiDevice<B> {
    type Error = Error;
}
