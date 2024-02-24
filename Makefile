build:
	. ~/export-esp.sh && \
		cargo build

flash:
	espflash flash target/jazagotchi/debug/jazagotchi

flash-monitor:
	espflash flash --monitor target/jazagotchi/debug/jazagotchi

