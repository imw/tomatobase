#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_halt as _;

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use microbit::board::Board;
use microbit::display::blocking::Display;
use microbit::pac::{self, interrupt};
use microbit::hal::{
    prelude::*,
    timer::Timer,
    saadc::SaadcConfig,
    Saadc,
    clocks::Clocks,
    gpio,
    pwm,
    rtc::{Rtc, RtcInterrupt},
    time::Hertz,
 };
use rtt_target::rtt_init_print;

const STOP_FREQUENCY: u32 = 500;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let mut b = Board::take().unwrap();

    let mut timer = Timer::new(b.TIMER0);


    let mut display = Display::new(b.display_pins);
    let mut leds = [
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ];


    let mut adc = Saadc::new(b.SAADC, SaadcConfig::default());
    let mut pin2 = b.pins.p0_02;
    let mut pin3 = b.pins.p0_03;
    let mut speaker_pin = b.speaker_pin.into_push_pull_output(gpio::Level::High);
    let _ = speaker_pin.set_low();

    let speaker = pwm::Pwm::new(b.PWM0);
    speaker
        // output the waveform on the speaker pin
        .set_output_pin(pwm::Channel::C0, speaker_pin.degrade())
        // Use prescale by 16 to achive darker sounds
        .set_prescaler(pwm::Prescaler::Div16)
        // Initial frequency
        .set_period(Hertz(1u32))
        // Configure for up and down counter mode
        .set_counter_mode(pwm::CounterMode::UpAndDown)
        // Set maximum duty cycle
        .set_max_duty(32767)
        // enable PWM
        .enable();

    speaker
        .set_seq_refresh(pwm::Seq::Seq0, 0)
        .set_seq_end_delay(pwm::Seq::Seq0, 0);

    // Configure 50% duty cycle
    let max_duty = speaker.max_duty();
    speaker.set_duty_on_common(max_duty / 2);


    let mut note: u32 = 100;
    let max_duty = speaker.max_duty();
    let alarm_period = 1_000u32;
    let mut alarm;
    timer.start(alarm_period);
    loop {
        let pressure1 = adc.read(&mut pin2).unwrap();
        let q1 = pressure1 / 3000;
        let pressure2 = adc.read(&mut pin3).unwrap();
        let q2 = pressure2 / 3000;

        
        let expired = timer.wait();
        let sat_upon = q1 > 0 || q2 > 0;
        match expired {
            Ok(_) => {
                if sat_upon {
                    timer.start(alarm_period);
                    alarm = false;
                } else {
                    alarm = true;
                }
            }
            Err(_) => { continue ; }
        }

        if alarm {
            if note < STOP_FREQUENCY {
                // Configure the new frequency, must not be zero.
                // Will change the max_duty
                speaker.set_period(Hertz(note));
            } else {
                // Continue at frequency
                speaker.set_period(Hertz(STOP_FREQUENCY));
            }
            // Restart the PWM at 50% duty cycle
            speaker.set_duty_on_common(max_duty / 2);

            if note >= STOP_FREQUENCY + 250 {
                defmt::info!("Fin");
                // Stop speaker and RTC
                speaker.disable();
            };
            // Increase the frequency
            note += 25;
        } else {
            speaker.disable();
            note = 100;
        }
    }
}
