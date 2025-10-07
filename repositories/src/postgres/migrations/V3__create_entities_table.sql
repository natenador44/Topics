CREATE TABLE IF NOT EXISTS entities (
    id uuid primary key,
    -- topic_id uuid, -- good for archiving the future?
    set_id uuid,
    payload jsonb not null,
    created timestamp with time zone default now(),
    updated timestamp with time zone default null,
    constraint set_id_fk foreign key (set_id) references sets(id)
);