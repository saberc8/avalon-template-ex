-- Add role status so disabled roles cannot grant authorization state.

ALTER TABLE sys_role
    ADD COLUMN IF NOT EXISTS status SMALLINT NOT NULL DEFAULT 1;

CREATE INDEX IF NOT EXISTS idx_role_status ON sys_role (status);
