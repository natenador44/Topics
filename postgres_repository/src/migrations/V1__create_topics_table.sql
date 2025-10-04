create table if not exists topics (
    id uuid primary key,
    name varchar(255) not null,
    description varchar(4096),
    name_tsv tsvector default null,
    description_tsv tsvector default null,
    created timestamp with time zone default now(),
    updated timestamp with time zone default null
);


create index if not exists topics_name_idx on topics using GIN (to_tsvector('english', name));
create index if not exists topics_description_idx on topics using GIN (to_tsvector('english', description));

CREATE or REPLACE FUNCTION update_topic_updated_timestamp() RETURNS TRIGGER
AS $$
BEGIN
        NEW.updated := now();
RETURN NEW;
end;
$$ LANGUAGE plpgsql;

drop trigger if exists topics_name_tsv_trigger on topics;
drop trigger if exists topics_name_trigger on topics;

create trigger topics_name_trigger before update on topics
    for each row
    execute function update_topic_updated_timestamp();

CREATE TRIGGER topics_name_tsv_trigger BEFORE UPDATE OR INSERT ON topics
    FOR EACH ROW
EXECUTE PROCEDURE tsvector_update_trigger(name_tsv, 'pg_catalog.english', name);

CREATE TRIGGER topics_description_tsv_trigger BEFORE UPDATE OR INSERT ON topics
    FOR EACH ROW
EXECUTE PROCEDURE tsvector_update_trigger(description_tsv, 'pg_catalog.english', description);
