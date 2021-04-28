target = ./target/debug/cachelab-rs
TEST = ./test-csim

csim:
	@cargo build
	@cp ${target} ./csim

test: csim
	@${TEST}

clean:
	@cargo clean
	@rm -f ./csim
