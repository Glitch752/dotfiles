const resolveGtk = require('resolve-gtk');

/** @type {import('postcss-load-config').Config} */
module.exports = {
  plugins: [
    resolveGtk({}),
    require('postcss-color-functional-notation')({}),
    require('@csstools/postcss-relative-color-syntax')({})
  ],
};
