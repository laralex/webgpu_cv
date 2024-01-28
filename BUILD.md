1. install nodejs, npm, rustc, cargo
1. install wasm-pack
   ```
   > curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```
1. build wasm
   ```
   wasm-pack build -d build/wasm
   ```

1. create js project
   ```
   npm init wasm-app www
   ```
1. modify www/index.js to import from your wasm generated .js file
1. modify www/package.json to add dependency on your wasm generated folder (build/wasm)
1. build js project
   ```
   cd www
   npm install
   ```
1. start js project
   ```
   npm run start
   ```