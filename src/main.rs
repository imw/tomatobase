#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use panic_halt as _;
use microbit::board::Board;
use microbit::display::blocking::Display;
use microbit::hal::{
    prelude::*,
    timer::Timer,
    saadc::SaadcConfig,
    Saadc,
 };
use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let b = Board::take().unwrap();

    let mut timer = Timer::new(b.TIMER0);

    let mut adc = Saadc::new(b.SAADC, SaadcConfig::default());
    let mut adc_pin1 = b.pins.p0_02;
    let mut adc_pin2 = b.pins.p0_03;

    let mut display = Display::new(b.display_pins);
    let mut leds = [
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ];


    loop {
        let pressure1 = adc.read(&mut adc_pin1).unwrap();
        let pressure2 = adc.read(&mut adc_pin2).unwrap();
        let mut q1 = pressure1 / 3000;
        let mut q2 = pressure2 / 3000;
        for i in 0..5 {
            let mut row = leds[i];
            if q1 > 0 { 
                row[0] = 1;
                q1 = q1 - 1;
            } else {
                row[0] = 0;
            }
            if q2 > 0 { 
                row[4] = 1;
                q2 = q2 - 1;
            } else {
                row[4] = 0;
            }
            leds[i] = row;
        }
        rprintln!("q1: {}", q1);
        rprintln!("q2: {}", q2);
        rprintln!("leds: {:?}", leds);
        
        display.show(&mut timer, leds, 500);
    }
}
