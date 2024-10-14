const isProd = process.env.NODE_ENV === 'production';

const internalHost = process.env.TAURI_DEV_HOST || 'localhost';

/** @type {import('next').NextConfig} */
const nextConfig = {
    output: "export",
    experimental: {
        optimizePackageImports: ['@mantine/core', '@mantine/hooks'],
    },
    images: {
        unoptimized: true
    },
    // Configure assetPrefix or else the server won't properly resolve your assets.
    assetPrefix: isProd ? null : `http://${internalHost}:3000`,
};

export default nextConfig;
