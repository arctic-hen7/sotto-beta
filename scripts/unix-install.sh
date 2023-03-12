#!/bin/bash
# This script installs Sotto on Linux and MacOS. It expects `sotto-app` and `sotto` to be available in the working directory.

echo "Assuming you have 'python3.10' available as a command. Please terminate the script now if you don't, and install it."
sleep 5

mkdir ~/.sotto
mv ./* ~/.sotto
cd ~/.sotto
python3.10 -m venv venv
source venv/bin/activate
pip install openai-whisper
chmod +x ./sotto-app
chmod +x ./sotto

if [[ $OSTYPE == 'darwin'* ]]; then
    # We're on MacOS, they'll have to create an applet themselves
    echo "Sotto installation complete! You'll need to create an applet to execute the 'sotto' file in this directory."
else
    # On Linux, we can just give them the desktop file
    cp ./unix-desktop.desktop ~/.local/share/applications/sotto.desktop
    echo "Please enter your password at the prompt to update the database of desktop applications on your system."
    sudo update-desktop-database
    echo "Sotto installation complete! You should find the Sotto application on your system!"
fi

# Uninstallation instructions: remove the `~/.local/share/applications/sotto.desktop` file, and delete the `~/.sotto` directory. That's all!
