# Go Example for Forge

This is a simple Go example demonstrating how to use Forge to **build, test, and vet** a Go project, providing a template for CI/CD automation.

## Project Structure

This directory contains the minimal files required for a functional Go program integrated with a Forge pipeline.  

examples/go/  
├── main.go         # Main program file  
├── main_test.go    # Unit test for the core function  
├── go.mod          # Go module dependency file  
├── forge.yaml      # Forge pipeline configuration  
└── README.md       # This file  

## How to Run Locally

Navigate to the `examples/go/` folder in your terminal to run the application, tests, and standard Go tools.

### 1. Run the Program
go run .  
**Expected output:**
Hello, Forge!

### 2. Run Tests
go test  
**Expected output:**
ok  	example.com/forgeexample	0.1s

### 3. Format Code

To ensure consistent code style:

go fmt ./...  

### 4. Vet Code for Issues

To check for common issues and potential bugs:

go vet ./...


## Forge Pipeline

The `forge.yaml` file defines the continuous integration workflow through 3 distinct stages: **Build**, **Test**, and **Vet**.

| Stage | Command | Purpose | 
 | ----- | ----- | ----- | 
| **Build** | `go build ./...` | Compiles the code and checks for compilation errors. | 
| **Test** | `go test ./...` | Executes all defined unit tests. | 
| **Vet** | `go vet ./...` | Performs static analysis to detect code issues. | 

### Caching

Go modules are effectively cached at `/go/pkg/mod` within the pipeline environment to significantly speed up subsequent builds.

## Learning Points

* **Packages:** Go programs are organized into packages; `main` is the required entry point for executable applications.

* **Functions:** The simple `greet(name string)` function demonstrates basic function definition and usage.

* **Tests:** Files ending in `_test.go` are automatically detected and run by the `go test` command.

* **Code Quality:** `go fmt` (formatting) and `go vet` (vetting) are essential tools for maintaining code quality.

* **Forge Integration:** The `forge.yaml` file maps these standard developer workflows directly into automated CI/CD stages.

## Author

Created as a contribution example for Forge