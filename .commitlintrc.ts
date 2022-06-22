import type { UserConfig } from "@commitlint/types";

const config: UserConfig = {
  extends: ["@commitlint/config-conventional"],

  parserPreset: {
    parserOpts: {
      headerPattern: /^([^\(\):]*)(?:\((.*)\))?!?: (.*)$/,
    },
  },

  rules: {
    "type-enum": [
      2,
      "always",
      [
        "chore",
        "proc-macro",
        "type-challenges",
        "nextjs-blog",
      ]
    ],
  },
};

module.exports = config;
