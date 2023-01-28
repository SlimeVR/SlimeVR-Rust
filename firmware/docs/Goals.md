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
* Structure our code into reusable libraries, such that they can be utilized by other
  members of the SlimeVR and Rust communities.
* Sensor fusion is imu-agnostic. We can reuse the algorithms across multiple sensors.
* Networking protocol is mcu-agnostic.

## Short Term Goals
Long term goals are important, but meaningless without short term goals. There is no
long term if the codebase isn't able to be used in the real world! Its important to
understand that in the short term, we are primarily focused on implementing a Minimal
Viable Product (MVP).

A firmware MVP has these features:
* Can be flashed to a device by a tech savvy person who isn't a rust developer, via
  accompanying documentation, without a hardware probe.
* Communicates with the SlimeVR server over UDP on ESP32C3. 
* Works with SlimeVR server of BLE on nrf52840. Can be accomplished via host-side
  BLE->UDP conversion program, if that is necessary or eaiser.
* Supports commonly used or highly desired IMUs by the SlimeVR community, such as
  MPU6050, BMI160, BNO080, LSM6DS3. Currently, this list is exhaustive. Other IMUs are
  a bonus but not required for the MVP.
* Has an architecture in place to abstract over specifics of hardware.
* Has a basic mechanism in place to configure pinouts.
* Has a basic mechanism in place to configure other features in the firmware.
* Has support for a few simpler types of sensor fusion, which are agnostic to the
  particular sensor. We want to provide a base for other rust-curious developers to
  experiment on top of, but we don't necessarily need or want something perfectly tuned
  yet. I would consider anything that requires saving a calibration in flash to be out
  of scope for the MVP.
* Support for sensing battery level.
* Either the bmi160 or lsm6ds3 on our firmware should outperform the mpu6050 in real
  world use. This allows hardware devs to choose the superior sensor, with the
  expectation that we will get even less IMU drift as we make the sensor fusion. We know
  its possible, so lets do it.
* Support for MPU6050, BNO080, LSM6DS3, and BMI160. The first two are because they are
  common in the SlimeVR community. The BNO080 is also a requirement because it is needed
  by the official testing jig in slimevr, and because I don't want people with expensive
  BNOs unable to use our firmware. The BMI160 is required because it allows us to make
  comparisons to the bmi160 support in the arduino firmware. The LSM6DS3 is required
  because it is the sensor bundled in the seeed studio xiao sense (nrf52840) board, and
  has superior performance to the bmi160 (on paper). It may be the key to beating the
  arduino firmware and making high quality fbt small and affordable (i.e. not needing
  a BNO080), so I am resting a lot of hope on this sensor comboed with sensor fusion.

In service of accomplishing this MVP, the following are explicit NON GOALS in the short
term, even though they are helpful in the long term:
* ~~Supporting every mcu under the sun~~. Avoid feature creep. Adding the nrf52840
  already increased the scope of the MVP by *a lot*. It was necessary, but I don't want
  to officially support another MCU yet. That being said, MCUs that we have already
  added support for, like the nrf52840, esp32c3, esp32, nrf52832, should continue to
  work.
* ~~Supporting every imu under the sun~~. Avoid feature creep. 
* ~~Supporting mangnetometers~~. I'm very skeptical of the value of magnetometers for
  FBT, and I don't want us spending time on them at this early stage. The codebase
  should not even have the conecept of a magnetometer yet. Everything should be 6dof.
* ~~Having a perfectly architected system~~. We often won't know what the right way is
  to do something, until we have written it once first. Lets write it and then refactor
  once we have experience with what we like and dislike about that implementation.
* ~~Committing to api or end-user stability~~. The codebase is experimental. End-users
  should expect to treat it as such. Anything can and will change. We are doing this in
  our free time, so if someone intends on shipping this code on hardware that they are
  selling, they should either get familiar enough with rust to make the changes or
  bugfixes they want and then submit a PR, they should pay one or more of the
  contributors to make the change so that it is actually worth their time. Even then,
  we still will not commit to stability on features until a later point in time, when
  the MVP is complete and we can think about the next steps.
* ~~Micro-optimizations, both in CPU and power draw~~. Looking at optimizing the code is
  always nice, but we need a proper benchmarking system (like criterion) in place to
  say anything meaningful, and I don't think we should focus on that right now. When it
  comes to optmizing power draw, that is more important especially for the networking
  code, but as long as the networking *works*, I'll be happy. We do however, want
  battery life roughly in the ballpark of the arduino firmware, at the very least.
* ~~Avoiding experimentation in the codebase~~. Instead, we should expect that as we
  implement features, we are going to have somewhat WIP code being added. This WIP code
  should be an improvement on what exists in the codebase at the time of contribution.
  So if its going to break an existing feature, that WIP code should be feature flagged
  such that we have to opt-into its use. If its not modifying an existing feature, then
  it doesn't need to be feature flagged, as long as it doesn't prevent the use of the
  firmware in its normal use. For more, see [Pull Request Tips.md].
