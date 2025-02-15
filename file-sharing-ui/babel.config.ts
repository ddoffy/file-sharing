const config = {
  presets: ['next/babel'],
  env: {
    production: {
      plugins: ['transform-remove-console'],
    },
  },
};

export default config;
