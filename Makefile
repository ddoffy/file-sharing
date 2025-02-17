build-server:
	@echo "Building server"
	cargo build --release
	@echo "Server built successfully"

stop-server:
	@echo "Stopping server"
	sudo systemctl stop file-sharing
	@echo "Server stopped successfully"

start-server:
	@echo "Starting server"
	sudo systemctl start file-sharing
	@echo "Server started successfully"

restart-server: stop-server start-server

status-server:
	@echo "Server status"
	sudo systemctl status file-sharing
	@echo "Server status checked"

all: stop-server build-server start-server status-server

.PHONY: stop-server build-server start-server status-server
