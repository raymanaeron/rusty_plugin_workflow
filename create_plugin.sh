#!/bin/bash

if [ $# -ne 3 ]; then
  echo "Usage: ./create_plugin.sh <plugin_name> <plugin_route> <resource_name>"
  exit 1
fi

PLUGIN_NAME=$1
PLUGIN_ROUTE=$2
RESOURCE_NAME=$3

# Convert resource_name to CamelCase
# First to lowercase, then capitalize each word
RESOURCE_NAME_CAMEL=$(echo "$RESOURCE_NAME" | tr '[:upper:]' '[:lower:]' | awk -F_ '{for(i=1;i<=NF;i++){$i=toupper(substr($i,1,1)) substr($i,2)}}1' | sed 's/ //g')

TEMPLATE_DIR="plugin_templates"
TARGET_DIR="plugins/$PLUGIN_NAME"

echo "Creating plugin: $PLUGIN_NAME (route=$PLUGIN_ROUTE, resource=$RESOURCE_NAME, resource_camel=$RESOURCE_NAME_CAMEL)"
mkdir -p "$TARGET_DIR/src"
mkdir -p "$TARGET_DIR/web"

# Process Cargo.toml
sed -e "s/{{plugin_name}}/$PLUGIN_NAME/g" \
    "$TEMPLATE_DIR/Cargo.toml.template" > "$TARGET_DIR/Cargo.toml"

# Process lib.rs - use both resource_name and camelcased version
sed -e "s/{{plugin_name}}/$PLUGIN_NAME/g" \
    -e "s/{{plugin_route}}/$PLUGIN_ROUTE/g" \
    -e "s/{{resource_name}}/$RESOURCE_NAME/g" \
    -e "s/{{resource_name_camel}}/$RESOURCE_NAME_CAMEL/g" \
    "$TEMPLATE_DIR/lib.rs.template" > "$TARGET_DIR/src/lib.rs"

# Process HTML
sed -e "s/{{plugin_route}}/$PLUGIN_ROUTE/g" \
    "$TEMPLATE_DIR/step-template.html" > "$TARGET_DIR/web/step-$PLUGIN_ROUTE.html"

# Process JS
sed -e "s/{{plugin_route}}/$PLUGIN_ROUTE/g" \
    -e "s/{{resource_name}}/$RESOURCE_NAME/g" \
    -e "s/{{resource_name_camel}}/$RESOURCE_NAME_CAMEL/g" \
    -e "s/{{plugin_name}}/$PLUGIN_NAME/g" \
    "$TEMPLATE_DIR/step-template.js" > "$TARGET_DIR/web/step-$PLUGIN_ROUTE.js"

# Process README from template instead of creating it from scratch
sed -e "s/{{plugin_name}}/$PLUGIN_NAME/g" \
    -e "s/{{plugin_route}}/$PLUGIN_ROUTE/g" \
    -e "s/{{resource_name}}/$RESOURCE_NAME/g" \
    -e "s/{{resource_name_camel}}/$RESOURCE_NAME_CAMEL/g" \
    "$TEMPLATE_DIR/README_template.md" > "$TARGET_DIR/README.md"

echo "✅ Plugin $PLUGIN_NAME scaffolded under $TARGET_DIR"
