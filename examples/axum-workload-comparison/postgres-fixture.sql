-- Run once as the disposable bootstrap owner, before the read-only app starts.
CREATE TABLE public.world (
    id integer PRIMARY KEY,
    random_number integer NOT NULL CHECK (random_number BETWEEN 1 AND 10000)
);
INSERT INTO public.world (id, random_number)
SELECT id, 1 + ((id * 7919) % 10000) FROM generate_series(1, 10000) AS id;
ANALYZE public.world;
REVOKE ALL ON DATABASE yt144 FROM PUBLIC;
REVOKE ALL ON SCHEMA public FROM PUBLIC;
GRANT CONNECT ON DATABASE yt144 TO yt144_app;
GRANT USAGE ON SCHEMA public TO yt144_app;
GRANT SELECT ON public.world TO yt144_app;
ALTER ROLE yt144_app SET default_transaction_read_only = on;
ALTER ROLE yt144_app SET statement_timeout = '750ms';
