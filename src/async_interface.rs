// async_interface

use crate::read_fsm::{self, ReadStatus};
use crate::{
    ACTIVE_MODE_RESPONSE, Error, OUTPUT_FRAME_SIZE, OutputFrame, PASSIVE_MODE_RESPONSE,
    RESPONSE_FRAME_SIZE, Response, SLEEP_RESPONSE,
};
use embedded_io_async::{Read, Write};

impl embedded_io_async::Error for crate::Error {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        embedded_io_async::ErrorKind::Other
    }
}

/// pms-7003 Sensor with async interface
pub struct Pms7003SensorAsync<Serial> {
    serial: Serial,
}

impl<Serial> embedded_io_async::ErrorType for Pms7003SensorAsync<Serial>
where
    Serial: Write + Read,
{
    type Error = crate::Error;
}

impl<Serial> Pms7003SensorAsync<Serial>
where
    Serial: Write + Read,
{
    /// Creates a new sensor instance
    /// * `serial` - single object implementing embedded-io-async serial traits
    pub fn new(serial: Serial) -> Self {
        Self { serial }
    }

    /// Reads sensor status.
    pub async fn read(&mut self) -> Result<OutputFrame, Error> {
        OutputFrame::from_buffer(&self.read_from_device([0_u8; OUTPUT_FRAME_SIZE]).await?)
    }

    /// Sleep mode. May fail because of incorrect response because of race condition between response and air quality status
    pub async fn sleep(&mut self) -> Result<(), Error> {
        self.send_cmd(&crate::create_command(0xe4, 0)).await?;
        self.receive_response(SLEEP_RESPONSE).await
    }

    /// Wake the sensor from sleep
    pub async fn wake(&mut self) -> Result<(), Error> {
        self.send_cmd(&crate::create_command(0xe4, 1)).await
    }

    /// Passive mode - sensor reports air quality on request
    pub async fn passive(&mut self) -> Result<(), Error> {
        self.send_cmd(&crate::create_command(0xe1, 0)).await?;
        self.receive_response(PASSIVE_MODE_RESPONSE).await
    }

    /// Active mode - sensor reports air quality continuously
    pub async fn active(&mut self) -> Result<(), Error> {
        self.send_cmd(&crate::create_command(0xe1, 1)).await?;
        self.receive_response(ACTIVE_MODE_RESPONSE).await
    }

    /// Requests status in passive mode
    pub async fn request(&mut self) -> Result<(), Error> {
        self.send_cmd(&crate::create_command(0xe2, 0)).await
    }

    async fn send_cmd(&mut self, cmd: &[u8]) -> Result<(), Error> {
        self.serial
            .write(cmd)
            .await
            .map_err(|_| Error::SendFailed)?;
        Ok(())
    }

    async fn read_from_device<T: AsMut<[u8]>>(&mut self, mut buffer: T) -> Result<T, Error> {
        let mut read = read_fsm::ReadStateMachine::new(buffer.as_mut(), 0);
        loop {
            let mut chbuf = [0; 1];
            match self.serial.read(&mut chbuf).await {
                Ok(count) if count == 1 => {
                    let ch: Result<u8, nb::Error<Error>> = Ok(chbuf[0]);
                    match read.update(ch) {
                        ReadStatus::Finished => return Ok(buffer),
                        ReadStatus::Failed => return Err(Error::ReadFailed),
                        _ => {}
                    }
                }
                Ok(_) => {}
                Err(_) => return Err(Error::ReadFailed),
            }
        }
    }

    async fn receive_response(&mut self, expected_response: Response) -> Result<(), Error> {
        if self.read_from_device([0u8; RESPONSE_FRAME_SIZE]).await? != expected_response {
            Err(Error::IncorrectResponse)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core::fmt::Debug;
    use embedded_hal::serial::{Read, Write};
    use embedded_hal_mock::MockError;
    use embedded_hal_mock::serial::{Mock as SerialMock, Transaction as SerialTransaction};
    use heapless::Vec;
    //extern crate std;

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[derive(Debug)]
    enum AsyncError<E> {
        Mock(E),
    }

    struct AsyncSerialMock(SerialMock<u8>);

    // impl AsyncSerialMock {
    //     pub fn done(&mut self) {
    //         self.0.done();
    //     }
    // }

    impl embedded_io_async::ErrorType for AsyncSerialMock {
        type Error = AsyncError<nb::Error<MockError>>;
    }

    impl<E: Debug> embedded_io_async::Error for AsyncError<E> {
        fn kind(&self) -> embedded_io_async::ErrorKind {
            embedded_io_async::ErrorKind::Other
        }
    }

    impl<E> From<E> for AsyncError<E> {
        fn from(e: E) -> AsyncError<E> {
            AsyncError::Mock(e)
        }
    }

    impl embedded_io_async::Read for AsyncSerialMock {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            //std::println!("read {} bytes", buf.len());
            for i in 0..buf.len() {
                let b = self.0.read()?;
                buf[i] = b;
                //std::println!("byte: {}", b);
            }
            Ok(buf.len())
        }
    }

    impl embedded_io_async::Write for AsyncSerialMock {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            for i in 0..buf.len() {
                self.0.write(buf[i])?;
            }
            Ok(buf.len())
        }
    }

    #[test]
    fn passive() {
        // Configure expectations
        let cmd = crate::create_command(0xe1, 0);
        let mut expectations: Vec<SerialTransaction<u8>, 32> = Vec::new();
        for byte in cmd.iter() {
            expectations.push(SerialTransaction::write(*byte)).ok();
        }
        let resp = PASSIVE_MODE_RESPONSE;
        for byte in resp.iter() {
            expectations.push(SerialTransaction::read(*byte)).ok();
        }
        let serial = AsyncSerialMock(SerialMock::new(expectations.as_slice()));

        let mut pms_sensor = Pms7003SensorAsync::new(serial);

        assert!(aw!(pms_sensor.passive()).is_ok());

        // When you believe there are no more calls on the mock,
        // call done() to assert there are no pending transactions.
        //serial.done();
    }

    #[test]
    fn sleep() {
        // Configure expectations
        let cmd = crate::create_command(0xe4, 0);
        let mut expectations: Vec<SerialTransaction<u8>, 32> = Vec::new();
        for byte in cmd.iter() {
            expectations.push(SerialTransaction::write(*byte)).ok();
        }
        let resp = SLEEP_RESPONSE;
        for byte in resp.iter() {
            expectations.push(SerialTransaction::read(*byte)).ok();
        }
        let serial = AsyncSerialMock(SerialMock::new(expectations.as_slice()));

        let mut pms_sensor = Pms7003SensorAsync::new(serial);

        assert!(aw!(pms_sensor.sleep()).is_ok());

        // When you believe there are no more calls on the mock,
        // call done() to assert there are no pending transactions.
        //serial.done();
    }
}
