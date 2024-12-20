lint:  ## cargo check and clippy. Skip clippy on guest code since it's not supported by risc0
	cargo +nightly fmt --all --check
	cargo check --all-targets --all-features
	cargo clippy --all-targets --all-features

test:
	cargo test  --all-features --all-targets

check-features: ## Checks that project compiles with all combinations of features.
	cargo hack check --workspace --feature-powerset --all-targets