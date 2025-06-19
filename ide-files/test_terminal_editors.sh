#!/bin/bash
# test_terminal_editors.sh

echo "=== Testing Terminal Editor Detection ==="

# Create test files
mkdir -p /tmp/ide-test
echo "console.log('Hello from test file');" > /tmp/ide-test/test.js
echo "print('Hello from Python test')" > /tmp/ide-test/test.py
echo "package main

func main() {
    println(\"Hello from Go test\")
}" > /tmp/ide-test/test.go

echo "Test files created in /tmp/ide-test/"

echo ""
echo "Please run the following commands in different terminals:"
echo "1. vim /tmp/ide-test/test.go"
echo "2. nano /tmp/ide-test/test.py"
echo ""
echo "Then run these detection commands:"
echo "cargo run -- --ide=vim --verbose"
echo "cargo run -- --ide=nano --verbose" 
echo "cargo run -- --auto --verbose"
echo ""
echo "Expected: Should detect the corresponding editor and file being edited"