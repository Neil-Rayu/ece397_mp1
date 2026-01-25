# ECE 397 eCTF MP1

Detailed instructions in [syllabus](https://sigpwny.com/2026-ectf-syllabus).

## Running

- Install dependencies:
  - Nixos: use flake (`nix develop`) and add [udev rules](https://probe.rs/docs/getting-started/probe-setup/#linux-udev-rules). Otherwise,
  - Rustup (should use everything specified in `rust-toolchain.toml`)
  - [probe-rs](https://probe.rs/docs/getting-started/installation/) ([setup instructions](https://probe.rs/docs/getting-started/probe-setup/) include details like udev rules)
  - picocom/PuTTY
- Connect to UART:
  - If on *nix with `picocom`, `picocom -b 115200 /dev/ttyACM0 --omap crcrlf` (replace `ttyACM0` with the device created). You can exit with `Ctrl+A` and `Ctrl+X`.
  - If on Windows with PuTTY, set baud to `115200` and select correct serial port
- Run binary: `cargo run --release`

## Pointers
- UART example: https://github.com/embassy-rs/embassy/blob/main/examples/mspm0l2228/src/bin/uart.rs
- LED blink: https://github.com/embassy-rs/embassy/blob/main/examples/mspm0l2228/src/bin/blinky.rs
- For delay: [`cortex_m::asm::delay`](https://docs.rs/cortex-m/latest/cortex_m/asm/fn.delay.html)