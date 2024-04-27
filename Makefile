DEBUG_BUILD = ./target/debug/sailboat
DB_NAME = ./sailboat.db
# WATCH_FLAGS = -i '*.db' -i '*-shm' -i '*-wal'

.PHONY: ngrok
ngrok:
	@./scripts/start-with-ngrok.sh

all: $(DEBUG_BUILD)
	cargo watch $(WATCH_FLAGS) -x run

.PHONY: $(DEBUG_BUILD)
$(DEBUG_BUILD):
	cargo build

.PHONY: release
release:
	cargo run --release

.PHONY: check
check:
	cargo watch $(WATCH_FLAGS) -s 'cargo check'

.PHONY: test
test:
	cargo test
	@./test/integration/run-integration-tests.sh

.PHONY: wtest
wtest:
	cargo watch $(WATCH_FLAGS) -s 'cargo test && ./test/integration/run-integration-tests.sh'

.PHONY: delete-db
delete-db:
	rm -f $(DB_NAME)*

.PHONY: reset-db
reset-db:
	rm -f $(DB_NAME)*
	cat ./src/db/migrations/0-init.sql | sqlite3 $(DB_NAME)

.PHONY: check
check:
	cargo check
