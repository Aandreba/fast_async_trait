publish:
	cd proc && cargo check
	cargo check
	cargo check --test main
	cd proc && cargo publish
	cargo publish