import { defineConfig, globalIgnores } from "eslint/config";
import tseslint from "typescript-eslint";
import nodePlugin from "eslint-plugin-n";

const eslintConfig = defineConfig([
  globalIgnores(["dist/**", "node_modules/**"]),
  ...tseslint.configs.recommended,
  nodePlugin.configs["flat/recommended"],
  {
    rules: {
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
      "@typescript-eslint/no-explicit-any": "warn",
      "n/no-missing-import": "off",
      "n/no-unpublished-import": "off",
    },
  },
]);

export default eslintConfig;
