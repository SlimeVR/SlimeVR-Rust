# Project Goals and Direction

We have categorized the project goals into short long term goals.

## Long Term Goals
Non-Technical:
* Raise the bar for affordability and quality of Full Body Tracking.
* Be open source, encourage but not require others to follow suit.
* Encourage the use of Rust by Doing Cool Shit™.
* Give back to the SlimeVR and the Rust communities.
* Have sustainability for both the code and contributors. 
* Collaborate rather than compete with other FBT solutions.
* Be welcoming to new contributors.
* 🦀 HAVE FUN 🦀

Technical Goals:
* Be memory and type safe. Make illegal states unrepresentable in code.
* Have readable and ergonomic code.
* Leverage async/await to make concurrency easy.
* Leverage cargo to make adding dependencies easy.
* Leverage Rust's math and sensor fusion ecosystem.
* Leverage Rust's embedded ecosystem (`embedded-hal`) instead of Arduino.
* Structure code into reusable libraries, such that they can be utilized by other
  members of the SlimeVR and Rust communities.
* Sensor fusion is imu-agnostic. Algorithms are reusable across sensors.
* Networking protocol is mcu-agnostic.

## Short Term Goals
In the short term, we are focused on implementing a Minimal Viable Product (MVP).
Until we reach the MVP, the code should be considered pre-pre-alpha.

A firmware MVP has these features:
* Flashable by a tech savvy person who isn't a rust developer and doesn't have a
  hardware probe.
* ESP32C3 works with SlimeVR server over UDP.
* NRF52840 works with SlimeVR server over BLE (possibly via BLE->UDP converter on the
  host).
* Supports the MPU6050, BMI160, BNO080, LSM6DS3. Other IMUs are a bonus but not needed
  for the MVP.
* Abstracts over hardware specifics.
* Supports configuring pinouts.
* Supports simple sensor fusion. Doesn't have to be perfectly tuned or save anything in
  flash.
* Support for sensing battery level.
* Either the BMI160 or LSM6DS3 should outperform the MPU6050. This will let hardware
  devs build boards with these sensors and later we can improve the fusion.

In service of this MVP, the following are explicit NON GOALS in the short term, even
though they are helpful in the long term:
* The list of MCUs needs to stay small. Adding the nrf52840 already increased the scope
  of the MVP considerably.
* Magnetometers for FBT are not worth the time and effort at this early stage.
* A perfect API and architecture design is unlikely on the first try, and the code will
  evolve over time. WIP code is fine as long as it doesn't break anything else.
