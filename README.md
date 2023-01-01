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

[![asciicast](https://asciinema.org/a/491359.svg)](https://asciinema.org/a/491359)
