# Embedded Rust
Getting rust to work an the Blue Pill stm32 board


# Steps Done
- connect both Boot Jumper to 0

```
  sudo apt install \
  gdb-multiarch \
  openocd
```

Create file ``/etc/udev/rules.d/70-st-link.rules`` for permissions with this content:
```
# STM32F3DISCOVERY rev A/B - ST-LINK/V2\
ATTRS{idVendor}=="0483", ATTRS{idProduct}=="3748", TAG+="uaccess"\
\
# STM32F3DISCOVERY rev C+ - ST-LINK/V2-1\
ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374b", TAG+="uaccess"\
```

Update:
```
sudo udevadm control --reload-rules
```

## setup rust
Install toolchain
```
rustup target add thumbv7m-none-eabi
```
Install cargo-binutils
```
cargo install cargo-binutils
rustup component add llvm-tools-preview
```
Setup Cargo 
```
cargo init --bin --edition 2018
```
  
Build binary
```
cargo build
```