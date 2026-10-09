BEGIN;

-- Transfer ownership of existing tables.
ALTER TABLE public.users OWNER TO garnet_migrator;
ALTER TABLE public.sessions OWNER TO garnet_migrator;
ALTER TABLE public.lists OWNER TO garnet_migrator;
ALTER TABLE public.items OWNER TO garnet_migrator;
ALTER TABLE public.members OWNER TO garnet_migrator;


-- Schema access.
GRANT USAGE ON SCHEMA public TO garnet_runtime;

-- CRUD permissions.
GRANT SELECT, INSERT, UPDATE, DELETE
ON ALL TABLES IN SCHEMA public
TO garnet_runtime;

-- Sequence permissions, if needed.
GRANT USAGE, SELECT
ON ALL SEQUENCES IN SCHEMA public
TO garnet_runtime;

-- Ensure future objects created by garnet_migrator are accessible.
ALTER DEFAULT PRIVILEGES FOR ROLE garnet_migrator IN SCHEMA public
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO garnet_runtime;

ALTER DEFAULT PRIVILEGES FOR ROLE garnet_migrator IN SCHEMA public
GRANT USAGE, SELECT ON SEQUENCES TO garnet_runtime;

COMMIT;
