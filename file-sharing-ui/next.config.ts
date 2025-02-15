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
  webpack: (config, options) => {
    const customConfig = config as any;

    customConfig.module.rules.push({
      test: /\.svg$/,
      use: ["@svgr/webpack"],
    });
    return customConfig;
  },

  // distDir: "dist",
  output: 'export',
  env: {
    FILE_SERVER_API: process.env.FILE_SERVER_API || "/",
  },
};

export default nextConfig;
