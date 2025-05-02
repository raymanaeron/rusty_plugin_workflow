#!/bin/bash

if [ $# -ne 3 ]; then
  echo "Usage: ./create_plugin.sh <plugin_name> <plugin_route> <resource_name>"
  exit 1
fi

PLUGIN_NAME=$1
PLUGIN_ROUTE=$2
RESOURCE_NAME=$3

# Convert plugin_name to CamelCase
# First to lowercase, then capitalize each word
PLUGIN_NAME_CAMEL=$(echo "$PLUGIN_NAME" | tr '[:upper:]' '[:lower:]' | awk -F_ '{for(i=1;i<=NF;i++){$i=toupper(substr($i,1,1)) substr($i,2)}}1' | sed 's/ //g')

# Convert plugin_route to CamelCase
# First to lowercase, then capitalize each word
PLUGIN_ROUTE_CAMEL=$(echo "$PLUGIN_ROUTE" | tr '[:upper:]' '[:lower:]' | awk -F_ '{for(i=1;i<=NF;i++){$i=toupper(substr($i,1,1)) substr($i,2)}}1' | sed 's/ //g')

# Convert resource_name to CamelCase
# First to lowercase, then capitalize each word
RESOURCE_NAME_CAMEL=$(echo "$RESOURCE_NAME" | tr '[:upper:]' '[:lower:]' | awk -F_ '{for(i=1;i<=NF;i++){$i=toupper(substr($i,1,1)) substr($i,2)}}1' | sed 's/ //g')

TEMPLATE_DIR="plugin_templates"
TARGET_DIR="plugins/$PLUGIN_NAME"

echo "Creating plugin: $PLUGIN_NAME (route=$PLUGIN_ROUTE, resource=$RESOURCE_NAME, resource_camel=$RESOURCE_NAME_CAMEL, plugin_camel=$PLUGIN_NAME_CAMEL)"
mkdir -p "$TARGET_DIR/src"
mkdir -p "$TARGET_DIR/web"

# Process Cargo.toml
sed -e "s/{{plugin_name}}/$PLUGIN_NAME/g" \
    -e "s/{{plugin_name_camel}}/$PLUGIN_NAME_CAMEL/g" \
    "$TEMPLATE_DIR/Cargo.toml.template" > "$TARGET_DIR/Cargo.toml"

# Process lib.rs - use both resource_name and camelcased version
sed -e "s/{{plugin_name}}/$PLUGIN_NAME/g" \
    -e "s/{{plugin_route}}/$PLUGIN_ROUTE/g" \
    -e "s/{{resource_name}}/$RESOURCE_NAME/g" \
    -e "s/{{resource_name_camel}}/$RESOURCE_NAME_CAMEL/g" \
    -e "s/{{plugin_name_camel}}/$PLUGIN_NAME_CAMEL/g" \
    "$TEMPLATE_DIR/lib.rs.template" > "$TARGET_DIR/src/lib.rs"

# Process HTML
sed -e "s/{{plugin_route}}/$PLUGIN_ROUTE/g" \
    -e "s/{{plugin_name}}/$PLUGIN_NAME/g" \
    -e "s/{{plugin_name_camel}}/$PLUGIN_NAME_CAMEL/g" \
    "$TEMPLATE_DIR/step-x.html.template" > "$TARGET_DIR/web/step-$PLUGIN_ROUTE.html"

# Process JS
sed -e "s/{{plugin_route}}/$PLUGIN_ROUTE/g" \
    -e "s/{{resource_name}}/$RESOURCE_NAME/g" \
    -e "s/{{resource_name_camel}}/$RESOURCE_NAME_CAMEL/g" \
    -e "s/{{plugin_name}}/$PLUGIN_NAME/g" \
    -e "s/{{plugin_name_camel}}/$PLUGIN_NAME_CAMEL/g" \
    "$TEMPLATE_DIR/step-x.js.template" > "$TARGET_DIR/web/step-$PLUGIN_ROUTE.js"

# Process README from template instead of creating it from scratch
sed -e "s/{{plugin_name}}/$PLUGIN_NAME/g" \
    -e "s/{{plugin_route}}/$PLUGIN_ROUTE/g" \
    -e "s/{{resource_name}}/$RESOURCE_NAME/g" \
    -e "s/{{resource_name_camel}}/$RESOURCE_NAME_CAMEL/g" \
    -e "s/{{plugin_name_camel}}/$PLUGIN_NAME_CAMEL/g" \
    "$TEMPLATE_DIR/README_x.md.template" > "$TARGET_DIR/README.md"

# Process plugin_definition.toml from template
sed -e "s/{{plugin_name}}/$PLUGIN_NAME/g" \
    -e "s/{{plugin_route}}/$PLUGIN_ROUTE/g" \
    -e "s/{{plugin_route_camel}}/$PLUGIN_ROUTE_CAMEL/g" \
    -e "s/{{resource_name}}/$RESOURCE_NAME/g" \
    -e "s/{{resource_name_camel}}/$RESOURCE_NAME_CAMEL/g" \
    -e "s/{{plugin_name_camel}}/$PLUGIN_NAME_CAMEL/g" \
    "$TEMPLATE_DIR/plugin_definition.toml.template" > "$TARGET_DIR/plugin_metadata.toml"

echo "Plugin $PLUGIN_NAME scaffolded under $TARGET_DIR"
