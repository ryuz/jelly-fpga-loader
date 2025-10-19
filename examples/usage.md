# Usage Examples

## Building

```bash
cargo build --release
```

## Examples

### Bitstream Download
```bash
# Download bitstream to local server
./target/release/jelly-fpga-loader bitdownload kv260_sample.bit

# Download bitstream to remote server
./target/release/jelly-fpga-loader bitdownload kv260_sample.bit -ip 192.168.1.100:8051
```

### DeviceTree Overlay
```bash
# Apply DTBO file
./target/release/jelly-fpga-loader overlay sample.dtbo

# Apply DTBO with bitstream
./target/release/jelly-fpga-loader overlay sample.dtbo -b kv260_sample.bit

# Convert DTS and apply
./target/release/jelly-fpga-loader overlay sample.dts -b kv260_sample.bit
```

### Accelerator Management
```bash
# Register accelerator
./target/release/jelly-fpga-loader register-accel my_accel bitstream.bit overlay.dtbo -j config.json

# Load accelerator
./target/release/jelly-fpga-loader load my_accel

# Unload accelerator
./target/release/jelly-fpga-loader unload 0

# Unregister accelerator
./target/release/jelly-fpga-loader unregister-accel my_accel
```