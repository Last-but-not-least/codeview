#!/bin/bash
# Script to commit and push the playground

cd "$(dirname "$0")"

echo "Fixing git permissions..."
sudo chown -R $(whoami):$(whoami) .git || {
    echo "Error: Cannot fix permissions. Please run this script with appropriate permissions."
    echo "Or commit manually:"
    echo "  cd $(pwd)"
    echo "  git add .github/workflows/pages.yml playground/"
    echo "  git commit -m 'Add web playground and GitHub Pages deployment'"
    echo "  git push origin main"
    exit 1
}

echo "Adding files to git..."
git add .github/workflows/pages.yml playground/ DEPLOY_INSTRUCTIONS.md commit-playground.sh

echo "Committing..."
git commit -m "Add web playground and GitHub Pages deployment

- Built with Vite + vanilla JavaScript (no build deps)
- Simulates codeview output for Rust, TypeScript, Python, JavaScript
- GitHub Actions workflow for automatic deployment
- Clean, dark-themed UI inspired by Dieter Rams
- Total size: ~22 KB (7.5 KB gzipped)

The playground lets users try codeview in their browser before installing."

echo "Pushing to origin/main..."
git push origin main

echo ""
echo "✓ Done! The GitHub Actions workflow will automatically deploy to GitHub Pages."
echo "  View workflow: https://github.com/Last-but-not-least/codeview/actions"
echo "  Once deployed: https://last-but-not-least.github.io/codeview/"
