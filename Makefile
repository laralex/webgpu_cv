PORT?=8081
WASM_NAME?=my_wasm
RUST_TARGET?=wasm32-unknown-unknown
SERVE_DIR?=www
SERVE_WASM_DIR?=${SERVE_DIR}/wasm
CARGO_TOOLCHAIN?=+stable
CARGO_FLAGS?=--target=${RUST_TARGET} --config package.name=\"${WASM_NAME}\"
WASM_BINDGEN_FLAGS?=--target=web --omit-default-module-path --out-dir ${SERVE_WASM_DIR} --out-name index

.PHONY: install
install:
	rustup toolchain install stable-x86_64-unknown-linux-gnu
	rustup $(CARGO_TOOLCHAIN) target add wasm32-unknown-unknown
	cargo install -f wasm-bindgen-cli
	cargo install wasm-opt --locked

.PHONY: wasm_debug
wasm_debug:
	cargo $(CARGO_TOOLCHAIN) build $(CARGO_FLAGS)
	wasm-bindgen --keep-debug $(WASM_BINDGEN_FLAGS) target/${RUST_TARGET}/debug/${WASM_NAME}.wasm 

.PHONY: wasm
wasm:
	cargo $(CARGO_TOOLCHAIN) build --release $(CARGO_FLAGS)
	wasm-bindgen $(WASM_BINDGEN_FLAGS) target/${RUST_TARGET}/release/${WASM_NAME}.wasm

.PHONY: wasm_ci
wasm_ci: CARGO_FLAGS += --jobs ${MAX_THREADS}
wasm_ci: wasm

.PHONY: wasm_opt
wasm_opt:
	wasm-opt ${SERVE_WASM_DIR}/index_bg.wasm -O2 --dce --output ${SERVE_WASM_DIR}/index_bg.wasm

.PHONY: pdf_link
pdf_link:
	ln -sfn $(shell find ${SERVE_DIR}/assets -type f -iname "*eng*.pdf" ! -iname "*softlink*") ${SERVE_DIR}/assets/__softlink_cv_eng.pdf
	ln -sfn $(shell find ${SERVE_DIR}/assets -type f -iname "*rus*.pdf" ! -iname "*softlink*") ${SERVE_DIR}/assets/__softlink_cv_rus.pdf

.PHONY: codegen
codegen:
	echo "\
	{\n\
   	\"git-commit\": \"$(shell git rev-parse HEAD)\",\n\
   	\"git-commit-date\": \"$(shell git show -s --format=%cD)\",\n\
   	\"debug\": $(if $(filter ${BUILD_TYPE},debug),true,false),\n\
   	\"deploy-date\": \"$(shell LANG=en_us_88591 date +'%a, %d %b %Y %H:%M:%S %z %Z')\"\n\
	}" > ${SERVE_DIR}/build-data.json

.PHONY: codegen_debug
codegen_debug:
	BUILD_TYPE=debug $(MAKE) codegen

.PHONY: build_debug
build_debug: wasm_debug codegen_debug pdf_link

# no `wasm_opt`
.PHONY: build_ci
build_ci: wasm_ci codegen pdf_link

.PHONY: build
build: wasm codegen pdf_link wasm_opt

# ===== DEVELOPER DEPLOYMENT
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

.PHONY: app_debug
app_debug: build_debug server_py

.PHONY: app
app: build server_py