import subprocess
import sys

def test_cargo_doc():
    result = subprocess.run(
        ["cargo", "doc", "--no-deps"],
        capture_output=True,
        text=True
    )
    if result.returncode != 0 or "warning" in result.stderr.lower():
        print("Documentation check failed:", result.stderr)
        sys.exit(1)

if __name__ == "__main__":
    test_cargo_doc()
