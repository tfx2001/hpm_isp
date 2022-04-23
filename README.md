# hpm_isp

A CLI ISP tool for HPMicro MCUs.

## Install

```shell
cargo install hpm_isp
```

## Usage

```shell
hpm_isp 0.1.0                              
tfx2001 <2479727366@qq.com>                
ISP tool for HPMicro MCUs.                 

USAGE:
    hpm_isp.exe <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    flash    Command of xpi nor flash
    help     Print this message or the help of the given subcommand(s)
```

## Example

```shell
hpm_isp flash 0 write 0x3000 ./hello_world.bin
```