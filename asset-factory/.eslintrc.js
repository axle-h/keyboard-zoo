module.exports = {
    env: {
        commonjs: true,
        es6: true,
        node: true,
    },
    globals: {
        Atomics: "readonly",
        SharedArrayBuffer: "readonly",
    },
    parserOptions: {
        ecmaVersion: 11,
    },
    overrides: [
        {
            parser: "@typescript-eslint/parser",
            files: ["*.ts"],
            parserOptions: {
                sourceType: 'module',
                ecmaVersion: 2020,
            },
            plugins: ["@typescript-eslint", "prettier"],
            extends: [
                "plugin:@typescript-eslint/recommended",
                "prettier",
                "prettier/@typescript-eslint"
            ],
            rules: {
                "prettier/prettier": "error",
                "@typescript-eslint/no-return-await": ["off"],
                "@typescript-eslint/no-await-in-loop": ["off"],
                "@typescript-eslint/ban-ts-ignore": ["off"],
                "@typescript-eslint/explicit-function-return-type": ["off"],
                "@typescript-eslint/camelcase": ["off"],
                "@typescript-eslint/interface-name-prefix": ["off"],
                "@typescript-eslint/no-explicit-any": ["off"],
            },
        },
        {
            files: ["*.test.ts", "*fixture.ts", "*Fixture.ts", "fake.ts"],
            rules: {
                "@typescript-eslint/no-empty-function": ["off"],
            },
        }
    ],
};
