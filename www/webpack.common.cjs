const path = require('path');
const CopyWebpackPlugin = require('copy-webpack-plugin');

module.exports = {
  entry: {
    app: './index.js',
    //adminApp: {}, './bootstrap.js'
  },
  plugins: [
    new CopyWebpackPlugin(['index.html'])
  ],
  output: {
    filename: "bundle.js",
    path: path.resolve(__dirname, 'dist'),
  },
  experiments: {
    asyncWebAssembly: true,
    syncWebAssembly: true,
  },
  module: {
    rules: [
        // {
        //     test: /\.ts$/,
        //     loader: 'ts-loader',
        //     options: {
        //         configFile: 'tsconfig.json',
        //     },
        // },
        {
            test: /\.wasm$/,
            type: "webassembly/async",
        },
    ],
},
};
