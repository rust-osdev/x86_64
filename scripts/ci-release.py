import toml
import requests
import subprocess

cargo_toml = toml.load("Cargo.toml")
crate_version = cargo_toml["package"]["version"]
print("Detected crate version " + crate_version)

api_url = "https://crates.io/api/v1/crates/x86_64/versions"
crates_io_versions = requests.get(api_url).json()

new_version = True
for version in crates_io_versions["versions"]:
    assert (version["crate"] == "x86_64")
    if version["num"] == crate_version:
        new_version = False
        break

if new_version:
    print("Could not find version " + crate_version + " on crates.io; creating a new release")

    print("  Running `cargo publish`")
    subprocess.run(["cargo", "publish"], check=True)

    tag_name = "v" + crate_version
    print("  Tagging commit as " + tag_name)
    subprocess.run(["git", "tag", tag_name], check=True)
    subprocess.run(["git", "push", "origin", tag_name], check=True)

    print("  Done")
else:
    print("Version " + crate_version + " already exists on crates.io")
