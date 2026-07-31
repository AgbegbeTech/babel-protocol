import js from "@eslint/js";
import globals from "globals";

export default [
  { ignores: ["dist/**", "node_modules/**"] },
  {
    ...js.configs.recommended,
    files: ["*.js"],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.node,
      sourceType: "module",
    },
  },
];
