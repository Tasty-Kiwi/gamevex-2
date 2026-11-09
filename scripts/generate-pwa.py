#!/usr/bin/env python3
"""Generate an offline worker whose revision covers the exact shipped runtime."""
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WEB = ROOT / "web"
backends = sys.argv[1:] or ["webgpu", "webgl2"]
if any(backend not in ("webgpu", "webgl2") for backend in backends):
    raise SystemExit("Unknown browser backend")
(WEB / "backend.js").write_text(f"export const graphicsBackends = {json.dumps(backends)};\n")
assets = ["index.html", "loader.js", "backend.js", "pwa.js", "manifest.webmanifest",
          "assets/fonts/Oxanium.ttf"]
for backend in backends:
    assets += [f"pkg/{backend}/gamevex_app.js", f"pkg/{backend}/gamevex_app_bg.wasm"]
assets += sorted(str(path.relative_to(WEB)) for path in (WEB / "icons").glob("*.png"))
revision = hashlib.sha256()
template = (ROOT / "scripts/service-worker.js").read_text()
revision.update(template.encode())
for asset in assets:
    revision.update(asset.encode() + b"\0" + (WEB / asset).read_bytes())
version = revision.hexdigest()[:20]
worker = template.replace("__VERSION__", json.dumps(version)).replace("__ASSETS__", json.dumps(assets))
(WEB / "sw.js").write_text(worker)
print(f"Offline runtime: {len(assets)} files, revision {version}")
