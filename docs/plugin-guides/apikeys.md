# APIKeys Plug-in

## Überblick
Das APIKeys-Plug-in verwaltet Benutzer, Rollen, JWT-Sitzungen und API-Schlüssel. Es ist die sicherheitskritische Schicht für Authentifizierung und Autorisierung innerhalb von bkg.rs.

## Fähigkeiten
- `keys.issue` – API-Schlüssel ausstellen.
- `keys.rotate` – Schlüsselrotation durchführen und Historie pflegen.
- `keys.audit` – Audit-Trail und Zugriffshistorie bereitstellen.

## Admin-UI
- Route: `/plugins/apikeys`
- Lebenszyklus: Start, Stop, Restart, Refresh
- Featurekarten (4): Benutzerverwaltung, Schlüssel-Lifecycle, Scope-Prüfung, Audit-Events
- Konfigurationseditor: Passwort-Richtlinien, Token-TTLs und Scope-Mappings
- Logstream: Security-Events, Fehlversuche und Rotationsmeldungen

## Abhängigkeiten
- Datenbank: SQLite `/data/bkg.db`
- Kryptographie: bcrypt, libsodium (über Node-Bindings)
- Integration: Gateway-Guards (`Authorization: Bearer`)

## Betriebshinweise
- Admin-Passwörter bei Deployment via `ADMIN_PASSWORD` setzen und sofort rotieren.
- Scopes regelmäßig auditieren und ungenutzte Schlüssel sperren.
- Audit-Logs exportieren, um Compliance-Anforderungen (z. B. ISO 27001) zu erfüllen.
