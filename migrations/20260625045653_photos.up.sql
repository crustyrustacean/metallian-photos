-- Add up migration script here
CREATE TABLE photos(
    id TEXT PRIMARY KEY,
    /* band, tour, and venue corresponds to the `band`, `tour`, and `venue` fields in the domain `Photo` type,
    which are `String` types and must exist, the NOT NULL is the translation to SQL */
    band TEXT NOT NULL,
    tour TEXT NOT NULL,
    venue TEXT NOT NULL,
    /* date_time_original, make, model, lens_make, and lens_model corresponds to the `date_time_original`, `make`, `model`,
    `lens_make` and `lens_model` fields in the domain `Photo` type,
    which are `Option<String>` types and may be omitted */
    date_time_original TEXT,
    make TEXT,
    model TEXT,
    lens_make TEXT,
    lens_model TEXT
);
