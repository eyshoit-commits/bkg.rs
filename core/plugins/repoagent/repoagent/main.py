from __future__ import annotations

import asyncio
import contextlib
import json
import logging
import os
import socket
from dataclasses import asdict, dataclass
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional

import aiofiles
import uvicorn
import websockets
from fastapi import FastAPI, HTTPException
from fastapi.responses import JSONResponse
from pydantic import BaseModel, Field, validator

LOGGER = logging.getLogger("bkg.repoagent")
logging.basicConfig(level=logging.INFO, format="[repoagent] %(asctime)s %(levelname)s %(message)s")

PLUGIN_NAME = os.environ.get("BKG_PLUGIN_NAME", "repoagent")
BUS_PORT = int(os.environ.get("BKG_PLUGIN_BUS_PORT", "43121"))
DATABASE_PATH = Path(os.environ.get("BKG_DATABASE_PATH", "/data/bkg.db"))

app = FastAPI(title="BKG RepoAgent", version="1.0.0")


class AnalyzeRequest(BaseModel):
    path: str = Field(..., description="Repository path to inspect")
    query: Optional[str] = Field(None, description="Optional text to search for")
    max_files: int = Field(100, ge=1, le=500, description="Maximum number of files to scan")

    @validator("path")
    def validate_path(cls, value: str) -> str:
        resolved = Path(value).expanduser().resolve()
        if not resolved.exists() or not resolved.is_dir():
            raise ValueError(f"Repository path {resolved} does not exist or is not a directory")
        return str(resolved)


class PatchChange(BaseModel):
    path: str
    content: str
    mode: str = Field("overwrite", pattern="^(overwrite|append)$")


class PatchRequest(BaseModel):
    root: str
    changes: List[PatchChange]

    @validator("root")
    def validate_root(cls, value: str) -> str:
        resolved = Path(value).expanduser().resolve()
        if not resolved.exists() or not resolved.is_dir():
            raise ValueError(f"Root path {resolved} does not exist")
        return str(resolved)


@dataclass
class RepositorySummary:
    total_files: int
    total_directories: int
    total_size: int
    matches: List[Dict[str, Any]]


async def collect_repository_info(request: AnalyzeRequest) -> RepositorySummary:
    root = Path(request.path)
    total_files = 0
    total_directories = 0
    total_size = 0
    matches: List[Dict[str, Any]] = []
    query = request.query.lower() if request.query else None

    for directory, subdirs, files in os.walk(root):
        if total_files >= request.max_files:
            break
        total_directories += 1
        for name in files:
            if total_files >= request.max_files:
                break
            total_files += 1
            file_path = Path(directory, name)
            try:
                stat = file_path.stat()
                total_size += stat.st_size
                if query:
                    async with aiofiles.open(file_path, "r", encoding="utf-8", errors="ignore") as handle:
                        content = await handle.read()
                        if query in content.lower():
                            matches.append(
                                {
                                    "path": str(file_path),
                                    "snippet": extract_snippet(content, query),
                                }
                            )
            except (OSError, IOError):
                continue
    return RepositorySummary(total_files, total_directories, total_size, matches)


def extract_snippet(content: str, query: str, radius: int = 120) -> str:
    index = content.lower().find(query)
    if index == -1:
        return content[:radius]
    start = max(0, index - radius // 2)
    end = min(len(content), index + radius // 2)
    return content[start:end]


async def apply_changes(request: PatchRequest) -> List[Dict[str, Any]]:
    root = Path(request.root)
    results: List[Dict[str, Any]] = []
    for change in request.changes:
        file_path = (root / change.path).resolve()
        if not str(file_path).startswith(str(root)):
            raise HTTPException(status_code=400, detail="Patch path escapes repository root")
        file_path.parent.mkdir(parents=True, exist_ok=True)
        mode = "a" if change.mode == "append" else "w"
        async with aiofiles.open(file_path, mode, encoding="utf-8") as handle:
            await handle.write(change.content)
        results.append({"path": str(file_path), "mode": change.mode})
    return results


@app.post("/analyze")
async def analyze_endpoint(request: AnalyzeRequest):
    summary = await collect_repository_info(request)
    return JSONResponse(
        {
            "summary": {
                "files": summary.total_files,
                "directories": summary.total_directories,
                "bytes": summary.total_size,
            },
            "matches": summary.matches,
        }
    )


@app.post("/patch")
async def patch_endpoint(request: PatchRequest):
    results = await apply_changes(request)
    return {"status": "ok", "changes": results}


@app.get("/health")
async def health() -> Dict[str, Any]:
    return {"status": "ok", "timestamp": datetime.utcnow().isoformat()}


async def send(ws: websockets.WebSocketClientProtocol, message: Dict[str, Any]) -> None:
    await ws.send(json.dumps(message))


async def handle_bus_message(ws: websockets.WebSocketClientProtocol, message: Dict[str, Any]):
    if message.get("type") != "request":
        return
    capability = message.get("capability")
    request_id = message.get("requestId")
    payload = message.get("payload")
    try:
        if capability == "repo.analyze":
            request = AnalyzeRequest(**payload)
            summary = await collect_repository_info(request)
            response = {
                "summary": asdict(summary),
            }
        elif capability == "repo.patch":
            request = PatchRequest(**payload)
            changes = await apply_changes(request)
            response = {"status": "ok", "changes": changes}
        else:
            raise ValueError(f"Unsupported capability {capability}")
        await send(
            ws,
            {
                "type": "response",
                "requestId": request_id,
                "success": True,
                "data": response,
            },
        )
    except Exception as exc:  # noqa: BLE001
        LOGGER.exception("Failed to handle capability %s", capability)
        await send(
            ws,
            {
                "type": "response",
                "requestId": request_id,
                "success": False,
                "error": str(exc),
            },
        )


async def heartbeat(ws: websockets.WebSocketClientProtocol):
    while True:
        await asyncio.sleep(10)
        await send(
            ws,
            {
                "type": "health",
                "plugin": PLUGIN_NAME,
                "status": "up",
            },
        )


async def bus_loop(port: int):
    url = f"ws://127.0.0.1:{BUS_PORT}"
    while True:
        try:
            async with websockets.connect(url) as ws:
                register = {
                    "type": "register",
                    "plugin": PLUGIN_NAME,
                    "port": port,
                    "capabilities": ["repo.analyze", "repo.patch"],
                    "meta": {
                        "database": str(DATABASE_PATH),
                    },
                }
                await send(ws, register)
                await send(
                    ws,
                    {
                        "type": "log",
                        "plugin": PLUGIN_NAME,
                        "level": "info",
                        "message": f"RepoAgent registered on port {port}",
                        "timestamp": datetime.utcnow().isoformat(),
                    },
                )
                hb_task = asyncio.create_task(heartbeat(ws))
                async for payload in ws:
                    message = json.loads(payload)
                    await handle_bus_message(ws, message)
                hb_task.cancel()
                with contextlib.suppress(asyncio.CancelledError):
                    await hb_task
        except Exception as exc:  # noqa: BLE001
            LOGGER.exception("Bus connection failed: %s", exc)
            await asyncio.sleep(3)


def allocate_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("0.0.0.0", 0))
        return sock.getsockname()[1]


def run() -> None:
    port = allocate_port()
    config = uvicorn.Config(app, host="0.0.0.0", port=port, log_level="info")
    server = uvicorn.Server(config)

    async def main_async():
        server_task = asyncio.create_task(server.serve())
        bus_task = asyncio.create_task(bus_loop(port))
        await asyncio.wait(
            {server_task, bus_task},
            return_when=asyncio.FIRST_COMPLETED,
        )

    asyncio.run(main_async())


if __name__ == "__main__":
    run()
