const path = require('path');
const MiniCssExtractPlugin = require("mini-css-extract-plugin");

module.exports = {
    mode: "development",
    plugins: [
        new MiniCssExtractPlugin({
            filename: "[name].css",
        }),
    ],
    module: {
        rules: [
            {
                test: /\.m?js$/,
                exclude: /(node_modules)/,
                use: {
                    // We can't reply on swcpack yet but we can still use swc
                    // `.swcrc` can be used to configure swc
                    loader: "swc-loader"
                }
            },
            {
                test: /\.s[ac]ss$/i,
                use: [
                    // Make sure CSS files are CSS files
                    MiniCssExtractPlugin.loader,
                    // Translates CSS into CommonJS
                    "css-loader",
                    // Compiles Sass to CSS
                    "sass-loader",
                ],
            },
        ],
    },
    entry: {
        main: path.resolve(__dirname, './resources/js/chat.js'),
        style: path.resolve(__dirname, './resources/css/main.scss'),
    },
    output: {
        path: path.resolve(__dirname, './public/assets'),
        filename: '[name].js',
    },
    // Turn off if unsafe-eval errors are thrown.
    devtool: false
};