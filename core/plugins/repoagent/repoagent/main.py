from __future__ import annotations

import asyncio
import contextlib
import json
import logging
import os
import socket
import time
from dataclasses import asdict, dataclass, field
from datetime import datetime
from fnmatch import fnmatch
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Sequence

import aiofiles
import uvicorn
import websockets
from fastapi import FastAPI, HTTPException, Query
from fastapi.responses import JSONResponse
from pydantic import BaseModel, Field, validator

try:
    import psutil  # type: ignore
except Exception:  # pragma: no cover - psutil may not be available at build time
    psutil = None

LOGGER = logging.getLogger("bkg.repoagent")
logging.basicConfig(level=logging.INFO, format="[repoagent] %(asctime)s %(levelname)s %(message)s")

PLUGIN_NAME = os.environ.get("BKG_PLUGIN_NAME", "repoagent")
BUS_PORT = int(os.environ.get("BKG_PLUGIN_BUS_PORT", "43121"))
DATABASE_PATH = Path(os.environ.get("BKG_DATABASE_PATH", "/data/bkg.db"))
CONFIG_PATH = os.environ.get("BKG_PLUGIN_CONFIG_PATH")
DEFAULT_CONFIG_FILE = Path(__file__).resolve().parent.parent / "config.json"

app = FastAPI(title="BKG RepoAgent", version="1.1.0")

CONFIG_SCHEMA: Dict[str, Any] = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "RepoAgentSettings",
    "type": "object",
    "properties": {
        "defaultRoot": {"type": "string", "description": "Default workspace root directory"},
        "workspaceRoots": {
            "type": "array",
            "items": {"type": "string"},
            "description": "Additional workspace roots that the RepoAgent is allowed to access",
        },
        "maxFiles": {
            "type": "integer",
            "minimum": 1,
            "maximum": 5000,
            "description": "Maximum number of files to scan when analysing repositories",
        },
        "ignoreGlobs": {
            "type": "array",
            "items": {"type": "string"},
            "description": "Glob patterns that will be ignored during scans and searches",
        },
        "commandAllowlist": {
            "type": "array",
            "items": {
                "type": "object",
                "required": ["name", "executable"],
                "properties": {
                    "name": {"type": "string"},
                    "executable": {
                        "type": "array",
                        "items": {"type": "string"},
                        "minItems": 1,
                    },
                    "allowArgs": {"type": "boolean", "default": False},
                    "timeoutSeconds": {"type": "integer", "minimum": 1, "maximum": 7200, "default": 300},
                    "workingDir": {"type": "string"},
                },
            },
        },
        "environment": {
            "type": "object",
            "additionalProperties": {"type": "string"},
            "description": "Environment variables that will be injected into allowlisted commands",
        },
        "enableGit": {"type": "boolean", "default": True},
        "telemetry": {
            "type": "object",
            "properties": {
                "sampleIntervalSeconds": {
                    "type": "integer",
                    "minimum": 5,
                    "maximum": 3600,
                    "default": 15,
                }
            },
            "default": {"sampleIntervalSeconds": 15},
        },
    },
    "required": ["defaultRoot", "maxFiles"],
}


@dataclass
class CommandSpec:
    name: str
    executable: Sequence[str]
    timeout_seconds: int = 300
    allow_args: bool = False
    working_dir: Optional[str] = None


@dataclass
class TelemetryConfig:
    sample_interval_seconds: int = 15


@dataclass
class RepoAgentSettings:
    default_root: Path
    workspace_roots: List[Path]
    max_files: int
    ignore_globs: List[str] = field(default_factory=list)
    command_allowlist: List[CommandSpec] = field(default_factory=list)
    environment: Dict[str, str] = field(default_factory=dict)
    enable_git: bool = True
    telemetry: TelemetryConfig = field(default_factory=TelemetryConfig)

    @classmethod
    def from_mapping(cls, payload: Dict[str, Any]) -> "RepoAgentSettings":
        default_root = Path(payload.get("defaultRoot") or payload.get("default_root") or ".").expanduser()
        workspace_roots = [Path(entry).expanduser() for entry in payload.get("workspaceRoots", [])]
        max_files = int(payload.get("maxFiles") or payload.get("max_files") or 500)
        ignore_globs = list(payload.get("ignoreGlobs") or payload.get("ignore_globs") or [])
        allowlist = []
        for entry in payload.get("commandAllowlist", []) or payload.get("command_allowlist", []):
            allowlist.append(
                CommandSpec(
                    name=entry["name"],
                    executable=entry["executable"],
                    timeout_seconds=int(entry.get("timeoutSeconds") or entry.get("timeout_seconds") or 300),
                    allow_args=bool(entry.get("allowArgs") or entry.get("allow_args") or False),
                    working_dir=entry.get("workingDir") or entry.get("working_dir"),
                )
            )
        telemetry_payload = payload.get("telemetry", {})
        telemetry = TelemetryConfig(
            sample_interval_seconds=int(telemetry_payload.get("sampleIntervalSeconds") or 15)
        )
        environment = {str(key): str(value) for key, value in (payload.get("environment") or {}).items()}
        enable_git = bool(payload.get("enableGit", payload.get("enable_git", True)))
        return cls(
            default_root=default_root.resolve(),
            workspace_roots=[path.resolve() for path in workspace_roots],
            max_files=max_files,
            ignore_globs=ignore_globs,
            command_allowlist=allowlist,
            environment=environment,
            enable_git=enable_git,
            telemetry=telemetry,
        )

    def serialise(self) -> Dict[str, Any]:
        return {
            "defaultRoot": str(self.default_root),
            "workspaceRoots": [str(path) for path in self.workspace_roots],
            "maxFiles": self.max_files,
            "ignoreGlobs": list(self.ignore_globs),
            "commandAllowlist": [
                {
                    "name": spec.name,
                    "executable": list(spec.executable),
                    "timeoutSeconds": spec.timeout_seconds,
                    "allowArgs": spec.allow_args,
                    **({"workingDir": spec.working_dir} if spec.working_dir else {}),
                }
                for spec in self.command_allowlist
            ],
            "environment": dict(self.environment),
            "enableGit": self.enable_git,
            "telemetry": {
                "sampleIntervalSeconds": self.telemetry.sample_interval_seconds,
            },
        }


@dataclass
class RepositorySummary:
    total_files: int
    total_directories: int
    total_size: int
    matches: List[Dict[str, Any]]


@dataclass
class TreeEntry:
    path: str
    kind: str
    size: int
    modified_at: float


@dataclass
class CommandResult:
    command: List[str]
    exit_code: int
    stdout: str
    stderr: str
    duration_seconds: float


class AnalyzeRequest(BaseModel):
    path: Optional[str] = Field(None, description="Repository path to inspect")
    query: Optional[str] = Field(None, description="Optional text to search for")
    max_files: Optional[int] = Field(None, ge=1, le=5000, description="Maximum number of files to scan")

    @validator("path")
    def validate_path(cls, value: Optional[str]) -> Optional[str]:
        if value is None:
            return value
        resolved = Path(value).expanduser()
        if not resolved.exists():
            raise ValueError(f"Repository path {resolved} does not exist")
        return str(resolved)


class PatchChange(BaseModel):
    path: str
    content: str
    mode: str = Field("overwrite", pattern="^(overwrite|append)$")


class PatchRequest(BaseModel):
    root: Optional[str]
    changes: List[PatchChange]


class TreeRequest(BaseModel):
    path: Optional[str]
    depth: int = Field(2, ge=1, le=10)
    limit: int = Field(500, ge=1, le=5000)


class ReadFileRequest(BaseModel):
    path: str
    encoding: str = "utf-8"


class WriteFileRequest(BaseModel):
    path: str
    content: str
    encoding: str = "utf-8"
    mode: str = Field("overwrite", pattern="^(overwrite|append|truncate)$")


class SearchRequest(BaseModel):
    query: str
    path: Optional[str]
    max_results: int = Field(200, ge=1, le=5000)
    case_sensitive: bool = False


class CommandInvocationRequest(BaseModel):
    command: str
    args: List[str] = Field(default_factory=list)
    timeout_seconds: Optional[int] = Field(None, ge=1, le=7200)


class CommitRequest(BaseModel):
    message: str
    paths: Optional[List[str]] = None
    signoff: bool = False


class RepoAgentService:
    def __init__(self, settings: RepoAgentSettings):
        self.settings = settings
        self.allowed_roots = {settings.default_root, *settings.workspace_roots}

    def _resolve_path(self, path: Optional[str]) -> Path:
        if path is None:
            return self.settings.default_root
        candidate = Path(path).expanduser()
        if not candidate.is_absolute():
            candidate = (self.settings.default_root / candidate).resolve()
        else:
            candidate = candidate.resolve()
        for root in self.allowed_roots:
            if candidate == root or candidate.is_relative_to(root):
                return candidate
        raise HTTPException(status_code=400, detail=f"Path {candidate} is outside the permitted workspace roots")

    def _is_ignored(self, path: Path) -> bool:
        relative = None
        for root in self.allowed_roots:
            if path.is_relative_to(root):
                relative = path.relative_to(root)
                break
        if relative is None:
            return True
        relative_str = str(relative)
        return any(fnmatch(relative_str, pattern) for pattern in self.settings.ignore_globs)

    async def analyse(self, request: AnalyzeRequest) -> RepositorySummary:
        root = self._resolve_path(request.path)
        max_files = request.max_files or self.settings.max_files
        query = request.query.lower() if request.query else None

        def _scan() -> RepositorySummary:
            total_files = 0
            total_directories = 0
            total_size = 0
            matches: List[Dict[str, Any]] = []
            for directory, subdirs, files in os.walk(root):
                current_dir = Path(directory)
                if self._is_ignored(current_dir):
                    subdirs[:] = []
                    continue
                total_directories += 1
                for name in files:
                    file_path = current_dir / name
                    if self._is_ignored(file_path):
                        continue
                    if total_files >= max_files:
                        return RepositorySummary(total_files, total_directories, total_size, matches)
                    total_files += 1
                    try:
                        stat = file_path.stat()
                        total_size += stat.st_size
                        if query:
                            with file_path.open("r", encoding="utf-8", errors="ignore") as handle:
                                content = handle.read()
                                lowered = content.lower()
                                if query in lowered:
                                    matches.append(
                                        {
                                            "path": str(file_path),
                                            "snippet": extract_snippet(content, query),
                                        }
                                    )
                    except (OSError, IOError):
                        continue
            return RepositorySummary(total_files, total_directories, total_size, matches)

        return await asyncio.to_thread(_scan)

    async def list_tree(self, request: TreeRequest) -> List[TreeEntry]:
        root = self._resolve_path(request.path)

        def _walk() -> List[TreeEntry]:
            entries: List[TreeEntry] = []
            queue: List[tuple[Path, int]] = [(root, 0)]
            while queue and len(entries) < request.limit:
                current, depth = queue.pop(0)
                if depth > request.depth:
                    continue
                try:
                    for entry in current.iterdir():
                        if self._is_ignored(entry):
                            continue
                        stat = entry.stat()
                        kind = "directory" if entry.is_dir() else "file"
                        entries.append(
                            TreeEntry(
                                path=str(entry),
                                kind=kind,
                                size=stat.st_size,
                                modified_at=stat.st_mtime,
                            )
                        )
                        if kind == "directory" and depth + 1 <= request.depth:
                            queue.append((entry, depth + 1))
                        if len(entries) >= request.limit:
                            break
                except (OSError, IOError):
                    continue
            return entries

        return await asyncio.to_thread(_walk)

    async def read_file(self, request: ReadFileRequest) -> Dict[str, Any]:
        path = self._resolve_path(request.path)
        if not path.exists() or not path.is_file():
            raise HTTPException(status_code=404, detail=f"File {path} not found")
        async with aiofiles.open(path, "r", encoding=request.encoding, errors="ignore") as handle:
            content = await handle.read()
        stat = await asyncio.to_thread(path.stat)
        return {
            "path": str(path),
            "content": content,
            "encoding": request.encoding,
            "size": stat.st_size,
            "modifiedAt": stat.st_mtime,
        }

    async def write_file(self, request: WriteFileRequest) -> Dict[str, Any]:
        path = self._resolve_path(request.path)
        path.parent.mkdir(parents=True, exist_ok=True)
        mode = "w" if request.mode in {"overwrite", "truncate"} else "a"
        async with aiofiles.open(path, mode, encoding=request.encoding) as handle:
            if request.mode == "truncate":
                await handle.truncate(0)
            await handle.write(request.content)
        stat = await asyncio.to_thread(path.stat)
        return {
            "path": str(path),
            "size": stat.st_size,
            "modifiedAt": stat.st_mtime,
        }

    async def apply_patch(self, request: PatchRequest) -> List[Dict[str, Any]]:
        root = self._resolve_path(request.root)
        results: List[Dict[str, Any]] = []
        for change in request.changes:
            target = self._resolve_path(str((root / change.path).resolve()))
            try:
                target.relative_to(root)
            except ValueError as exc:  # noqa: PERF203
                raise HTTPException(status_code=400, detail="Patch path escapes repository root") from exc
            target.parent.mkdir(parents=True, exist_ok=True)
            mode = "a" if change.mode == "append" else "w"
            async with aiofiles.open(target, mode, encoding="utf-8") as handle:
                await handle.write(change.content)
            results.append({"path": str(target), "mode": change.mode})
        return results

    async def search(self, request: SearchRequest) -> Dict[str, Any]:
        root = self._resolve_path(request.path)
        query = request.query if request.case_sensitive else request.query.lower()
        matches: List[Dict[str, Any]] = []

        def _search() -> List[Dict[str, Any]]:
            collected: List[Dict[str, Any]] = []
            for directory, _, files in os.walk(root):
                current_dir = Path(directory)
                if self._is_ignored(current_dir):
                    continue
                for name in files:
                    file_path = current_dir / name
                    if self._is_ignored(file_path):
                        continue
                    try:
                        with file_path.open("r", encoding="utf-8", errors="ignore") as handle:
                            content = handle.read()
                            haystack = content if request.case_sensitive else content.lower()
                            if query in haystack:
                                collected.append(
                                    {
                                        "path": str(file_path),
                                        "snippet": extract_snippet(content, request.query),
                                    }
                                )
                                if len(collected) >= request.max_results:
                                    return collected
                    except (OSError, IOError):
                        continue
            return collected

        matches = await asyncio.to_thread(_search)
        return {"query": request.query, "matches": matches}

    async def _execute(self, command: Sequence[str], working_dir: Path, timeout: int) -> CommandResult:
        start = time.perf_counter()
        process = await asyncio.create_subprocess_exec(
            *command,
            cwd=str(working_dir),
            env={**os.environ, **self.settings.environment},
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        try:
            stdout, stderr = await asyncio.wait_for(process.communicate(), timeout=timeout)
        except asyncio.TimeoutError as exc:
            process.kill()
            raise HTTPException(status_code=504, detail=f"Command timed out after {timeout} seconds") from exc
        duration = time.perf_counter() - start
        return CommandResult(
            command=list(command),
            exit_code=process.returncode,
            stdout=stdout.decode("utf-8", errors="ignore"),
            stderr=stderr.decode("utf-8", errors="ignore"),
            duration_seconds=duration,
        )

    async def run_command(self, request: CommandInvocationRequest) -> CommandResult:
        spec = next((entry for entry in self.settings.command_allowlist if entry.name == request.command), None)
        if spec is None:
            raise HTTPException(status_code=400, detail=f"Command {request.command} is not allowlisted")
        if request.args and not spec.allow_args:
            raise HTTPException(status_code=400, detail="Arguments are not permitted for this command")
        timeout = request.timeout_seconds or spec.timeout_seconds
        working_dir = self._resolve_path(spec.working_dir) if spec.working_dir else self.settings.default_root
        cmd = list(spec.executable)
        if spec.allow_args and request.args:
            cmd.extend(request.args)
        return await self._execute(cmd, working_dir, timeout)

    async def commit(self, request: CommitRequest) -> Dict[str, Any]:
        if not self.settings.enable_git:
            raise HTTPException(status_code=400, detail="Git integration is disabled for this plug-in")
        repo_root = self.settings.default_root
        if not (repo_root / ".git").exists():
            raise HTTPException(status_code=400, detail="No git repository found at the default root")
        add_cmd: List[str]
        if request.paths:
            resolved_paths = []
            for path in request.paths:
                resolved = self._resolve_path(path)
                if not resolved.exists():
                    raise HTTPException(status_code=404, detail=f"Path {resolved} does not exist")
                try:
                    relative = resolved.relative_to(repo_root)
                except ValueError as exc:  # noqa: PERF203
                    raise HTTPException(status_code=400, detail=f"Path {resolved} is outside the repository root") from exc
                resolved_paths.append(str(relative))
            add_cmd = ["git", "add", *resolved_paths]
        else:
            add_cmd = ["git", "add", "-A"]
        add_result = await self._execute(add_cmd, repo_root, timeout=300)
        commit_cmd = ["git", "commit", "-m", request.message]
        if request.signoff:
            commit_cmd.append("--signoff")
        commit_result = await self._execute(commit_cmd, repo_root, timeout=600)
        return {
            "staging": {
                "command": add_result.command,
                "exitCode": add_result.exit_code,
                "stdout": add_result.stdout,
                "stderr": add_result.stderr,
                "duration": add_result.duration_seconds,
            },
            "commit": {
                "command": commit_result.command,
                "exitCode": commit_result.exit_code,
                "stdout": commit_result.stdout,
                "stderr": commit_result.stderr,
                "duration": commit_result.duration_seconds,
            },
        }


def extract_snippet(content: str, query: str, radius: int = 160) -> str:
    lowered = content.lower()
    key = query.lower()
    index = lowered.find(key)
    if index == -1:
        return content[:radius]
    start = max(0, index - radius // 2)
    end = min(len(content), index + len(query) + radius // 2)
    return content[start:end]


def load_settings() -> RepoAgentSettings:
    if CONFIG_PATH and Path(CONFIG_PATH).exists():
        with open(CONFIG_PATH, "r", encoding="utf-8") as handle:
            payload = json.load(handle)
            data = payload.get("settings", payload)
            return RepoAgentSettings.from_mapping(data)
    if DEFAULT_CONFIG_FILE.exists():
        with open(DEFAULT_CONFIG_FILE, "r", encoding="utf-8") as handle:
            payload = json.load(handle)
            data = payload.get("settings", payload)
            return RepoAgentSettings.from_mapping(data)
    return RepoAgentSettings.from_mapping({
        "defaultRoot": str(Path.cwd()),
        "workspaceRoots": [str(Path.cwd())],
        "maxFiles": 500,
    })


SETTINGS = load_settings()
SERVICE = RepoAgentService(SETTINGS)


@app.get("/health")
async def health() -> Dict[str, Any]:
    return {"status": "ok", "timestamp": datetime.utcnow().isoformat(), "settings": SETTINGS.serialise()}


@app.get("/v1/config")
async def read_config():
    return SETTINGS.serialise()


@app.post("/analyze")
async def analyze_endpoint(request: AnalyzeRequest):
    summary = await SERVICE.analyse(request)
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
    results = await SERVICE.apply_patch(request)
    return {"status": "ok", "changes": results}


@app.get("/v1/repos/tree")
async def tree_endpoint(
    path: Optional[str] = Query(None, description="Root path to traverse"),
    depth: int = Query(2, ge=1, le=10),
    limit: int = Query(500, ge=1, le=5000),
):
    entries = await SERVICE.list_tree(TreeRequest(path=path, depth=depth, limit=limit))
    return {
        "root": str(SERVICE._resolve_path(path)),
        "entries": [asdict(entry) for entry in entries],
    }


@app.get("/v1/repos/file")
async def read_file_endpoint(path: str = Query(..., description="File path"), encoding: str = "utf-8"):
    result = await SERVICE.read_file(ReadFileRequest(path=path, encoding=encoding))
    return result


@app.post("/v1/repos/file")
async def write_file_endpoint(request: WriteFileRequest):
    result = await SERVICE.write_file(request)
    return {"status": "ok", **result}


@app.get("/v1/repos/search")
async def search_endpoint(
    query: str = Query(..., description="Search query"),
    path: Optional[str] = Query(None),
    max_results: int = Query(200, ge=1, le=5000),
    case_sensitive: bool = Query(False),
):
    payload = SearchRequest(query=query, path=path, max_results=max_results, case_sensitive=case_sensitive)
    result = await SERVICE.search(payload)
    return result


@app.post("/v1/commands/execute")
async def command_endpoint(request: CommandInvocationRequest):
    result = await SERVICE.run_command(request)
    return {
        "command": result.command,
        "exitCode": result.exit_code,
        "stdout": result.stdout,
        "stderr": result.stderr,
        "duration": result.duration_seconds,
    }


@app.post("/v1/git/commit")
async def commit_endpoint(request: CommitRequest):
    result = await SERVICE.commit(request)
    return {"status": "ok", **result}


async def send(ws: websockets.WebSocketClientProtocol, message: Dict[str, Any]) -> None:
    await ws.send(json.dumps(message))


async def handle_bus_message(ws: websockets.WebSocketClientProtocol, message: Dict[str, Any]):
    if message.get("type") != "request":
        return
    capability = message.get("capability")
    request_id = message.get("requestId")
    payload = message.get("payload") or {}
    try:
        if capability == "repo.analyze":
            request = AnalyzeRequest(**payload)
            summary = await SERVICE.analyse(request)
            response = {"summary": asdict(summary)}
        elif capability == "repo.patch":
            request = PatchRequest(**payload)
            changes = await SERVICE.apply_patch(request)
            response = {"status": "ok", "changes": changes}
        elif capability == "repo.tree":
            request = TreeRequest(**payload)
            entries = await SERVICE.list_tree(request)
            response = {"entries": [asdict(entry) for entry in entries]}
        elif capability == "repo.file.read":
            request = ReadFileRequest(**payload)
            response = await SERVICE.read_file(request)
        elif capability == "repo.file.write":
            request = WriteFileRequest(**payload)
            response = await SERVICE.write_file(request)
        elif capability == "repo.search":
            request = SearchRequest(**payload)
            response = await SERVICE.search(request)
        elif capability == "repo.command":
            request = CommandInvocationRequest(**payload)
            result = await SERVICE.run_command(request)
            response = {
                "command": result.command,
                "exitCode": result.exit_code,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "duration": result.duration_seconds,
            }
        elif capability == "repo.commit":
            request = CommitRequest(**payload)
            response = await SERVICE.commit(request)
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


async def telemetry(ws: websockets.WebSocketClientProtocol):
    if psutil is None:
        return
    process = psutil.Process(os.getpid())
    process.cpu_percent(interval=None)
    interval = max(5, SETTINGS.telemetry.sample_interval_seconds)
    while True:
        cpu_raw = process.cpu_percent(interval=None)
        cpu_count = psutil.cpu_count() or 1
        cpu = cpu_raw / cpu_count
        mem = process.memory_info().rss
        await send(
            ws,
            {
                "type": "telemetry",
                "plugin": PLUGIN_NAME,
                "cpu": cpu,
                "mem_bytes": mem,
            },
        )
        await asyncio.sleep(interval)


async def bus_loop(port: int):
    url = f"ws://127.0.0.1:{BUS_PORT}"
    while True:
        try:
            async with websockets.connect(url) as ws:
                register = {
                    "type": "register",
                    "plugin": PLUGIN_NAME,
                    "port": port,
                    "capabilities": [
                        "repo.analyze",
                        "repo.patch",
                        "repo.tree",
                        "repo.file.read",
                        "repo.file.write",
                        "repo.search",
                        "repo.command",
                        "repo.commit",
                    ],
                    "configSchema": CONFIG_SCHEMA,
                    "meta": {
                        "database": str(DATABASE_PATH),
                        "settings": SETTINGS.serialise(),
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
                telemetry_task = asyncio.create_task(telemetry(ws))
                async for payload in ws:
                    message = json.loads(payload)
                    await handle_bus_message(ws, message)
                hb_task.cancel()
                telemetry_task.cancel()
                with contextlib.suppress(asyncio.CancelledError):
                    await hb_task
                    await telemetry_task
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
