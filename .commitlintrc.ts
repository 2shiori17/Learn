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

        "datalog",
        "nextjs-blog",
        "proc-macro",
        "type-challenges",
      ]
    ],
  },
};

module.exports = config;
