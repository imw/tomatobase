%title: tomato base
%author: imw
%date: 2022-08-12

-> tomato base <-
=========

-> the seat cushion that tells you when you get off your can <-

---

-> # this <-

- hardware
- software
- approaches
- results
- questions


-------------------------------------------------

-> # hardware <-

- bbc microbit v2
- plug board
- bread board
- 2x joy-it SEN-PRESSURE10
- jumpers

-------------------------------------------------

-> # software <-

- rust
- nrf-rs community tools
- cargo embed
- gdb-multiarch

-------------------------------------------------

-> # approaches - basic i/o <-

  let pressure1 = adc.read(&mut adc_pin1).unwrap();
  let pressure2 = adc.read(&mut adc_pin2).unwrap();
  let mut q1 = pressure1 / 3000;
  let mut q2 = pressure2 / 3000;
  for i in 0..5 {
      let mut row = leds[i];
      ...
  }
  display.show(&mut timer, leds, 500);

-------------------------------------------------

-> # approaches - timing and interrupts <-

  static RTC: Mutex<RefCell<Option<Rtc<pac::RTC0>>>> = Mutex::new(RefCell::new(None));
  static SPEAKER: Mutex<RefCell<Option<pwm::Pwm<pac::PWM0>>>> = Mutex::new(RefCell::new(None));
  static ADC: Mutex<RefCell<Option<Saadc>>> = Mutex::new(RefCell::new(None));
  ...
  unsafe {
      pac::NVIC::unmask(pac::Interrupt::RTC0);
  }   
  pac::NVIC::unpend(pac::Interrupt::RTC0);
  ...
   fn RTC0() {
      static mut FREQUENCY: u32 = 1;
      /* Enter critical section */
      cortex_m::interrupt::free(|cs| {
          /* Borrow devices */
          if let (Some(speaker), Some(rtc),Some(adc)) = (
              SPEAKER.borrow(cs).borrow().as_ref(),
              RTC.borrow(cs).borrow().as_ref(),
              ADC.borrow(cs).borrow().as_ref(),
          ) {
	  ...
   }

-------------------------------------------------

-> # approaches - rtc.wait() <-
  let expired = timer0.wait();
  let sat_upon = q1 > 0 || q2 > 0;
  match expired {
      Err(_) => {
          if sat_upon {
              timer0.start(alarm_period);
              alarm = false;
          } else {
              alarm = true;
          }
      }
      Ok(_) => {
          continue
      }
  }
  ...

-------------------------------------------------
-> # approaches - rtc.read() <-

  let sat_upon = q1 > 0 || q2 > 0;
  if !sat_upon {
      last_stand = rtc.get_counter();
  }

  let time = rtc.get_counter();
  if time - last_stand > alarm_period {
      alarm = true
  } else {
      alarm = false
  }

-------------------------------------------------

-> # results <-

-> ## live demo <-

-------------------------------------------------

-> # questions <-

- what would a good enclosure look like?
- how to attach sensors well?
- what about a version with linqstat?
- what about a standing desk integration
- how to add cooling
- how to add haptics
