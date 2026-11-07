-- the kind of device (mobile, tablet, desktop), the browser and the os
-- from the user agent, which itself isn't stored; null for the events
-- from before and imported ones
ALTER TABLE events ADD COLUMN device TEXT NULL;
ALTER TABLE events ADD COLUMN browser TEXT NULL;
ALTER TABLE events ADD COLUMN os TEXT NULL;
