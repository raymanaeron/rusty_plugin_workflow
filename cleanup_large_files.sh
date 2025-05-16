#!/bin/bash
set -e

echo "This script will remove the large binary files from Git history."
echo "WARNING: This will rewrite Git history. If you've shared this repository"
echo "with others, they will need to re-clone or handle the rebase carefully."
echo
echo "Press Ctrl+C to cancel or Enter to continue..."
read

# Create a temporary backup branch
git branch backup-before-cleanup

# Use BFG Repo Cleaner if available, otherwise use filter-branch
if command -v bfg &> /dev/null; then
    echo "Using BFG Repo Cleaner to remove large files..."
    bfg --delete-files "*.a" --no-blob-protection
else
    echo "BFG not found, using git filter-branch (slower)..."
    git filter-branch --force --index-filter \
        'git rm --cached --ignore-unmatch engine_ios_ui/build/device/librusty_plugin.a engine_ios_ui/build/simulator/librusty_plugin.a' \
        --prune-empty --tag-name-filter cat -- --all
fi

# Aggressive garbage collection
echo "Running git garbage collection..."
git reflog expire --expire=now --all
git gc --prune=now --aggressive

echo "Large files have been removed from Git history."
echo "To push these changes to remote, use:"
echo "  git push origin --force"
echo
echo "A backup branch 'backup-before-cleanup' has been created."
