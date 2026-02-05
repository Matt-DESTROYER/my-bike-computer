# My bike computer
I recently came accross this [ESP32-S3 RLCD Development Board at Waveshare](https://www.waveshare.com/esp32-s3-rlcd-4.2.htm?sku=33298).
It seemed cool and I immediately got to thinking, what could I make with this?
Long story short, the answer is a bike computer!
Paired with a Beitian BN 880 GPS and a 32 GB SD card; the plan is a bicycle computer that can tell me my current speed, the distance I've travelled, the temperature, and even display a map (which will come from that SD).

## Setting up your environment
To build this project for ESP chips, you will need the following setup:
```sh
cargo install espup
espup install
```
> Note: this will probably take a little while.

### To generate your own project:
```sh
cargo install esp-generate
esp-generate --chip chip-name project-name
```
