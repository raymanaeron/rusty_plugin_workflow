#!/bin/bash

echo "Deleting root target folder..."

if [ -d "target" ]; then
  rm -rf target
fi

if [ -d "target" ]; then
  echo "Failed to delete target folder."
else
  echo "Target folder successfully deleted."
fi
