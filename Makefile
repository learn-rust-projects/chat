build:
	@cargo build

test:
	@cargo nextest run --all-features

release:
	@cargo release commit --execute
	@cargo release tag --execute
	@cargo release push --execute

update-submodule:
	@git submodule update --init --recursive --remote

docker:
	@cd docker/postgresql && docker compose up -d

start: docker
	@echo "Starting chat-server and notify-server..."
	@CHAT_CONFIG=chat-server/chat.yml cargo run -p chat-server &
	@NOTIFY_CONFIG=notify-server/notify.yml cargo run -p notify-server &
	@wait

stop:
	@cd docker/postgresql && docker compose down
	@pkill -f chat-server || true
	@pkill -f notify-server || true

.PHONY: build test release update-submodule docker start stop
