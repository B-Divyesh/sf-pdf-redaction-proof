#!/usr/bin/env python3
import hashlib, json, pathlib, sys

tag, repo, root = sys.argv[1], sys.argv[2], pathlib.Path(sys.argv[3])
files = [p for p in root.iterdir() if p.is_file() and p.name not in {"SHA256SUMS", "latest.json"}]

def choose(suffixes, hints=()):
    matches = [p for p in files if any(p.name.lower().endswith(s) for s in suffixes)]
    for hint in hints:
        hinted = [p for p in matches if hint in p.name.lower()]
        if hinted: matches = hinted; break
    if not matches: raise SystemExit(f"Missing release asset: {suffixes}")
    path = sorted(matches, key=lambda p: len(p.name))[0]
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    return {"url": f"https://github.com/{repo}/releases/download/{tag}/{path.name}", "sha256": digest, "asset": path.name}

manifest = {
    "version": tag,
    "platforms": {
        "macos": choose((".dmg",)),
        "windows": choose((".msi", ".exe"), (".msi",)),
        "linux": choose((".appimage",), ("amd64", "x86_64")),
    },
}
(root / "latest.json").write_text(json.dumps(manifest, indent=2) + "\n")
