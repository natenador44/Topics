create table topics (
    id uuid primary key,
    name varchar(255) not null,
    description varchar(4096),
    created timestamp with time zone default now(),
    updated timestamp with time zone
);
