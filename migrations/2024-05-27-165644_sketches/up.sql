-- Your SQL goes here
CREATE TABLE sketches(
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  creator_id  INT NOT NULL,
  creation_time TIMESTAMP WITH TIME ZONE NOT NULL,
  DATA TEXT NOT NULL,
  FOREIGN KEY (creator_id) REFERENCES users(id)
)