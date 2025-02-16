# Use the official Node.js 22 image for ARM64 (Jetson)
FROM arm64v8/node:22 AS builder

# Set working directory
WORKDIR /app

# Copy package.json and package-lock.json first (for better caching)
COPY package.json package-lock.json ./

# Install dependencies
RUN npm install --frozen-lockfile

# Copy the rest of the application files
COPY . .

# Build the Next.js application
RUN npm run build

# -- Production Stage --
FROM arm64v8/node:22 AS runner

# Set working directory
WORKDIR /app

# Copy built files from the builder stage
COPY --from=builder /app ./

# Expose the port Next.js runs on
EXPOSE 3000

# Start Next.js in production mode
CMD ["npm", "run", "start"]
