-- migrations/__TIMESTAMP___create___features___table.sql
CREATE TABLE __features__ (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx___features___created_at ON __features__ (created_at DESC);
