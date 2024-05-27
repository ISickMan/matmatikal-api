-- Your SQL goes here
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  username VARCHAR NOT NULL UNIQUE,
  email VARCHAR NOT NULL UNIQUE,
  google_id VARCHAR,
  pass_hash VARCHAR NOT NULL,
  birthday DATE NOT NULL,
  grade SMALLINT NOT NULL,
  creation_time TIMESTAMP WITH TIME ZONE NOT NULL
)