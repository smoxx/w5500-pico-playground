# WIZnet W5500-EVB-Pico-PoE Blinky

Simple embassy embedded project to control LED presets
over the local Network.

## Setup

---

Build the project:

```bash
cargo +stable build --release
```

Flash it onto the microcontroller:

```bash
cargo +stable build --release
```

## Usage

---

Connect the Microcontroller with either LAN and USB-C,
or only LAN if your switch supports PoE
and your microcontroller has the WIZPoE-P1 Daughterboard.

Check your Router for a device named "w5500-pico-playground".
Remember the assigned IP Address.
(Hostname can be changed in `network.rs`)

Send requests via Netcat, e.g.:

```bash
printf '<your-command>' | nc -u -w 2 <IPv4 Adress of the pico> 4000
```

The available commands are defined in `control.rs`:

- 'ping' -> Pico answers 'pong'
- 'heartbeat' -> Heartbeat LED Pattern
- 'slow' -> Slow LED Pattern
- 'breathe' -> Breathing LED Pattern
- 'on' -> Switch LED on
- 'off' -> Switch LED off
