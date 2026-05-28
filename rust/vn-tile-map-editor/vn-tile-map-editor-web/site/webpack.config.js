const CopyPlugin = require("copy-webpack-plugin");
const path = require("path");

module.exports = {
    entry: "./index.js",
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "index.js",
        publicPath: "./",
    },
    mode: "development",

    experiments: {
        asyncWebAssembly: true,
    },

    optimization: {
        splitChunks: false,
        runtimeChunk: false,
    },

    resolve: {
        symlinks: false,
    },

    plugins: [
        new CopyPlugin({
            patterns: [
                { from: "index.html" },
                { from: "../../assets", to: "assets" }
            ],
        }),
    ],

    devServer: {
        client: {
            overlay: false,
        }
    }
};
