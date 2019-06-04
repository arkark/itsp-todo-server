-- Your SQL goes here

CREATE TABLE tasks (
  id bigserial not null primary key,
  deadline timestamp not null,
  title varchar not null,
  memo varchar not null
);
