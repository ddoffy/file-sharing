import type { NextConfig } from "next";
import * as dotenv from "dotenv";
import * as fs from "fs";
import * as path from "path";

// Determine the environment from a custom variable or NODE_ENV
const envFile = process.env.NEXT_ENV_FILE || ".env";
const envPath = path.resolve(process.cwd(), envFile);

console.log("Loading environment from: ", envPath);

if (fs.existsSync(envPath)) {
  dotenv.config({ path: envPath });
}

const nextConfig: NextConfig = {
  /* config options here */
  webpack: (config, { isServer, dev }) => {
    config.module.rules.push({
      test: /\.svg$/,
      use: ["@svgr/webpack"],
    });

    if (!dev && !isServer) {
      config.optimization.minimizer.forEach((minimizer) => {
        if (minimizer.constructor.name === "TerserPlugin") {
          // Enable Terser option to drop console statements
          minimizer.options.terserOptions.compress.drop_console = true;
        }
      });
    }

    return config;
  },

  // distDir: "dist",
  output: "export",
  swcMinify: true, // Enable SWC minification
  compiler: {
    // Remove all console.* calls in production
    removeConsole: true,
  },
  env: {
    FILE_SERVER_API: process.env.FILE_SERVER_API || "/",
  },
};

export default nextConfig;
