install-rust-components:
	rustup toolchain install nightly-2023-09-30 --component rust-src,rustc-dev,llvm-tools

install-dev-deps:
	cargo install \
		just \
		cargo-generate-rpm \
		cross

generate-rpm-x86:
	cargo generate-rpm
