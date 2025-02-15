build-server:
	@echo "Building server"
	cargo build --release
	@echo "Server built successfully"

build-client:
	@echo "Building client"
	cd file-sharing-ui && npm install && npm run build
	@echo "Client built successfully"

build: build-server build-client
