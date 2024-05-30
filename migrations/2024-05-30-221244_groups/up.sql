-- Your SQL goes here
CREATE TABLE sketch_groups(
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  creator_id  INT NOT NULL,
  FOREIGN KEY (creator_id) REFERENCES users(id)
)