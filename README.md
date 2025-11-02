# Rust interface for PMS 7003 air quality sensor

A `no-std` library for interfacing with PMS-7003 air quality sensors. This sensor uses a Serial interface that runs at 9600 baud. The fan of the
device runs at 5V, but the interface digital pins are at 3.3V.

### PMS7003 Sensor Cable Wire Connections from the datasheet
```
| Wire Co | Logic Signal | Pin No |
|--------:|-------------:|-------:|
|    Blue |          VCC |      1 |
|   Black |          VCC |      2 |
|   White |          GND |      3 |
|    Grey |          GND |      4 |
|  Purple |        Reset |      5 |
|   Green |          N/C |      6 |
|  Yellow |           RX |      7 |
|  Orange |          N/C |      8 |
|     Red |           TX |      9 |
|   Brown |          Set |     10 |
```

## Examples

There are 3 examples that use linux-embedded-hal to show various modes of the sensor. These modes don't use the Set/Reset lines of the sensor.

1. passive_mode.rs - places sensor in passive mode, then requests one
   reading, reads that, then places sensor in active mode and
   continously reads the sensor
2. read.rs - continously reads the sensor
3. sleep_and_wake.rs - puts the sensor to sleep, then wakes it, then continously reads the sensor

## Features

This crate has 2 non-default features.

 * **async** This feature implements the
   [embedded-hal-async](https://docs.rs/embedded-io-async/latest/embedded_io_async/index.html)
   interface to the sensor.

 * **defmt** This feature will enable
   [defmt](https://docs.rs/defmt/latest/defmt/)

## License

- MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)

## Links

- [defmt](https://defmt.ferrous-systems.com/)
- [Plantower PM2.5 Sensor PMS7003](https://plantower.com/en/products_33/76.html)
