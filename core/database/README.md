# core/database

Hier liegen zukünftig alle Migrationsskripte und Datenbankschemata für das bkg-Kernsystem. Die NestJS-Dienste greifen über den zentralen Pfad `process.env.BKG_DATABASE_PATH` auf das SQLite/PostgreSQL-Backend zu. Migrationen werden als SQL-Dateien oder über ein dediziertes Tool (z. B. Prisma, TypeORM) versioniert.
