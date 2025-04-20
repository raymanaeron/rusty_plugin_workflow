#!/bin/bash

if [ $# -ne 3 ]; then
  echo "Usage: ./create_plugin.sh <plugin_name> <plugin_route> <resource_name>"
  exit 1
fi

PLUGIN_NAME=$1
PLUGIN_ROUTE=$2
RESOURCE_NAME=$3

TEMPLATE_DIR="plugin_templates"
TARGET_DIR="plugins/$PLUGIN_NAME"

echo "Creating plugin: $PLUGIN_NAME (route=$PLUGIN_ROUTE, resource=$RESOURCE_NAME)"
mkdir -p "$TARGET_DIR/src"
mkdir -p "$TARGET_DIR/web"

# Process Cargo.toml
sed -e "s/{{plugin_name}}/$PLUGIN_NAME/g" \
    "$TEMPLATE_DIR/Cargo.toml.template" > "$TARGET_DIR/Cargo.toml"

# Process lib.rs
sed -e "s/{{plugin_name}}/$PLUGIN_NAME/g" \
    -e "s/{{plugin_route}}/$PLUGIN_ROUTE/g" \
    -e "s/{{resource_name}}/$RESOURCE_NAME/g" \
    "$TEMPLATE_DIR/lib.rs.template" > "$TARGET_DIR/src/lib.rs"

# Process HTML
sed -e "s/{{plugin_route}}/$PLUGIN_ROUTE/g" \
    "$TEMPLATE_DIR/step-template.html" > "$TARGET_DIR/web/step-$PLUGIN_ROUTE.html"

# Process JS
sed -e "s/{{plugin_route}}/$PLUGIN_ROUTE/g" \
    -e "s/{{resource_name}}/$RESOURCE_NAME/g" \
    "$TEMPLATE_DIR/step-template.js" > "$TARGET_DIR/web/step-$PLUGIN_ROUTE.js"

# README
cat <<EOF > "$TARGET_DIR/README.md"
# Plugin: $PLUGIN_NAME

Route: \`$PLUGIN_ROUTE\`  
Resource: \`$RESOURCE_NAME\`

## File Structure

\`\`\`
plugins/
└── $PLUGIN_NAME/
    ├── src/
    │   └── lib.rs
    ├── web/
    │   ├── step-$PLUGIN_ROUTE.html
    │   └── step-$PLUGIN_ROUTE.js
    └── Cargo.toml
\`\`\`

## Core Plugin Usage

To load this plugin in your engine/lib.rs:
- Add it as a core plugin like other plugins
- Or use execution plan loader with metadata from the plan

EOF

echo "✅ Plugin $PLUGIN_NAME scaffolded under $TARGET_DIR"
