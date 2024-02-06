#!/bin/bash
#git clone https://github.com/wzab/vhost-device.git -b gpio-python
export PATH_TO_LIBGPIOD=/home/dev/libgpiod-2.1
export SYSTEM_DEPS_LIBGPIOD_NO_PKG_CONFIG=1
export SYSTEM_DEPS_LIBGPIOD_SEARCH_NATIVE="${PATH_TO_LIBGPIOD}/lib/.libs/"
export SYSTEM_DEPS_LIBGPIOD_LIB=gpiod
export SYSTEM_DEPS_LIBGPIOD_INCLUDE="${PATH_TO_LIBGPIOD}/include/"
(
  cd vhost-device/vhost-device-gpio
  cargo build --features "mock_gpio"
  cd py
  python3 -m venv gpio
  (
    source gpio/bin/activate
    pip install tinyrpc gevent pgi werkzeug
  )
)
xfce4-terminal -H -x ./rungui &
# Wait some time so that GUI starts.
sleep 3
cd vhost-device/target/debug
LD_LIBRARY_PATH=${PATH_TO_LIBGPIOD}/lib/.libs ./vhost-device-gpio -l s1 -s /tmp/gpio.sock
