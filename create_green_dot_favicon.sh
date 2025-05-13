#!/bin/bash

# Create a 32x32 transparent image with a green dot
convert -size 32x32 xc:none -fill "#00FF00" -draw "circle 16,16 16,8" favicon.png

# Convert to .ico format with multiple sizes
convert favicon.png -define icon:auto-resize=16,24,32,48,64 favicon.ico

# Clean up temporary file
rm favicon.png

echo "Green dot favicon.ico created successfully!"
