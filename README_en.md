日本語版は[こちら](README.md)

# Overview

A command-line tool for writing bitstreams to FPGAs and applying DeviceTree Overlays using [jelly-fpga-server](https://github.com/ryuz/jelly-fpga-server).

This tool can be used for self-builds on boards as well as remote operations by connecting to servers.

It is particularly suited for development workflows where bitstream generation and application cross-compilation are performed in a Vivado environment, followed by remote execution on FPGA boards using scp and ssh.

Since operations requiring root privileges are handled by the jelly-fpga-server, this tool does not require root privileges to run.

# Installation

## Build and Install from Source Code

Since gRPC is used, the Protocol Buffers compiler (protoc) needs to be installed.

```bash
sudo apt update
sudo apt install protobuf-compiler
```

Installation with cargo is performed using the following command:

```bash
cargo install --git https://github.com/ryuz/jelly-fpga-loader.git
```

## Install Binary

Download the latest version for your corresponding OS and architecture from the GitHub [Releases](https://github.com/ryuz/jelly-fpga-loader/releases) page and extract it to a directory in your PATH.

Alternatively, you can easily install using cargo-binstall.

If cargo-binstall is not installed, install it with the following command:

```bash
curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
```

Then install jelly-fpga-loader with the following command:

```bash
cargo-binstall --git https://github.com/ryuz/jelly-fpga-loader.git jelly-fpga-loader
```

## Usage

### Simple Bitstream Download

```bash
jelly-fpga-loader bitdownload <bitstream file> --ip <FPGA server IP address>
```

This command writes the specified bitstream file to the FPGA.

If the --ip option is not specified, it connects to localhost (127.0.0.1:8051).


### Applying DeviceTree Overlay

```bash
jelly-fpga-loader overlay <dtbo file> --bit <bitstream file> --ip <FPGA server IP address>
```

or

```bash
jelly-fpga-loader overlay <dtbo file> --bin <bin file> --ip <FPGA server IP address>
```

These commands transfer the specified bitstream file to /lib/firmware and then apply the DeviceTree Overlay.

After transfer, the bitstream is converted to a bin file. If named xxxx.bit, it will be converted and handled as xxxx.bit.bin. When specified with the --bin option, it is transferred as-is.

When the DeviceTree references a bitstream, you can transfer them together by specifying with the --bit or --bin options.

If the extension of the file specified in <dtbo file> is .dtbo or .dtb, it is transferred as-is. If it's .dts, it is compiled internally by calling dtc before transfer.

If the --ip option is not specified, it connects to localhost (127.0.0.1:8051).


### Registering Accelerators

Register accelerator packages when using DFX (Dynamic Function eXchange) functionality.

```bash
jelly-fpga-loader register-accel <accel name> <dtbo file> <bitstream file> --json <json file> --ip <FPGA server IP address>
```

This registers accelerator packages for use with Xilinx's xmutil command or dfx-mgr-client command.

Each file is individually transferred to firmware before registration.

For <bitstream file>, if the extension is .bin, it is transferred as-is; if .bit, it is converted to .bin before transfer.

If the extension of the file specified in <dtbo file> is .dtbo or .dtb, it is transferred as-is. If it's .dts, it is compiled internally by calling dtc before transfer.

### Unregistering Accelerators

```bash
jelly-fpga-loader unregister-accel <accel name> --ip <FPGA server IP address>
```

This command unregisters the specified accelerator package.

If the --ip option is not specified, it connects to localhost (127.0.0.1:8051).

### Loading Accelerators

```bash
jelly-fpga-loader load <accel name> --ip <FPGA server IP address>
```

This command loads the specified accelerator package.

If the --ip option is not specified, it connects to localhost (127.0.0.1:8051).

### Unloading Accelerators

```bash
jelly-fpga-loader unload <slot> --ip <FPGA server IP address>
```

This command unloads the accelerator package from the specified slot.

If the slot number is omitted, it unloads from slot 0.

If the --ip option is not specified, it connects to localhost (127.0.0.1:8051).


### Converting DTS Files to DTB Files

```bash
jelly-fpga-loader dts2dtbo <dts file> <dtbo file>
```

This command sends the dts file to the server and converts it to a dtbo file.

If the --ip option is not specified, it connects to localhost (127.0.0.1:8051).