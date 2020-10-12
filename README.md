[![crates.io](https://img.shields.io/crates/v/drogue-es-wifi.svg)](https://crates.io/crates/drogue-es-wifi)
[![docs.rs](https://docs.rs/drogue-es-wifi/badge.svg)](https://docs.rs/drogue-es-wifi)
[![Matrix](https://img.shields.io/matrix/drogue-iot:matrix.org)](https://matrix.to/#/#drogue-iot:matrix.org)

# `drogue-es-wifi`

Driver for the [Inventek eS-WiFi SPI WiFi offload board](https://www.digikey.com/en/products/detail/inventek-systems/ISM43362-M3G-L44-E-C6-2-1-8/7070042).

## Usage

The eS-WiFi board is interfaced over an SPI peripheral, plus a handful of additional pins:

* SPI
  * SCK
  * CIPO
  * COPI
* chip-select
* ready
* wake-up
* reset

### Pins

Gather and configure your SPI and non-SPI pins appropriately for your particular chipset:

```rust
let sck = gpioc.pc10.into_af6(&mut gpioc.moder, &mut gpioc.afrh);

let cipo = gpioc.pc11.into_af6(&mut gpioc.moder, &mut gpioc.afrh);

let copi = gpioc.pc12.into_af6(&mut gpioc.moder, &mut gpioc.afrh);

let mut cs = gpioe.pe0.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

let mut ready = gpioe.pe1.into_pull_up_input(&mut gpioe.moder, &mut gpioe.pupdr);

let mut reset = gpioe.pe8.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

let mut wakeup = gpiob.pb13.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

let mut spi = Spi::spi3(
    device.SPI3,
    (sck, cipo, copi),
    MODE,
    20.mhz(),
    clocks,
    &mut rcc.apb1r1,
);
```

### Clock

An `embedded-time` `Clock` is required in order to provide appropriate delays and timeouts.
The [drogue-embedded-timer](https://crates.io/crates/drogue-embedded-timer) crate provides utilities and instructions for managing clocks.

### Initialization

One the SPI peripheral and additional pins and clock are initialized, you can initialize the WiFi adapter:

```rust
let wifi = drogue_es_wifi::adapter::Adapter::new(
    spi,
    cs,
    ready,
    wakeup,
    reset,
    &CLOCK,
).unwrap();
```

## Join an access point

The `&CLOCK` must be ticking forward prior to using the adapter.

To join a WEP-secured access-point:

```rust
let response = wifi.join_wep("drogue", "rodneygnome");
```

## TCP connections

The adapter directly implements the [`drogue-network`](https://crates.io/crates/drogue-network) `TcpStack`:

```rust
let remote = HostSocketAddr::new(
    HostAddr::new(IpAddr::from_str("192.168.1.245").unwrap(), Option::None),
    80,
);

let mut socket = wifi.open(drogue_network::tcp::Mode::Blocking).unwrap();
let mut socket = wifi.connect(socket, remote).unwrap();

let len = wifi.write(&mut socket, b"GET / HTTP/1.1\r\nhost:192.168.1.245\r\n\r\n").unwrap();

loop {
    let mut buffer: [u8; 1024] = [0; 1024];
    let result = wifi.read(&mut socket, &mut buffer);

    match result {
        Ok(len) => {
            let s = core::str::from_utf8(&buffer[0..len]).unwrap();
            log::info!( "{}", s);
        }
        Err(e) => {
            match e {
                Error::Other(e) => {
                    log::error!("error {:?}", e);
                    break;
                }
                Error::WouldBlock => {
                }
            }
        }
    }
}

wifi.close(socket).unwrap();
```
