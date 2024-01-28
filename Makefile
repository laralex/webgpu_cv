PORT?=8081
HTTP_SERVER_ROOT?=src/minimal_http_server

.PHONY: wasm
wasm:
	wasm-pack build -d build/wasm

.PHONY: kill_server
kill_server:
	(lsof -t -i :8081 -s TCP:LISTEN | xargs kill -9) || true

.PHONY: server_js
server_js: kill_server
	cd ${HTTP_SERVER_ROOT} && node server.js ${PORT} &

.PHONY: server_py
server_py: kill_server
	cd ${HTTP_SERVER_ROOT} && python3 -m http.server ${PORT} &

.PHONY: app
app: wasm server_js
	cd www && npm run start