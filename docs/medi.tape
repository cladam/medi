# medi.tape - A demo script for vhs

# Set the output file and terminal theme
Output medi.gif
Set Theme "Catppuccin Macchiato"
Set FontSize 22
Set Width 1200
Set Height 600
Set BorderRadius 10
Set PlaybackSpeed 0.8

# First, set Neovim as EDITOR
Type "export EDITOR=nvim"
Enter
Sleep 1s

# Start with a clean slate by checking the status
Type "medi status"
Enter
Sleep 3s

# Clear the screen for the next command
Ctrl+L
Sleep 1s

# Create a new note with tags
Type 'medi new rust-cli-post -m "Draft post about building CLIs in Rust." --tag rust --tag blog'
Enter
Sleep 2s

# Add a task to that note
Type 'medi task add rust-cli-post "Write the introduction paragraph"'
Enter
Sleep 2s

# List our tasks to see the new one
Type "medi task list"
Enter
Sleep 3s

# Clear the screen for the next command
Ctrl+L
Sleep 1s

# I remember writing about CLIs, but forgot the key. Let's do a search.
Type 'medi search "CLI"'
Enter
Sleep 3s

# Clear the screen for the next command
Ctrl+L
Sleep 1s

# Create a second note that links to the first
Type 'medi new gui-pkm -m "There are some amazing GUI tools but I prefer my own, even got backlinks [[rust-cli-post]]."'
Enter
Sleep 2s

# Now for the magic: find backlinks to our original post
Type "medi backlinks rust-cli-post"
Enter
Sleep 3s

# Clear the screen for the next command
Ctrl+L
Sleep 1s

# Checking the status again
Type "medi status"
Enter
Sleep 3s

# Let's create a note from a pipe
Type 'export MEDI_VAR="My medi note from an environment variable"'
Enter
Type 'echo $MEDI_VAR | medi new medi-var --tag env'
Enter
Sleep 2s

# Check status for the new note
Type "medi status medi-var"
Enter
Sleep 3s

# Clear the screen for the next command
Ctrl+L
Sleep 1s

# --- Cleanup ---
# Now, clean up the notes we created during the demo.
# Use --force to skip the interactive confirmation.
Type "medi delete rust-cli-post --force"
Enter
Sleep 2s
Type "medi delete gui-pkm --force"
Enter
Sleep 2s
Type "medi delete medi-var --force"
Enter
Sleep 2s
# Show the final clean status
Type "medi status"
Enter
Sleep 3s

