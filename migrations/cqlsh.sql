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
    created_at timestamp,
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

INSERT INTO threads (id, node_id, bucket_id, title, created_at, first_post_id, first_post_user_id, last_post_id, last_post_user_id) VALUES (1, 1, 1, 'Test Thread', '2023-03-12T14:27:00+00:00', 1, 1, 7, 1);
INSERT INTO threads (id, node_id, bucket_id, title, created_at, first_post_id, first_post_user_id, last_post_id, last_post_user_id) VALUES (2, 1, 1, 'Other Thread', '2023-03-12T14:27:03+00:00', 4, 420, 4, 420);
INSERT INTO threads (id, node_id, bucket_id, title, created_at, first_post_id, first_post_user_id, last_post_id, last_post_user_id) VALUES (3, 2, 1, 'Chuck Thread', '2023-03-12T14:27:04+00:00', 5, 69, 5, 69);

-- Thread view counter table
-- These are special in Scylla.
-- https://docs.scylladb.com/stable/using-scylla/counters.html
DROP TABLE IF EXISTS thread_views;
CREATE TABLE thread_views (
    id bigint PRIMARY KEY,
    view_count counter
);

DROP TABLE IF EXISTS thread_replies;
CREATE TABLE thread_replies (
    id bigint PRIMARY KEY,
    reply_count counter
);

-- No inserts, only updates.
UPDATE thread_replies SET reply_count = reply_count + 4 WHERE id = 1;
UPDATE thread_replies SET reply_count = reply_count + 1 WHERE id = 2;
UPDATE thread_replies SET reply_count = reply_count + 1 WHERE id = 3;

--
-- Posts
--
DROP TABLE IF EXISTS posts;
CREATE TABLE posts (
    id bigint,
    thread_id bigint,
    created_at timestamp,
    user_id bigint,
    ugc_id uuid,
    PRIMARY KEY (id)
); -- threads order newest to oldest


INSERT INTO posts (id, thread_id, user_id, created_at, ugc_id) VALUES (1, 1, 1, '2023-03-12T14:27:00+00:00', 9d1fe4ff-00a4-418f-8234-8ee2208f85eb);
INSERT INTO posts (id, thread_id, user_id, created_at, ugc_id) VALUES (2, 1, 69, '2023-03-12T14:27:01+00:00', 077d372c-8836-44e4-a75d-7f119a5ac195);
INSERT INTO posts (id, thread_id, user_id, created_at, ugc_id) VALUES (3, 1, 420, '2023-03-12T14:27:02+00:00', 90d07d83-1736-491d-872f-9fce4d5250a9);
INSERT INTO posts (id, thread_id, user_id, created_at, ugc_id) VALUES (4, 2, 420, '2023-03-12T14:27:03+00:00', 8fabdde1-1ccb-42ab-8cd3-70c91fe571c6);
INSERT INTO posts (id, thread_id, user_id, created_at, ugc_id) VALUES (5, 3, 69, '2023-03-12T14:27:04+00:00', 0c287743-199f-4160-95b0-4992785b62a2);
INSERT INTO posts (id, thread_id, user_id, created_at, ugc_id) VALUES (6, 1, 69, '2023-03-12T14:27:05+00:00', 0ec2e499-356f-465a-bb37-ad9904f29122); -- duplicate position
INSERT INTO posts (id, thread_id, user_id, created_at, ugc_id) VALUES (7, 1, 1, '2023-03-12T14:27:06+00:00', cfc00480-3ae0-4af4-ab5e-542414c9c968);

--
-- Post Position
--
DROP TABLE IF EXISTS post_positions;
CREATE TABLE post_positions (
    thread_id bigint,
    position bigint,
    post_id bigint,
    PRIMARY KEY (thread_id, position, post_id)
) WITH CLUSTERING ORDER BY (position ASC, post_id ASC);

INSERT INTO post_positions (thread_id, position, post_id) VALUES (1, 1, 1);
INSERT INTO post_positions (thread_id, position, post_id) VALUES (1, 2, 2);
INSERT INTO post_positions (thread_id, position, post_id) VALUES (1, 3, 3);
INSERT INTO post_positions (thread_id, position, post_id) VALUES (2, 1, 4);
INSERT INTO post_positions (thread_id, position, post_id) VALUES (3, 1, 5);
INSERT INTO post_positions (thread_id, position, post_id) VALUES (1, 4, 6); -- duplicate position
INSERT INTO post_positions (thread_id, position, post_id) VALUES (1, 4, 7);

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

INSERT INTO ugc (id, created_at, content) VALUES (9d1fe4ff-00a4-418f-8234-8ee2208f85eb, 1, '2023-03-12T14:27:00+00:00', 'First post');
INSERT INTO ugc (id, created_at, content) VALUES (077d372c-8836-44e4-a75d-7f119a5ac195, 69, '2023-03-12T14:27:01+00:00', 'Second post');
INSERT INTO ugc (id, created_at, content) VALUES (90d07d83-1736-491d-872f-9fce4d5250a9, 420, '2023-03-12T14:27:02+00:00', 'Third post');
INSERT INTO ugc (id, created_at, content) VALUES (8fabdde1-1ccb-42ab-8cd3-70c91fe571c6, 420, '2023-03-12T14:27:03+00:00', 'Fourth post');
INSERT INTO ugc (id, created_at, content) VALUES (0c287743-199f-4160-95b0-4992785b62a2, 69, '2023-03-12T14:27:04+00:00', 'Fifth post');
INSERT INTO ugc (id, created_at, content) VALUES (0ec2e499-356f-465a-bb37-ad9904f29122, 69, '2023-03-12T14:27:05+00:00', 'Sixth post');
INSERT INTO ugc (id, created_at, content) VALUES (cfc00480-3ae0-4af4-ab5e-542414c9c968, 1, '2023-03-12T14:27:06+00:00', 'Sixth post');
INSERT INTO ugc (id, created_at, content) VALUES (cfc00480-3ae0-4af4-ab5e-542414c9c968, 1, '2023-03-12T14:27:07+00:00', 'Seventh* post, sorry'); -- edited post

--
-- User
--
DROP TABLE IF EXISTS users;
CREATE TABLE users (
    id bigint,
    username text,
    username_normal text,
    email text,
    password text,
    password_cipher text,
    PRIMARY KEY (id)
);

DROP INDEX IF EXISTS users_by_name_normal;
CREATE INDEX users_by_name_normal ON volksforo.users (username_normal);

INSERT INTO users (id, username, username_normal, password, password_cipher) VALUES (1, 'admin', 'admin', 'password', 'plaintext');
INSERT INTO users (id, username, username_normal, password, password_cipher) VALUES (69, 'Sneed', 'sneed', 'password', 'plaintext');
INSERT INTO users (id, username, username_normal, password, password_cipher) VALUES (420, 'Chuck', 'chuck', 'password', 'plaintext');

--
-- User Sessions
--
DROP TABLE IF EXISTS user_sessions;
CREATE TABLE user_sessions (
    id uuid,
    user_id bigint,
    created_at timestamp,
    last_seen_at timestamp,
    PRIMARY KEY(id)
);

DROP INDEX IF EXISTS user_sessions_by_user_id;
CREATE INDEX user_sessions_by_user_id ON volksforo.user_sessions (user_id);
