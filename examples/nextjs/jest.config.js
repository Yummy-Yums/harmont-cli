module.exports = {
  testEnvironment: "jsdom",
  setupFilesAfterEach: ["@testing-library/jest-dom"],
  transform: {
    "^.+\\.(t|j)sx?$": ["babel-jest", { presets: ["next/babel"] }],
  },
};
