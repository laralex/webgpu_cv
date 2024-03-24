PORT?=8081
HTTP_SERVER_ROOT?=src/minimal_http_server


.PHONY: install
install:
	cargo install -f wasm-bindgen-cli

.PHONY: wasm
wasm:
	wasm-pack build --target web -d www/wasm --mode no-install
	# cargo build --target=wasm32-unknown-unknown
	# wasm-bindgen --out-dir=www/wasm --target=web --omit-default-module-path my-wasm.wasm

.PHONY: kill_server
kill_server:
	(lsof -t -i :${PORT} -s TCP:LISTEN | xargs kill -9) || true

.PHONY: server_webpack
server_webpack: kill_server
	cd www && \
		npm install && \
		npm run build && \
		DEVPORT=${PORT} npm run start-dev

.PHONY: server_js
server_js: kill_server
	cd ${HTTP_SERVER_ROOT} && node server.js ${PORT} &

.PHONY: server_py
server_py: kill_server
	cd ${HTTP_SERVER_ROOT} && python3 -m http.server ${PORT} &

.PHONY: app
dev_app: wasm server_webpack
	