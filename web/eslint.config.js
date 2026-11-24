import js from "@eslint/js";
import babelParser from "@babel/eslint-parser";
import jsxA11y from "eslint-plugin-jsx-a11y";
import reactHooks from "eslint-plugin-react-hooks";
import globals from "globals";

function babel(plugins) {
  return {
    requireConfigFile: false,
    babelOptions: { babelrc: false, configFile: false, parserOpts: { plugins } },
  };
}

// TypeScript 7 ships no JavaScript compiler API, so typescript-eslint cannot load here.
// Babel parses the TypeScript syntax; `tsc --noEmit` stays the type and unused-symbol check.
export default [
  { ignores: ["dist/**", "node_modules/**", "playwright-report/**", "test-results/**"] },
  {
    files: ["**/*.{ts,tsx,js,mjs}"],
    languageOptions: {
      parser: babelParser,
      parserOptions: babel(["typescript"]),
      ecmaVersion: "latest",
      sourceType: "module",
      globals: { ...globals.browser },
    },
    linterOptions: { reportUnusedDisableDirectives: "error" },
    plugins: { "react-hooks": reactHooks, "jsx-a11y": jsxA11y },
    rules: {
      ...js.configs.recommended.rules,
      ...reactHooks.configs.recommended.rules,
      ...jsxA11y.flatConfigs.recommended.rules,
      // Types and unused symbols are the compiler's job; the core rules misread TypeScript.
      "no-undef": "off",
      "no-unused-vars": "off",
      "no-redeclare": "off",
    },
  },
  {
    files: ["**/*.tsx"],
    languageOptions: { parserOptions: babel(["typescript", "jsx"]) },
  },
  {
    files: ["*.config.{js,ts,mjs}", "scripts/**/*.mjs", "tests/**/*.{ts,tsx}"],
    languageOptions: { globals: { ...globals.node } },
  },
];
