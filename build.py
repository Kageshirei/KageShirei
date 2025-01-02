import os

def build_ci():
    print("Building in CI")
    # Do the build here

def build_local():
    print("Building locally")
    # Do the build here

if __name__ == "__main__":
    # Check if the environment variable is set, if it is we are running in GitHub Actions
    is_ci = os.getenv("GITHUB_ACTIONS") == "true"

    print(f"Environment detected: {'CI' if is_ci else 'Local'}")

    if is_ci:
        build_ci()
    else:
        build_local()
