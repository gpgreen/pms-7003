use pms_7003::Pms7003Sensor;

struct RxMock {}
struct TxMock {}

impl embedded_hal_nb::serial::ErrorType for RxMock {
    type Error = pms_7003::Error;
}

impl embedded_hal_nb::serial::Read<u8> for RxMock {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        Err(nb::Error::Other::<pms_7003::Error>(
            pms_7003::Error::NoResponse,
        ))
    }
}

impl embedded_hal_nb::serial::ErrorType for TxMock {
    type Error = pms_7003::Error;
}

impl embedded_hal_nb::serial::Write<u8> for TxMock {
    fn write(&mut self, _: u8) -> nb::Result<(), Self::Error> {
        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }
}

#[test]
fn crate_instance_using_separate_rx_tx() {
    let tx = TxMock {};
    let rx = RxMock {};

    let mut pms = Pms7003Sensor::new_tx_rx(tx, rx);
    let _ = pms.sleep();
}
