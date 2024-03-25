PORT?=8081
WASM_NAME?=my_wasm
RUST_TARGET?=wasm32-unknown-unknown
SERVE_DIR?=www
SERVE_WASM_DIR?=${SERVE_DIR}/wasm

.PHONY: install
install:
	cargo install -f wasm-bindgen-cli
	cargo install wasm-opt --locked

.PHONY: wasm
wasm:
	cargo build --profile release --target=${RUST_TARGET} --config package.name=\"${WASM_NAME}\"
	wasm-bindgen --target=web --omit-default-module-path \
		target/${RUST_TARGET}/release/${WASM_NAME}.wasm \
		--out-dir ${SERVE_WASM_DIR} --out-name index
	wasm-opt ${SERVE_WASM_DIR}/index_bg.wasm -O2 --dce --output ${SERVE_WASM_DIR}/index_bg.wasm

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
	cd www && node server.js ${PORT} &

.PHONY: server_py
server_py: kill_server
	cd www && python3 -m http.server ${PORT}

.PHONY: app
dev_app: wasm server_py
