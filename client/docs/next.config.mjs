import { createMDX } from 'fumadocs-mdx/next';

const withMDX = createMDX();

const isGithubPages = process.env.GITHUB_ACTIONS === 'true';

/** @type {import('next').NextConfig} */
const config = {
  reactStrictMode: true,
  // Enable static export for GitHub Pages
  output: isGithubPages ? 'export' : undefined,
  // Set the base path to the repository name for GitHub Pages
  basePath: isGithubPages ? '/ForkFit' : undefined,
  // Disable image optimization for static exports
  images: {
    unoptimized: true,
  },
};

export default withMDX(config);
