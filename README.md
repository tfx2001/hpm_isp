# hpm_isp

[![Crate.io](https://img.shields.io/crates/v/hpm_isp)](https://crates.io/crates/hpm_isp)
[![Crate.io](https://img.shields.io/crates/d/hpm_isp)](https://crates.io/crates/hpm_isp)
[![Stars](https://img.shields.io/github/stars/tfx2001/hpm_isp)](https://github.com/tfx2001/hpm_isp)
[![LICENSE](https://img.shields.io/github/license/tfx2001/hpm_isp)](https://github.com/tfx2001/hpm_isp/blob/main/LICENSE)
[![Build](https://github.com/tfx2001/hpm_isp/actions/workflows/build.yml/badge.svg)](https://github.com/tfx2001/hpm_isp/actions/workflows/build.yml)

An ISP (In-system programming) tool for HPMicro MCUs.

## Install

### Pre-built binaries (Recommended)

[Release](https://github.com/tfx2001/hpm_isp/releases/latest)

### Cargo

```shell
cargo install hpm_isp
```

### For Linux users

```shell
sudo cp 99-hpm_bootrom.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
```

## Usage

```shell
# Write to flash (use default memory config)
hpm_isp flash 0 write 0x400 flash.bin
# Write to flash (use custom memory config)
# Note: if hpm_isp.bin exists in the working directory, it will be used by default.
# So you don't need to pass -c option explicitly.
hpm_isp flash -c hpm_isp.bin 0 write 0x400 flash.bin
# Read from flash
hpm_isp flash 0 read 0x0 0x4000 flash.bin
# Use config wizard to generate config file (save as hpm_isp.bin)
hpm_isp wizard
```

[![asciicast](https://asciinema.org/a/491359.svg)](https://asciinema.org/a/491359)
