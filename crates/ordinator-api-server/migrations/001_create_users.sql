CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,           -- UUID as text
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT,            -- Argon2 PHC string Can be null for external auth.
    role TEXT NOT NULL,                     -- 'Scheduler', 'Supervisor', 'Technician', 'Admin'
    assets TEXT NOT NULL DEFAULT '[]',      -- JSON array of asset IDs
    provider TEXT NOT NULL, -- 'Local', 'AzureAd', 'Google'
    is_active INTEGER NOT NULL DEFAULT 1,   -- SQLite boolean (0/1)
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    -- Ensure Local users have password, external users don't
    CHECK (
        (provider = 'Local' AND password_hash IS NOT NULL) OR
        (provider != 'Local' AND password_hash IS NULL)
    )
);

-- index for email lookups
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Supervisor-Technician relationship table
CREATE TABLE IF NOT EXISTS supervisor_technicians (
    supervisor_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    technician_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (supervisor_id, technician_id)
);

-- Index for looking up technicians by supervisor
CREATE INDEX IF NOT EXISTS idx_supervisor_technicians_supervisor ON supervisor_technicians(supervisor_id);

-- Index for looking up supervisor by technician
CREATE INDEX IF NOT EXISTS idx_supervisor_technicians_technician ON supervisor_technicians(technician_id);
