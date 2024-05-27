-- Your SQL goes here
CREATE TABLE sketches(
  id SERIAL PRIMARY KEY,
  creator_id  INT NOT NULL,
  creation_time TIMESTAMP WITH TIME ZONE NOT NULL,
  FOREIGN KEY (creator_id) REFERENCES users(id)
)