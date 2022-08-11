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

/*
static RTC: Mutex<RefCell<Option<Rtc<pac::RTC0>>>> = Mutex::new(RefCell::new(None));
static SPEAKER: Mutex<RefCell<Option<pwm::Pwm<pac::PWM0>>>> = Mutex::new(RefCell::new(None));
static ADC: Mutex<RefCell<Option<Saadc>>> = Mutex::new(RefCell::new(None));
static PIN2: Mutex<RefCell<Option<gpio::p0::P0_02<gpio::Input<gpio::PullUp>>>>> = Mutex::new(RefCell::new(None));
static PIN3: Mutex<RefCell<Option<gpio::p0::P0_03<gpio::Input<gpio::PullUp>>>>> = Mutex::new(RefCell::new(None));
*/

const STOP_FREQUENCY: u32 = 500;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let mut b = Board::take().unwrap();

    let mut timer0 = Timer::new(b.TIMER0);
    let mut timer1 = Timer::new(b.TIMER1);


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
        .set_max_duty(32767);

    speaker
        .set_seq_refresh(pwm::Seq::Seq0, 0)
        .set_seq_end_delay(pwm::Seq::Seq0, 0);

    // Configure 50% duty cycle
    let max_duty = speaker.max_duty();
    speaker.set_duty_on_common(max_duty / 2);

    /*
    cortex_m::interrupt::free(move |cs| {
        // NB: The LF CLK pin is used by the speaker
        let _clocks = Clocks::new(b.CLOCK)
            .enable_ext_hfosc()
            .set_lfclk_src_synth()
            .start_lfclk();

        *ADC.borrow(cs).borrow_mut() = Some(adc);

        //TODO -- what is the actual type for the refcell?
        *PIN2.borrow(cs).borrow_mut() = Some(adc_pin1);
        *PIN3.borrow(cs).borrow_mut() = Some(adc_pin2);

        let mut rtc = Rtc::new(b.RTC0, 511).unwrap();
        rtc.enable_counter();
        rtc.enable_interrupt(RtcInterrupt::Tick, Some(&mut b.NVIC));
        rtc.enable_event(RtcInterrupt::Tick);

        *RTC.borrow(cs).borrow_mut() = Some(rtc);


        // Use the PWM peripheral to generate a waveform for the speaker
        *SPEAKER.borrow(cs).borrow_mut() = Some(speaker);

        // Configure RTC interrupt
        unsafe {
            pac::NVIC::unmask(pac::Interrupt::RTC0);
        }
        pac::NVIC::unpend(pac::Interrupt::RTC0);
    });        

        */

    let mut note: u32 = 100;
//    let mut last_low = timer.read();
    let max_duty = speaker.max_duty();
    let alarm_period = 100_000_000u32;
    let mut alarm = false;
    timer0.start(alarm_period);
    loop {
//        let time = timer0.read();
        let pressure1 = adc.read(&mut pin2).unwrap();
        let q1 = pressure1 / 3000;
        let pressure2 = adc.read(&mut pin3).unwrap();
        let q2 = pressure2 / 3000;

        
        let expired = timer0.wait();
        let sat_upon = q1 > 0 || q2 > 0;
        let mut led_indicator = leds.clone();
        match expired {
            Err(_) => {
                if sat_upon {
                    led_indicator[0][0] = 1;
                    display.show(&mut timer1, led_indicator, 500);
                    timer0.start(alarm_period);
                    alarm = false;
                } else {
                    led_indicator[0][2] = 1;
                    display.show(&mut timer1, led_indicator, 500);
                    alarm = true;
                }
            }
            Ok(_) => { 
                led_indicator[0][4] = 1;
                display.show(&mut timer1, led_indicator, 500);
            }
        }

        if alarm {
            speaker.enable();
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
        /*

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
        
        display.show(&mut timer, leds, 100);
        */
    }
}


/*
// RTC interrupt, exectued for each RTC tick
#[interrupt]
fn RTC0() {
    /* Enter critical section */
    cortex_m::interrupt::free(|cs| {
        /* Borrow devices */
        if let (Some(speaker), Some(rtc), Some(adc), Some(pin2), Some(pin3)) = (
            SPEAKER.borrow(cs).borrow().as_ref(),
            RTC.borrow(cs).borrow().as_ref(),
            ADC.borrow(cs).borrow().as_ref(),
            PIN2.borrow(cs).borrow().as_ref(),
            PIN3.borrow(cs).borrow().as_ref(),
        ) {
            let _pressure1 = adc.read(&mut pin2).unwrap();
            let _pressure2 = adc.read(&mut pin3).unwrap();
            if *FREQUENCY < STOP_FREQUENCY {
                // Configure the new frequency, must not be zero.
                // Will change the max_duty
                speaker.set_period(Hertz(*FREQUENCY));
            } else {
                // Continue at frequency
                speaker.set_period(Hertz(STOP_FREQUENCY));
            }
            // Restart the PWM at 50% duty cycle
            let max_duty = speaker.max_duty();
            speaker.set_duty_on_common(max_duty / 2);
            if *FREQUENCY >= STOP_FREQUENCY + 250 {
                defmt::info!("Fin");
                // Stop speaker and RTC
                speaker.disable();
                rtc.disable_counter();
            };
            // Clear the RTC interrupt
            rtc.reset_event(RtcInterrupt::Tick);
        }
    });
    // Increase the frequency
    *FREQUENCY += 1;
}
*/
