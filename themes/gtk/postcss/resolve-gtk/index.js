const { color, serializeRGB, SyntaxFlag } = require('@csstools/css-color-parser');
const { tokenize } = require('@csstools/css-tokenizer');
const { replaceComponentValues, stringify, parseCommaSeparatedListOfComponentValues, isFunctionNode } = require('@csstools/css-parser-algorithms');

const COLOR_FUNCTION_REGEX = /\bcolor\(/i;
const COLOR_NAME_REGEX = /^color$/i;

function transformColor(/** @type {import('postcss').Declaration} */ decl) {
    const originalValue = decl.value;
    if (!(COLOR_FUNCTION_REGEX.test(originalValue))) {
        return;
    }

    const tokens = tokenize({ css: originalValue });
    const replacedRGB = replaceComponentValues(
        parseCommaSeparatedListOfComponentValues(tokens),
        (componentValue) => {
            if (!isFunctionNode(componentValue) || !COLOR_NAME_REGEX.test(componentValue.getName())) {
                return;
            }

            const colorData = color(componentValue);
            if (!colorData) {
                return;
            }

            if (
                colorData.syntaxFlags.has(SyntaxFlag.Experimental) ||
                colorData.syntaxFlags.has(SyntaxFlag.HasNoneKeywords) ||
                colorData.syntaxFlags.has(SyntaxFlag.RelativeColorSyntax)
            ) {
                return;
            }

            return serializeRGB(colorData);
        },
    );

    const modified = stringify(replacedRGB);
    if (modified === originalValue) {
        return;
    }

    decl.cloneBefore({ value: modified });
    decl.remove();
}

const COLOR_MIX_FUNCTION_REGEX = /\bcolor-mix\(/i;
const COLOR_MIX_NAME_REGEX = /^color-mix$/i;

function transformColorMix(/** @type {import('postcss').Declaration} */ decl) {
    const originalValue = decl.value;
    if (!(COLOR_MIX_FUNCTION_REGEX.test(originalValue))) {
        return;
    }

    const tokens = tokenize({ css: originalValue });
    const replacedRGB = replaceComponentValues(
        parseCommaSeparatedListOfComponentValues(tokens),
        (componentValue) => {
            if (!isFunctionNode(componentValue) || !COLOR_MIX_NAME_REGEX.test(componentValue.getName())) {
                return;
            }

            const colorData = color(componentValue);
            if (!colorData) {
                return;
            }

            if (colorData.syntaxFlags.has(SyntaxFlag.Experimental)) {
                return;
            }

            if (colorData.syntaxFlags.has(SyntaxFlag.ColorMixVariadic)) {
                return;
            }

            return serializeRGB(colorData);
        },
    );

    const modifiedRGB = stringify(replacedRGB);
    if (modifiedRGB === originalValue) {
        return;
    }

    decl.cloneBefore({ value: modifiedRGB });
    decl.remove();
}

const OKLAB_OKLCH_FUNCTION_REGEX = /\b(?:oklab|oklch)\(/i;
const OKLAB_OKLCH_NAME_REGEX = /^(?:oklab|oklch)$/i;

function transformOklab(/** @type {import('postcss').Declaration} */ decl) {
    const originalValue = decl.value;
    if (!(OKLAB_OKLCH_FUNCTION_REGEX.test(originalValue))) {
        return;
    }

    const tokens = tokenize({ css: originalValue });
    const replacedRGB = replaceComponentValues(
        parseCommaSeparatedListOfComponentValues(tokens),
        (componentValue) => {
            if (!isFunctionNode(componentValue) || !OKLAB_OKLCH_NAME_REGEX.test(componentValue.getName())) {
                return;
            }

            const colorData = color(componentValue);
            if(!colorData) {
                return;
            }

            // if (
            //     colorData.syntaxFlags.has(SyntaxFlag.Experimental) ||
            //     colorData.syntaxFlags.has(SyntaxFlag.HasNoneKeywords) ||
            //     colorData.syntaxFlags.has(SyntaxFlag.RelativeColorSyntax)
            // ) {
            //     console.log(`Unable to parse color: ${componentValue.toString()}`);
            //     return;
            // }

            return serializeRGB(colorData);
        },
    );

    const modifiedRGB = stringify(replacedRGB);
    if (modifiedRGB === originalValue) {
        return;
    }

    decl.cloneBefore({ value: modifiedRGB });
    decl.remove();
}

function miscGtkTransformations(/** @type {import('postcss').Declaration} */ decl) {
    // Gtk3 doesn't support percentages for certain properties (???) so convert them to decimals
    const percentageFixProperties = ["opacity"];

    if (percentageFixProperties.includes(decl.prop)) {
        decl.value = decl.value.replaceAll(/(\d*\.?\d+)%/g, (match, num) => {
            const decimal = parseFloat(num) / 100;
            return decimal.toString();
        });
    }
}

function resolveGtkColors(/** @type {import('postcss').Root} */ root) {
    // Record top-level @define-color statements
    const colors = new Map();

    // Define accent colors, which stylesheets assume GTK defines and can't be collapsed otherwise
    colors.set("accent_fg_color", "white");
    // This is the default accent color, which unfortunately means we give up accents. Meh.
    colors.set("accent_bg_color", "#3584e4");

    root.walkAtRules("define-color", (rule) => {
        if (rule.params) {
            const params = rule.params.trim().split(" ");
            const name = params[0];
            if (name) {
                colors.set(name, params.slice(1).join(" "));
            }

            // Problematically, define-color calls can also include unsupported features like oklab colors.
            // If we find them, just remove it. Meh.
            const unsupportedHints = [
                /oklab/i,
                /color-mix/i,
                /* like rgb(_ _ _ / __%) */
                /rgb\(\s*[\d.]+\s*[\d.]+\s*[\d.]+\s*\/\s*[\d.]+%\s*\)/i,
            ];
            for (const hint of unsupportedHints) {
                if (hint.test(rule.params)) {
                    console.warn(`Warning: Unsupported color definition found: ${rule.params}`);
                    rule.remove();
                    return;
                }
            }
        }
    });
    // Color definition themselves can be in color rules, so iterate until all colors are resolved
    // Color definition references look like @color
    let changed;
    do {
        changed = false;
        for (const [name, value] of colors.entries()) {
            let match = value.match(/@([a-zA-Z0-9_-]+)/);
            if (match) {
                const refName = match[1];
                if (colors.has(refName)) {
                    // Replace the reference with the actual color value
                    const newValue = value.replace(`@${refName}`, colors.get(refName));
                    if (newValue !== value) {
                        colors.set(name, newValue);
                        changed = true;
                    }
                } else {
                    console.warn(`Warning: Color reference @${refName} not defined.`);
                }
            }
        }
    } while (changed);

    // Replace color definitions in the stylesheet
    let locations = 0;
    root.walkDecls(decl => {
        if (decl.value) {
            for (const [name, value] of colors.entries()) {
                // Replace @color references with actual color values
                const regex = new RegExp(`@${name}`, 'g');
                while(regex.test(decl.value)) {
                    decl.value = decl.value.replace(regex, value);
                    locations++;
                }
            }
        }
    });

    console.log(`Resolved ${colors.size} GTK colors in ${locations} locations`);
}

function resolveVars(/** @type {import('postcss').Root} */ root) {
    // Alright, and the even hackier part now! To improve folding, assume `currentColor` always refers to white...
    root.walkDecls(decl => {
        decl.value = decl.value.replaceAll(/currentColor/g, "white");
        decl.cloneBefore({ value: decl.value });
        decl.remove();
    });

    // Then, a much more permissive version of postcss-custom-properties that removes _all_ custom properties,
    // no matter if it actually makes sense to
    const customProperties = new Map();

    // A few of our own definitions for small edge cases
    customProperties.set("--monospace-font-family", "monospace");
    customProperties.set("--monospace-font-size", "1em");

    root.walkDecls(decl => {
        if (decl.prop.startsWith("--")) {
            customProperties.set(decl.prop, decl.value);
            decl.remove();
        }
    });

    // Variables can also have variables in them... so do the same thing.
    let changed;
    do {
        changed = false;
        for (const [name, value] of customProperties.entries()) {
            let match = value.match(/var\((--[a-zA-Z0-9_-]+)\)/);
            if (match) {
                const refName = match[1];
                if (customProperties.has(refName)) {
                    const newValue = value.replace(`var(${refName})`, customProperties.get(refName));
                    if (newValue !== value) {
                        customProperties.set(name, newValue);
                        changed = true;
                    }
                } else {
                    console.warn(`Warning: Variable reference ${refName} not defined.`);
                }
            }
        }
    } while (changed);

    root.walkDecls(decl => {
        // Replace var(--variable, default) with the variable or the default.
        // If no default is provided, we remove the node lol

        const regex = /var\((--[a-zA-Z0-9_-]+)(?:, ([^)]*))?\)/g;
        if (!regex.test(decl.value)) {
            return;
        }

        let shouldRemove = false;
        decl.value = decl.value.replaceAll(regex, (match, variable, defaultValue) => {
            if (customProperties.has(variable)) {
                return customProperties.get(variable);
            } else if (defaultValue) {
                return defaultValue;
            } else {
                shouldRemove = true;
                return match;
            }
        });
        
        if(shouldRemove) {
            console.warn(`Removed declaration with unresolved variable: ${decl.toString()}`);
        } else {
            decl.cloneBefore({ value: decl.value });
        }

        decl.remove();
    });
}

function removeInvalidProperties(/** @type {import('postcss').Root} */ root) {
    // Remove invalid properties
    const invalidProperties = ['filter', 'border-spacing', 'transform', '-gtk-icon-size', 'line-height'];
    let invalidPropertiesRemoved = 0;
    root.walkDecls(decl => {
        if (invalidProperties.includes(decl.prop)) {
            decl.remove();
            invalidPropertiesRemoved++;
        }
    });

    console.log(`Removed ${invalidPropertiesRemoved} invalid properties`);
}

function removeEmptySelectors(/** @type {import('postcss').Root} */ root) {
    // Remove empty selectors
    let emptySelectorsRemoved = 0;
    root.walkRules(rule => {
        if (rule.nodes.length === 0) {
            rule.remove();
            emptySelectorsRemoved++;
        }
    });

    console.log(`Removed ${emptySelectorsRemoved} empty selectors`);
}

/**
 * @type {import('postcss').PluginCreator}
 */
module.exports = (opts = {}) => {
  return {
    postcssPlugin: "resolve-gtk3-colors",
    
    Root (root, postcss) {
        resolveGtkColors(root);
        resolveVars(root);
        removeInvalidProperties(root);

        root.walkDecls(decl => transformColor(decl));
        root.walkDecls(decl => transformColorMix(decl));
        root.walkDecls(decl => transformOklab(decl));
        root.walkDecls(decl => miscGtkTransformations(decl));

        removeEmptySelectors(root);
    }
  }
}

module.exports.postcss = true