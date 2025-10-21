# Operations

## Health Checks

- `GET /health` – overall service health (aggregates plug-in status).
- `GET /admin/plugins` – detailed plug-in runtime state.
- Plug-in log streams (`/admin/plugins/:name/logs`) provide live diagnostics in the UI.

The Angular Admin tab exposes the same information visually and highlights degraded plug-ins.

## Authentication Workflow

1. Obtain an initial session by POSTing to `/auth/login` with admin credentials (`ADMIN_PASSWORD`).
2. The apikeys plug-in issues a 24-hour session token used as a Bearer token for API requests.
3. Administrators can mint long-lived API keys with scoped permissions via the Admin UI or `POST /admin/keys`.
4. `auth.validate` is invoked on every guard-protected endpoint to enforce scope requirements.

## Plug-in Lifecycle

- **Start/Stop** – Triggered via `POST /admin/plugins/:name/start` and `/stop` or the Plugins UI tab.
- **Restart** – Equivalent to stop followed by start; future iterations can expose an explicit endpoint.
- **Configuration** – Submit updated JSON via `POST /admin/plugins/:name/config`; persisted to SQLite and mirrored to `plugins/plugins.json`.
- **Logs** – Streams aggregated logs via SSE; use the Plugins UI or curl with `Accept: text/event-stream`.

## Backup Strategy

The only persistent artefact is `/data/bkg.db`. Regularly snapshot this file to protect users, API keys, and plug-in configuration. Ensure backups are encrypted and rotated per organisational policy.

## Security Considerations

- Always provide a strong `ADMIN_PASSWORD`. Changing the password requires deleting or updating the `users` table entry.
- API keys are bcrypt-hashed at rest; loss of the plaintext key requires issuing a new one.
- Network exposure should be fronted by TLS termination (handled externally) to secure API/WebSocket traffic.
- Plug-ins run as subprocesses; ensure model files and repository paths are mounted with least privilege.

## Incident Response

1. Review `/health` and `/admin/plugins` to identify failing components.
2. Inspect plug-in logs via the Plugins tab to pinpoint errors.
3. Restart affected plug-ins using the admin controls.
4. If authentication fails, verify the apikeys plug-in is running and the SQLite database is writable.
5. For systemic issues, stop the API gracefully – it will propagate shutdown to plug-in processes.
