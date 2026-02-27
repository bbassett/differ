.PHONY: dev test check

dev:
	npm run tauri dev

test:
	cd src-tauri && cargo test

check:
	cd src-tauri && cargo check
	npm run build
