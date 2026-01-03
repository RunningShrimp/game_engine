/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,

  env: {
    NEXT_PUBLIC_API_URL: process.env.NEXT_PUBLIC_API_URL,
  },

  images: {
    domains: [
      'localhost',
      'plugins.gameengine.com',
      'cdn.plugins.gameengine.com',
      's3.amazonaws.com',
    ],
  },

  // Enable experimental features for better performance
  experimental: {
    optimizeCss: true,
  },

  // Generate static pages where possible
  output: 'standalone',

  // Webpack configuration
  webpack: (config, { isServer }) => {
    if (!isServer) {
      config.resolve.fallback = {
        ...config.resolve.fallback,
        fs: false,
      };
    }
    return config;
  },
};

module.exports = nextConfig;
