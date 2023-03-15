-- This should be an actual migration system but the ScyllaDB driver doesn't support that currently.
-- Warning: Running this deletes all existing data.

CREATE KEYSPACE volksforo WITH replication = {'class': 'SimpleStrategy', 'replication_factor' : 1};
USE volksforo;


--
-- Nodes
--
DROP TABLE IF EXISTS nodes;
CREATE TABLE nodes (
    id bigint,
    display_order int,
    title text,
    description text,
    PRIMARY KEY (id, display_order)
) WITH CLUSTERING ORDER BY (display_order ASC);

INSERT INTO nodes (id, display_order, title, description) VALUES (1, 20, 'Chuck''s Fuck and Suck', '18+ only.');
INSERT INTO nodes (id, display_order, title, description) VALUES (2, 10, 'Sneed''s Feed and Seed', 'Formerly Chuck''s.');

--
-- Threads
--
DROP TABLE IF EXISTS threads;
CREATE TABLE threads (
    id bigint,
    node_id bigint,
    bucket_id int,
    title text,
    subtitle text,
    first_post_id bigint,
    first_post_user_id bigint,
    last_post_id bigint,
    last_post_user_id bigint,
    PRIMARY KEY (node_id, bucket_id, last_post_id)
) WITH CLUSTERING ORDER BY (bucket_id ASC, last_post_id ASC);

-- Indexes are tricky.
-- https://docs.scylladb.com/stable/using-scylla/secondary-indexes.html
-- We want to PK node_id,bucket_id,last_post_id so we can do pagination easier.
DROP INDEX IF EXISTS threads_by_id;
CREATE INDEX threads_by_id ON volksforo.threads (id);

INSERT INTO threads (id, node_id, bucket_id, title, first_post_id, first_post_user_id, last_post_id, last_post_user_id) VALUES (1, 1, 1, 'Test Thread', 1, 1, 7, 1);
INSERT INTO threads (id, node_id, bucket_id, title, first_post_id, first_post_user_id, last_post_id, last_post_user_id) VALUES (2, 1, 1, 'Other Thread', 4, 420, 4, 420);
INSERT INTO threads (id, node_id, bucket_id, title, first_post_id, first_post_user_id, last_post_id, last_post_user_id) VALUES (3, 2, 1, 'Chuck Thread', 5, 69, 5, 69);

--
-- Posts
--
DROP TABLE IF EXISTS posts;
CREATE TABLE posts (
    id bigint,
    thread_id bigint,
    position int,
    user_id bigint,
    ugc_id uuid,
    PRIMARY KEY ((thread_id, position), id)
) WITH CLUSTERING ORDER BY (id ASC); -- threads order newest to oldest


INSERT INTO posts (id, thread_id, position, user_id, ugc_id) VALUES (1, 1, 1, 1, 9d1fe4ff-00a4-418f-8234-8ee2208f85eb);
INSERT INTO posts (id, thread_id, position, user_id, ugc_id) VALUES (2, 1, 2, 69, 077d372c-8836-44e4-a75d-7f119a5ac195);
INSERT INTO posts (id, thread_id, position, user_id, ugc_id) VALUES (3, 1, 3, 420, 90d07d83-1736-491d-872f-9fce4d5250a9);
INSERT INTO posts (id, thread_id, position, user_id, ugc_id) VALUES (4, 2, 1, 420, 8fabdde1-1ccb-42ab-8cd3-70c91fe571c6);
INSERT INTO posts (id, thread_id, position, user_id, ugc_id) VALUES (5, 3, 1, 69, 0c287743-199f-4160-95b0-4992785b62a2);
INSERT INTO posts (id, thread_id, position, user_id, ugc_id) VALUES (6, 1, 4, 69, 0ec2e499-356f-465a-bb37-ad9904f29122); -- duplicate position
INSERT INTO posts (id, thread_id, position, user_id, ugc_id) VALUES (7, 1, 4, 1, cfc00480-3ae0-4af4-ab5e-542414c9c968);

--
-- UGC
--
DROP TABLE IF EXISTS ugc;
CREATE TABLE ugc (
    id uuid,
    ip_id uuid,
    user_id bigint,
    created_at timestamp,
    content text,
    PRIMARY KEY (id, created_at)
) WITH CLUSTERING ORDER BY (created_at DESC);

INSERT INTO ugc (id, created_at, content) VALUES (9d1fe4ff-00a4-418f-8234-8ee2208f85eb, '2023-03-12T14:27:00+00:00', 'First post');
INSERT INTO ugc (id, created_at, content) VALUES (077d372c-8836-44e4-a75d-7f119a5ac195, '2023-03-12T14:27:01+00:00', 'Second post');
INSERT INTO ugc (id, created_at, content) VALUES (90d07d83-1736-491d-872f-9fce4d5250a9, '2023-03-12T14:27:02+00:00', 'Third post');
INSERT INTO ugc (id, created_at, content) VALUES (8fabdde1-1ccb-42ab-8cd3-70c91fe571c6, '2023-03-12T14:27:03+00:00', 'Fourth post');
INSERT INTO ugc (id, created_at, content) VALUES (0c287743-199f-4160-95b0-4992785b62a2, '2023-03-12T14:27:04+00:00', 'Fifth post');
INSERT INTO ugc (id, created_at, content) VALUES (0ec2e499-356f-465a-bb37-ad9904f29122, '2023-03-12T14:27:05+00:00', 'Sixth post');
INSERT INTO ugc (id, created_at, content) VALUES (cfc00480-3ae0-4af4-ab5e-542414c9c968, '2023-03-12T14:27:06+00:00', 'Sixth post');
INSERT INTO ugc (id, created_at, content) VALUES (cfc00480-3ae0-4af4-ab5e-542414c9c968, '2023-03-12T14:27:07+00:00', 'Seventh* post, sorry');

--
-- User
--
DROP TABLE IF EXISTS users;
CREATE TABLE users (
    id bigint,
    username text,
    PRIMARY KEY (id)
);

INSERT INTO users (id, username) VALUES (1, 'admin');
INSERT INTO users (id, username) VALUES (69, 'Sneed');
INSERT INTO users (id, username) VALUES (420, 'Chuck');