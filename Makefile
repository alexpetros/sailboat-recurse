$DEBUG_BUILD = ./target/debug/sailboat

all: $(DEBUG_BUILD)
	cargo watch -x run

.PHONY: $(DEBUG_BUILD)
$(DEBUG_BUILD):
	cargo build

.PHONY: test
test:
	cargo build
	@./test/integration/run-integration-tests.sh

.PHONY: test
watch-test:
	cargo watch -s ./test/integration/run-integration-tests.sh
