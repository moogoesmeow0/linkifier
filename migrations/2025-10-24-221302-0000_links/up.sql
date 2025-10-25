CREATE TABLE IF NOT EXISTS "links" (
    "link" varchar(512) PRIMARY KEY NOT NULL UNIQUE,
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "redirect" varchar(512) NOT NULL
);
