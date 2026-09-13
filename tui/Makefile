.PHONY: build build-release clean cross-build release strip check

TARGET ?= aarch64-unknown-linux-gnu

build:
	cargo build

build-release:
	cargo build --release

cross-build:
	cross build --release --target $(TARGET)

release: cross-build
	@echo "Binaries:"
	@ls -lh target/$(TARGET)/release/sys-tui \
	         target/$(TARGET)/release/net-tui \
	         target/$(TARGET)/release/loot-tui \
	         target/$(TARGET)/release/scan-tui 2>/dev/null || echo "Build failed"

strip: build-release
	@for bin in sys-tui net-tui loot-tui scan-tui; do \
	    strip target/$(TARGET)/release/$$bin 2>/dev/null && \
	    echo "Stripped: $$bin $$(ls -lh target/$(TARGET)/release/$$bin | awk '{print $$5}')"; \
	done

clean:
	cargo clean

check:
	cargo check --target $(TARGET)
