KERNELDIR ?= /lib/modules/$(shell uname -r)/build
PWD := $(shell pwd)
ARCH := $(shell uname -m)

Target_ko :=

# 多个 dts 目标，空格分隔，不要写 .dts 后缀
Target_dts := cardputerzero-overlay spk-gpio24-high-overlay spi0-spidev2-gpio22-overlay camera-gpio16-high-overlay

Target_dtbo := $(addsuffix .dtbo,$(Target_dts))

# Module name
ifneq ($(strip $(Target_ko)),)
obj-m += $(Target_ko).o
endif

all: $(Target_dtbo)

ifneq ($(strip $(Target_ko)),)
$(Target_ko).ko: $(Target_ko).c
	$(MAKE) -C $(KERNELDIR) M=$(PWD) ARCH=$(ARCH) modules
endif

%.dtbo: %.dts
	dtc -@ -I dts -O dtb -o $@ $<

install_dts: $(Target_dtbo)
	cp $(Target_dtbo) /boot/firmware/overlays/

clean:
	$(MAKE) -C $(KERNELDIR) M=$(PWD) clean
	rm -f $(Target_dtbo)

install: all install_dts
	if [ -d '../py32ioexp-1.0' ] ; then $(MAKE) -C ../py32ioexp-1.0 install ; fi
	if [ -d '../st7789v-1.0' ] ; then $(MAKE) -C ../st7789v-1.0 install ; fi
	if [ -d '../tca8418-1.0' ] ; then $(MAKE) -C ../tca8418-1.0 install ; fi
	if [ -d '../es8389-1.0' ] ; then $(MAKE) -C ../es8389-1.0 install ; fi
	if [ -d '../bq27220-1.0' ] ; then $(MAKE) -C ../bq27220-1.0 install ; fi
	if [ -d '../lsm6ds3tr-1.0' ] ; then $(MAKE) -C ../lsm6ds3tr-1.0 install ; fi
	echo "ok"

.PHONY: all clean install install_dts