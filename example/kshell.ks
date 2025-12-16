# --- Example of a script that will be alble to run with KShell ---

# Navigate into source directory
cd src

# Print a greeting
echo "Starting build process..."

# Collect all C source files
sources = find . -name "*.c"

# Compile them into a single executable
gcc $sources -o ../hello.exe

# Return to project root
cd ..

# Define a reusable function
function run_program(path) {
    echo "Executing: $path"
    exec $path
    echo "Exit code: $?"
}

# Run the compiled program
run_program ./hello.exe

# Example: run a program from PATH
python --version

# Conditional execution
if ./hello.exe; then
    echo "Program ran successfully!"
else
    echo "Program failed!"
fi

# Loop through all .c files and show line counts
for file in $(find src -name "*.c"); do
    lines = wc -l $file
    echo "File: $file has $lines lines"
done

# Demonstrate piping and variable capture
list = ls src | grep ".c"
echo "Source files found: $list"

# Nested conditionals
if test -f ./hello.exe; then
    echo "Executable exists."
    if test -x ./hello.exe; then
        echo "Executable has run permissions."
    else
        echo "Executable is missing run permissions."
    fi
else
    echo "Executable not found."
fi

# Type inspection
type echo
type gcc

# Environment variables
PATH = $PATH:/usr/local/bin
echo "Updated PATH: $PATH"

# Error handling example
if ! gcc $sources -o ../hello.exe; then
    echo "Compilation failed, aborting script."
    exit 1
fi

# Final run
./hello.exe
