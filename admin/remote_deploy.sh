#!/bin/bash

STAGING_DIR="fuente"
BACKUP_DIR="$STAGING_DIR/backups"
NEW_APP_NAME="admin_staging"
OLD_APP_NAME="admin"
DIST_DIR="/var/www/fuente_dev/$OLD_APP_NAME"

# Function to display error message and exit
error_exit() {
    echo "$1" 1>&2
    exit 1
}

# Check if the 'dist' folder exists if it doesnt, create it 
if [ ! -d "$DIST_DIR" ]; then
    mkdir -p "$DIST_DIR" || error_exit "Failed to create dist folder."
fi 

# Check if the 'backups' folder exists if it doesnt, create it 
if [ ! -d "$BACKUP_DIR" ]; then
    mkdir -p "$BACKUP_DIR" || error_exit "Failed to create backups folder."
fi 

# Backup the current app with timestamp
mv "$DIST_DIR" "$BACKUP_DIR/$OLD_APP_NAME-$(date +"%Y-%m-%d-%H-%M-%S")" || error_exit "Failed to backup current app."

# Copy the new app to the dist folder 
cp -r "$STAGING_DIR/$NEW_APP_NAME" "$DIST_DIR" || error_exit "Failed to copy new app to dist folder."

# Clean up 
rm -r "$STAGING_DIR/$NEW_APP_NAME" || error_exit "Failed to clean up staging folder."

echo "Deployment completed successfully."


