# Website

This website is built using [Docusaurus](https://docusaurus.io/) and deployed to GitHub Pages.

## Installation

```bash
npm install
```

## Local Development

```bash
npm start
```

This starts a local development server and opens a browser window. Most changes are reflected live without restarting the server.

## Build

```bash
npm run build
```

This generates static content into the `build` directory.

## Deployment

Deployment is handled automatically by GitHub Actions on merge to `main` (when files in `website/` change). See `.github/workflows/deploy-docs.yml`.
