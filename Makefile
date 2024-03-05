DEBUG_BUILD = ./target/debug/sailboat
# WATCH_FLAGS = --ignore 'src/templates/'

all: $(DEBUG_BUILD)
	cargo watch $(WATCH_FLAGS) -x run

.PHONY: $(DEBUG_BUILD)
$(DEBUG_BUILD):
	cargo build

.PHONY: test
test:
	cargo test
	@./test/integration/run-integration-tests.sh

.PHONY: wtest
wtest:
	cargo watch $(WATCH_FLAGS) -s 'cargo test && ./test/integration/run-integration-tests.sh'
