$DEBUG_BUILD = ./target/debug/sailboat

all: $(DEBUG_BUILD)
	cargo run

.PHONY: $(DEBUG_BUILD)
$(DEBUG_BUILD):
	cargo build

.PHONY: test
test:
	@./test/integration/run-integration-tests.sh
