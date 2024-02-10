import json
import subprocess
import tomllib
import urllib.request

with open("Cargo.toml", "rb") as f:
    cargo_toml = tomllib.load(f)
crate_version = cargo_toml["package"]["version"]
print("Detected crate version " + crate_version)

index_url = "https://index.crates.io/x8/6_/x86_64"
for line in urllib.request.urlopen(index_url):
    version_info = json.loads(line)
    assert (version_info["name"] == "x86_64")
    if version_info["vers"] == crate_version:
        print("Version " + crate_version + " already exists on crates.io")
        break
else:
    print("Could not find version " + crate_version +
          " on crates.io; creating a new release")

    print("  Running `cargo publish`")
    subprocess.run(["cargo", "publish"], check=True)

    tag_name = "v" + crate_version
    print("  Tagging commit as " + tag_name)
    sha = subprocess.run(["git", "rev-parse", "HEAD"], check=True,
                         stdout=subprocess.PIPE).stdout.decode("utf-8").strip()
    subprocess.run([
        "gh", "api", "/repos/rust-osdev/x86_64/git/refs",
        "-X", "POST", "-H", "Accept: application/vnd.github.v3+json",
        "-F", "ref=refs/tags/" + tag_name,
        "-F", "sha="+sha
    ])

    print("  Done")
