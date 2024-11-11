#!/bin/bash

# Function to display error message and exit
error_exit() {
    echo "$1" 1>&2
    exit 1
}

# get username and hostname from env variables  

username=$SERVER_USERNAME
hostname=$SERVER_HOSTNAME

# Check if username and hostname are set 
if [ -z "$username" ] || [ -z "$hostname" ]; then
    error_exit "Username and hostname must be set."
fi

project="fuente"
app_name="driver_staging"

# Run trunk build --release
trunk build --release || error_exit "trunk build --release failed."

# Check if 'dist' folder exists
if [ ! -d "dist" ]; then
    error_exit "'dist' folder not found!"
fi

# Create a folder with the project name and copy 'dist' folder into it
mkdir "$app_name" || error_exit "Failed to create folder with project name."
# Copy the contents of 'dist' folder into the folder with the project name
cp -r dist/* "$app_name" || error_exit "Failed to copy 'dist' folder contents."

# SCP the folder to the server
scp -r "$app_name" "$username@$hostname:~/$project/$app_name" || error_exit "SCP failed."

# Clean up
rm -r "$app_name"
rm -r dist

# run remote script 
ssh "$username@$hostname" "bash -s" < remote_deploy.sh || error_exit "Remote script failed."

echo "Deployment successful and folders cleaned up."


