#!/bin/bash

NEW_APP_NAME="invoicer_staging"
OLD_APP_NAME="invoicer"
WORKING_DIR="fuente/$OLD_APP_NAME"
STAGING_DIR="fuente/$NEW_APP_NAME"
BACKUP_DIR="fuente/backups"

# Function to display error message and exit
error_exit() {
    echo "$1" 1>&2
    exit 1
}

# Check if the staging folder was copied in  
if [ ! -d "$STAGING_DIR" ]; then
    error_exit "Staging folder not found."
fi

# Check if the 'backups' folder exists if it doesnt, create it 
if [ ! -d "$BACKUP_DIR" ]; then
    mkdir -p "$BACKUP_DIR" || error_exit "Failed to create backups folder."
fi

# Check if proceess is running 
if [ -n "$(ps -ef | grep $OLD_APP_NAME | grep -v grep)" ]; then
    pkill $OLD_APP_NAME || error_exit "Failed to stop the current app."
fi

# Backup the current working app with timestamp
mv "$WORKING_DIR" "$BACKUP_DIR/$OLD_APP_NAME-$(date +"%Y-%m-%d-%H-%M-%S")" || error_exit "Failed to backup current app."

# Copy the new app to the working folder 
cp -r "$STAGING_DIR" "$WORKING_DIR" || error_exit "Failed to copy new app to working folder."

# Run the new app 
cd "$WORKING_DIR" || error_exit "Failed to change directory to working folder."
nohup "./$OLD_APP_NAME" > invoice_dev.log 2>&1 & || error_exit "Failed to run the new app."

# Clean up 

rm -r "$STAGING_DIR" || error_exit "Failed to clean up staging folder."


echo "Deployment completed successfully."



