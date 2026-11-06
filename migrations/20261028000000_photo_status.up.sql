ALTER TABLE photos ADD COLUMN status TEXT NOT NULL DEFAULT 'ready'
    CHECK (status IN ('uploading', 'ready', 'deleting'));
