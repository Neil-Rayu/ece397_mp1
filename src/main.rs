#![no_std]
#![no_main]

use core::marker::PhantomData;

use cortex_m_rt::entry;
use cortex_m::asm;
use defmt::*;
use embassy_mspm0::gpio::{Output, Level};
use embassy_mspm0::trng::Trng;
use embassy_mspm0::uart::{Uart, Config};
use rand_core::{TryRngCore, CryptoRng};
use {defmt_rtt as _, panic_halt as _};

trait VaultState {}
struct Unbound;
struct Locked;
struct Unlocked;

impl VaultState for Unbound {}
impl VaultState for Locked {}
impl VaultState for Unlocked {}

struct Vault<State: VaultState> {
    pin: [u8; 2],
    failed_attempts: u32,
    secret: &'static str,
    _state: PhantomData<State>,
}

impl Default for Vault<Unbound> {
    fn default() -> Self {
        Vault {
            pin: [0; 2],
            secret: "",
            failed_attempts: 0,
            _state: core::marker::PhantomData,
        }
    }
}

impl Vault<Unbound> {
    pub fn bind(self, pin: [u8; 2]) -> Vault<Locked> {
        Vault {
            pin,
            failed_attempts: 0,
            secret: "",
            _state: core::marker::PhantomData,
        }
    }
}

impl Vault<Locked> {
    // Secret should be introduced when successfully unlocked.
    pub fn unlock(self, pin: [u8; 2]) -> Result<Vault<Unlocked>, Vault<Locked>> {
        if pin == self.pin {
            Ok(Vault {
                pin: self.pin,
                failed_attempts: self.failed_attempts,
                secret: "whaaat la policia noooo",
                _state: core::marker::PhantomData,
            })
        } else {
            Err(Vault {
                pin: self.pin,
                failed_attempts: self.failed_attempts + 1,
                secret: "",
                _state: core::marker::PhantomData,
            })
        }
    }
}

fn generate_pin<T: CryptoRng>(mut rng: T) -> [u8; 2] {
    let mut pin = [0u8; 2];
    rng.try_fill_bytes(&mut pin).expect("RNG failed");
    pin[0] = pin[0] % 4 + 1; 
    pin[1] = pin[1] % 4 + 1; 
    pin
}

fn read_command(uart: &mut Uart<'_, embassy_mspm0::mode::Blocking>) -> [u8; 3] {
    let mut cmd_buf = [0u8; 3];
    let mut idx = 0;

    let mut byte = [0u8; 1];
    unwrap!(uart.blocking_read(&mut byte));

    while byte[0] != b'\n' && byte[0] != b'\r' {
        if idx < cmd_buf.len() {
            cmd_buf[idx] = byte[0];
            idx += 1;
        }
        // Keep reading even if buffer is full (to drain input)
        unwrap!(uart.blocking_read(&mut byte));
    }
    
    // If we stopped at \r, also consume the \n that follows
    if byte[0] == b'\r' {
        unwrap!(uart.blocking_read(&mut byte)); // Read the \n
    }
        
    cmd_buf
}

#[entry]
fn main() -> ! {
    info!("eCTF MP1 started");

    let p = embassy_mspm0::init(Default::default());

    let mut trng = Trng::new(p.TRNG).expect("Failed to initialize TRNG");

    let instance = p.UART0;
    let tx = p.PA10;
    let rx = p.PA11;

    let config = Config::default();
    let mut uart = unwrap!(Uart::new_blocking(instance, rx, tx, config));

    let mut led1 = Output::new(p.PA0, Level::Low);
    led1.set_inversion(true);



    loop {
        let vault: Vault<Unbound> = Default::default();
        // ...wait for x command
        loop {
            let _cmd = read_command(&mut uart);
            if _cmd[0] == b'x' {
                break;
            }
        }

        let pin = generate_pin(trng.unwrap_mut());
        let mut vault = vault.bind(pin);

        info!("pin generated: {} {}", pin[0], pin[1]);
        // TODO: blink LED to show pin...
        for _ in 0..pin[0] {
            asm::delay(9_600_000); // ~300ms at 32MHz
            info!("toggle");
            led1.toggle();
            asm::delay(6_400_000); // ~200ms at 32MHz
            led1.toggle();
        }
        asm::delay(19_200_000); // ~600ms at 32MHz
        for _ in 0..pin[1] {
            asm::delay(9_600_000); // ~300ms at 32MHz
            info!("toggle");
            led1.toggle();
            asm::delay(6_400_000); // ~200ms at 32MHz
            led1.toggle();
        }
        let vault = loop {
            // ...wait for g__ command
            let mut _cmd: [u8; 3] = read_command(&mut uart);
            loop {
                
                if _cmd[0] == b'g' {
                    break;
                }
                _cmd = read_command(&mut uart);
            }
            
            let mut pin = [0u8; 2]; // TODO: get pin from command
            pin[0] = _cmd[1] - b'0';  
            pin[1] = _cmd[2] - b'0';

            match vault.unlock(pin) {
                Ok(unlocked_vault) => {
                    unwrap!(uart.blocking_write(b"pin correct\r\n"));
                    break unlocked_vault;
                }
                Err(locked_vault) => {
                    unwrap!(uart.blocking_write(b"pin incorrect\r\n"));
                    vault = locked_vault;
                }
            }
        };
        loop {
            // ... wait for command
            let mut _cmd = read_command(&mut uart);
            loop {
                if _cmd[0] == b'q' || _cmd[0] == b'u' {
                    break;
                }
                _cmd = read_command(&mut uart);
            }
            if _cmd[0] == b'q' {
                let secret_bytes = vault.secret.as_bytes();
                unwrap!(uart.blocking_write(secret_bytes));
                unwrap!(uart.blocking_write(b"\r\n"));
            }
            if _cmd[0] == b'u' {
                break;
            } 
        }
    }
}
