![dev-version](https://img.shields.io/badge/dev_version-0.1.0-orange)
![build](https://github.com/adoble/up2stream-uart/actions/workflows/ci_checks.yml/badge.svg)



# up2stream-uart

This crate provides a UART driver for the **Arylic Up2Steam Pro** board.

It provides most of the functionality provided by the UART interface to the board.

The public API is available as functions on the [Up2Stream] struct.

The main driver is created using `up2stream_uart::Up2Stream::new` which accepts
an UART peripheral that implements the `embedded_hal::serial::{Read, Write}` traits.

Values are not set directly, but through the use of either enums or scalar types (such as [Volume] or [Bass])
that provide range constraints on the values (for instance `Bass` can only accept values between -10 and +10 inclusive).
If a value if out of range an error is returned. As such, type safety is ensured.

## Example

Gets the current volume and sets it to a lower level
```rust
use up2stream_uart::{Up2Stream, Volume, ScalarParameter, Error};

// Initialise a serial peripheral on your MCU that implements the traits
// `embedded_hal::serial::{Read, Write}`. This is assigned
// the variable `serial` in the code example

// Initialise the drive using the previously setup serial peripheral
let mut up2stream_device = Up2Stream::new(&mut serial);

// Set the initial volume
let initial_vol = Volume::new(50)?;
up2stream_device.set_volume(initial_vol)?;

// Do some more processing ...

// Get the volume from the device
let actual_volume: i8 = up2stream_device.volume()?.get();

// Reduce the volume by 1 step
if actual_volume > 0 {
    let new_volume = Volume::new(actual_volume - 1)?;
    up2stream_device.set_volume(new_volume)?;
}

```
## Restrictions
* Currently only covers version 3 of the API.


## API description for the UART interface to the Up2Stream Pro.
The Arylic API for the UART  can be downloaded [here](https://developer.arylic.com/download/api-info-4.xlsx).

Configuration of the UART is 115200,8,N,1, no flow control. Source is [here](https://forum.arylic.com/t/latest-api-documents-and-uart-protocols/534/5).



## Legal Notice

Distributed under a MIT license.

The author of this sofware is not affilated in anyway with the manaufacturer or distributers of the Arylic Up2Stream Pro board.
The author just brought it for personal use and needed a driver in Rust!


