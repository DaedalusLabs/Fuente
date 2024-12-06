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
app_name="invoicer"
staging_name="invoicer_staging"

# Run release build 
cargo build --release || error_exit "cargo build --release failed."

# Check if target/release folder exists
if [ ! -d "../target/release" ]; then
    error_exit "'dist' folder not found!"
fi

# Create a folder with the project name and copy release binary into it
mkdir "$staging_name" || error_exit "Failed to create folder with project name."

# Copy the release binary into the folder with the project name 
cp -r ../target/release/$app_name "$staging_name" || error_exit "Failed to copy release binary."

# copy macaroon file 
cp admin.macaroon "$staging_name" || error_exit "Failed to copy macaroon file."

# SCP the folder to the server
scp -r "$staging_name" "$username@$hostname:~/$project" || error_exit "SCP failed."

# Clean up
rm -r "$staging_name"

# run remote script 
# ssh "$username@$hostname" "bash -s" < remote_deploy.sh || error_exit "Remote script failed."

echo "Deployment successful and folders cleaned up."


