from scripts.shellcalls import *

raw_call('curl https://sh.rustup.rs -sSf | sh')
call('rustup target add thumbv7m-none-eabi')
call('cargo install cargo-binutils')
call('rustup component add llvm-tools-preview')
install('gdb-multiarch openocd qemu-system-arm')
call(
    'sudo echo > /etc/udev/rules.d/70-st-link.rules\
# STM32F3DISCOVERY rev A/B - ST-LINK/V2\\\
ATTRS{idVendor}=="0483", ATTRS{idProduct}=="3748", TAG+="uaccess"\\\
\\\
# STM32F3DISCOVERY rev C+ - ST-LINK/V2-1\\\
ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374b", TAG+="uaccess"\\'
)
call('sudo udevadm control --reload-rule')
