CREATE TABLE public.accounts (
    id integer PRIMARY KEY,
    tenant text NOT NULL,
    enabled boolean NOT NULL,
    can_view boolean NOT NULL
);
CREATE TABLE public.documents (
    id integer PRIMARY KEY,
    tenant text NOT NULL,
    title text NOT NULL,
    body text NOT NULL
);
INSERT INTO public.accounts VALUES (1, 'alpha', true, true), (2, 'beta', true, true);
INSERT INTO public.documents
SELECT i, 'alpha', 'Report ' || i || ' <&>', 'Private <&> "quoted" ''café'' ' || i
FROM generate_series(1, 8) AS i;
INSERT INTO public.documents
SELECT 100 + i, 'beta', 'Other tenant ' || i, 'Other tenant private data'
FROM generate_series(1, 8) AS i;
REVOKE ALL ON SCHEMA public FROM PUBLIC;
GRANT USAGE ON SCHEMA public TO yt170_app;
GRANT SELECT ON public.accounts, public.documents TO yt170_app;
ALTER ROLE yt170_app SET default_transaction_read_only = on;
ALTER ROLE yt170_app SET statement_timeout = '750ms';
ANALYZE public.accounts;
ANALYZE public.documents;
