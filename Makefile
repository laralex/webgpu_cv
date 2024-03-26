PORT?=8081
WASM_NAME?=my_wasm
RUST_TARGET?=wasm32-unknown-unknown
SERVE_DIR?=www
SERVE_WASM_DIR?=${SERVE_DIR}/wasm

.PHONY: install
install:
	cargo install -f wasm-bindgen-cli
	cargo install wasm-opt --locked

.PHONY: wasm_debug
wasm_debug:
	cargo build --target=${RUST_TARGET} --config package.name=\"${WASM_NAME}\"
	wasm-bindgen --target=web --omit-default-module-path \
		target/${RUST_TARGET}/debug/${WASM_NAME}.wasm \
		--out-dir ${SERVE_WASM_DIR} --out-name index

.PHONY: wasm
wasm:
	cargo build --release --target=${RUST_TARGET} --config package.name=\"${WASM_NAME}\"
	wasm-bindgen --target=web --omit-default-module-path \
		target/${RUST_TARGET}/release/${WASM_NAME}.wasm \
		--out-dir ${SERVE_WASM_DIR} --out-name index

.PHONY: wasm_opt
wasm_opt:
	wasm-opt ${SERVE_WASM_DIR}/index_bg.wasm -O2 --dce --output ${SERVE_WASM_DIR}/index_bg.wasm

.PHONY: pdf_link
pdf_link:
	ln $(shell find ${SERVE_DIR}/assets -type f -iname "*eng*.pdf" ! -iname "*softlink*") ${SERVE_DIR}/assets/__softlink_cv_eng.pdf
	ln $(shell find ${SERVE_DIR}/assets -type f -iname "*rus*.pdf" ! -iname "*softlink*") ${SERVE_DIR}/assets/__softlink_cv_rus.pdf

.PHONY: codegen
codegen:
	echo "\
	/* NOTE: BUILD_DATA is automatically generated in Makefile */ \n\
	let BUILD_DATA = \
	{ \n\
   	'git-commit': \"$(shell git rev-parse HEAD)\", \n\
   	'git-commit-date': \"$(shell git show -s --format=%cD)\", \n\
   	'debug': $(if $(filter ${BUILD_TYPE},debug),true,false), \n\
   	'deploy-date': \"$(shell LANG=en_us_88591 date +'%a, %d %b %Y %H:%M:%S %z %Z')\" \n\
	}" > ${SERVE_DIR}/build-data.js

.PHONY: codegen_debug
codegen_debug:
	BUILD_TYPE=debug $(MAKE) codegen

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

.PHONY: build_debug
build_debug: wasm_debug codegen_debug pdf_link

.PHONY: app_debug
app_debug: build_debug server_py

.PHONY: build_ci
build_ci: wasm codegen pdf_link

.PHONY: build
build: build_ci wasm_opt

.PHONY: app
app: build server_py