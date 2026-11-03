import subprocess
import sys


def run(*args):
    subprocess.run(args, check=True)


tag = sys.argv[1].strip() if len(sys.argv) > 1 else ""

if not tag:
    print("Existing tags:")
    result = subprocess.run(
        ["git", "tag", "--sort=v:refname"],
        capture_output=True,
        text=True,
        check=True,
    )

    tags = result.stdout.splitlines()
    for existing_tag in tags[-5:]:
        print(f"  {existing_tag}")

    tag = input("New tag (e.g. v1.0.0): ").strip()

if not tag:
    sys.exit("No tag given")
if (
    not tag.startswith("v")
    or tag.count(".") != 2
    or not all(part.isdigit() for part in tag[1:].split("."))
):
    sys.exit("Tag must match vMAJOR.MINOR.PATCH")

run("git", "tag", "-a", tag, "-m", tag)
run("git", "push", "origin", tag)
