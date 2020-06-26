# RA8875 Driver

A Rust driver for the RA8875 display chip. This driver is adapted from
Adafruit's open-source driver for their RA8875 line of TFT displays. [See their
original repository for more details on supported hardware][adafruit-repo].

[adafruit-repo]: https://github.com/adafruit/Adafruit_RA8875

This driver implements the `embedded-graphics` `DrawTarget` interface. This
driver is not yet feature-complete, but has enough features to get started
using the Adafruit driver board.
