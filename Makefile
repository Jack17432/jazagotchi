build:
	cargo build

flash:
	espflash flash target/jazagotchi/debug/jazagotchi

flash-monitor:
	espflash flash --monitor target/jazagotchi/debug/jazagotchi

docs:
	cargo doc --open --document-private-items --workspace --all-features