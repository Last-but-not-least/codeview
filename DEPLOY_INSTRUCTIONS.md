# Deployment Instructions for Playground

The codeview playground has been successfully built and is ready to deploy!

## What's Been Created

### 1. Playground Website (`playground/` directory)
- **Tech stack**: Vite + vanilla JavaScript (no build dependencies needed)
- **Features**:
  - Code editor with syntax highlighting
  - Live output showing codeview-style collapsed/expanded views
  - Language selector (Rust, TypeScript, Python, JavaScript)
  - Example code snippets for each language
  - Toggle between interface and expanded modes
  - Clean, dark-themed UI inspired by Dieter Rams
  - Fully responsive design

### 2. GitHub Actions Workflow (`.github/workflows/pages.yml`)
- Builds the playground using `npm run build` (which uses `npx vite`)
- Deploys to GitHub Pages automatically on push to main
- No dependencies to install - uses npx to run Vite on-demand

## Files Created

```
codeview/
├── .github/
│   └── workflows/
│       └── pages.yml          # GitHub Actions deployment workflow
└── playground/
    ├── .gitignore
    ├── package.json           # Build scripts (no dependencies)
    ├── vite.config.js         # Vite config with base path /codeview/
    ├── index.html             # Main HTML file
    └── src/
        ├── main.js            # Application entry point
        ├── style.css          # Styling
        ├── examples.js        # Example code snippets
        └── parser.js          # Code parser (simulates codeview)

Built output: playground/dist/
```

## Deployment Steps

### Step 1: Commit and Push

Due to git permission constraints in the container, please run these commands **on the host** or **fix .git permissions**:

```bash
cd /path/to/codeview
git add .github/workflows/pages.yml playground/
git commit -m "Add web playground and GitHub Pages deployment"
git push origin main
```

Or fix permissions and commit from container:
```bash
cd /home/node/.openclaw/workspace/codeview
sudo chown -R node:node .git
git add .github/workflows/pages.yml playground/
git commit -m "Add web playground and GitHub Pages deployment"
git push origin main
```

### Step 2: Enable GitHub Pages

1. Go to https://github.com/Last-but-not-least/codeview/settings/pages
2. Under "Build and deployment":
   - **Source**: Deploy from a branch
   - Change to: **GitHub Actions**
3. The workflow will automatically run and deploy

### Step 3: Access the Playground

Once deployed, the playground will be available at:
**https://last-but-not-least.github.io/codeview/**

## Local Development

To test the playground locally:

```bash
cd playground
npm run dev
```

This will start a local dev server (no dependencies to install, uses npx).

## How the Parser Works

The playground includes a simplified JavaScript parser (`src/parser.js`) that mimics codeview's output by:

1. Using regex to identify top-level code structures (functions, classes, structs, impl blocks, etc.)
2. Extracting signatures and collapsing bodies to `{ ... }` or `: ...`
3. Preserving original line numbers
4. Supporting both "interface mode" (collapsed) and "expanded mode" (full source for selected items)

The parser handles language-specific patterns for Rust, TypeScript, Python, and JavaScript.

## Build Verification

The build has been tested successfully:
```
✓ 6 modules transformed
✓ built in 116ms
dist/index.html                  2.34 kB │ gzip: 0.85 kB
dist/assets/index-5UYq5_WS.css   2.50 kB │ gzip: 0.96 kB
dist/assets/index-CTkp12V6.js   16.83 kB │ gzip: 4.70 kB
```

Total size: ~22 KB (7.5 KB gzipped) - lightweight and fast!

## Next Steps

After deployment:
1. Add the playground link to the main README
2. Consider adding a "Try it in your browser" badge
3. Share on social media / Show HN

The playground gives potential users a quick way to understand what codeview does before installing!
