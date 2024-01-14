CREATE TABLE device (
  id TEXT NOT NULL PRIMARY KEY,
  user_agent TEXT NOT NULL,
  name TEXT
);

CREATE TABLE subscription (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  data TEXT NOT NULL,
  device_id TEXT NOT NULL,
  FOREIGN KEY(device_id) REFERENCES device(id)
);
