# Python Example for FORGE

This is a minimal Python app to demonstrate a FORGE pipeline.

## Files

- `app.py` → simple Python app
- `test_app.py` → unit test
- `requirements.txt` → dependencies
- `forge.yaml` → FORGE pipeline configuration

## Requirements

- Docker installed
- Python 3.11-slim (if running manually)
- Forge CLI (optional for running automatically)

## How to Run

### Option 1: Using Forge CLI

```bash
forge-cli validate --file forge.yaml
forge-cli run --file forge.yaml
```

- Note: On Windows, Forge CLI may default to Alpine. In that case, see Option 2.

## Manual Docker Run on windows

```bash
docker run -it --rm -v ${PWD}:/app -w /app python:3.11-slim bash
pip install -r examples/python/requirements.txt
flake8 examples/python
pytest examples/python
python examples/python/app.py
```

### What it Does

1. Installs dependencies using pip
2. Runs linting using flake8
3. Runs unit tests using pytest
4. Runs the Python app

## Notes

- Minor lint warnings may appear (e.g., missing blank lines)
- This example is intended for demonstrating a FORGE pipeline.