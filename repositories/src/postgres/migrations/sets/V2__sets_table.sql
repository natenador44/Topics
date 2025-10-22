create table if not exists sets (
    id uuid primary key,
    topic_id uuid,
    name varchar(255) not null,
    description varchar(4096),
    created timestamp with time zone default now(),
    updated timestamp with time zone,
    constraint t_id_fk foreign key (topic_id) references topics (id) on delete cascade
);