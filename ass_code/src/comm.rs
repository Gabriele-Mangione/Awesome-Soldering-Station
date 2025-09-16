
use core::{cell::RefCell, fmt::Write};

use critical_section::Mutex;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, WithTimeout};
use embedded_io::{Read, ReadReady};
use esp_hal::{
    gpio::AnyPin,
    handler,
    interrupt::InterruptConfigurable,
    uart::{self, AnyUart, AtCmdConfig, Uart, UartInterrupt},
};

pub struct Comm {
    uart_peripheral: AnyUart,
    to_uc_n: AnyPin,
    to_uc_p: AnyPin,
}

impl Comm {
    async fn pulse_check() -> bool {
        critical_section::with(|cs| {
            let mut u = UART.borrow_ref_mut(cs).as_mut().unwrap();
            u.write_char('?').expect("uart write fail");
            let mut buf = [0u8; 1];
        });

        match SOLDERING_MESSAGE.receive().with_timeout(Duration::from_millis(100)).await {
            Err(e) => {
                return false;
            },
            Ok(SolderingMessage::PulseCheck) => {
                return true;
            }
            Ok(o) => {
                panic!("pulse check returned something else");
            }
        }
    }
}

enum SolderingMessage {
    PulseCheck,
    Magnet,
    Gyro,
    Unknown
}

static UART: Mutex<RefCell<Option<Uart<esp_hal::Blocking>>>> = Mutex::new(RefCell::new(None));
static SOLDERING_MESSAGE: Channel<CriticalSectionRawMutex, SolderingMessage, 10> = Channel::new();

#[embassy_executor::task]
async fn communication_task(c: Comm) {
    let mut u = Uart::new(
        c.uart_peripheral,
        uart::Config::default()
            .with_baudrate(9600)
            .with_rx_timeout(0),
    )
    .unwrap()
    .with_rx(c.to_uc_n)
    .with_tx(c.to_uc_p);

    u.write_char('?').expect("uart write fail");

    u.set_interrupt_handler(uart_handler);
    critical_section::with(|cs| {
        u.set_at_cmd(AtCmdConfig::default());
        u.listen(UartInterrupt::AtCmd);

        UART.borrow_ref_mut(cs).replace(u);
    });

    loop {

        //wait for message
    }
}

#[handler]
fn uart_handler() {
    critical_section::with(|cs| {
        let mut u = UART.borrow_ref_mut(cs);
        let u = u.as_mut().unwrap();

        let mut buf = [0u8;1];
        u.read(&mut buf);

        let m: SolderingMessage = match buf[0] {
            b'?' => SolderingMessage::PulseCheck,
            b'M' => SolderingMessage::Magnet,
            b'G' => SolderingMessage::Gyro,
            _ => SolderingMessage::Unknown,
        };
        SOLDERING_MESSAGE.send(m);

    });
}
