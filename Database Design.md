# Tables
## `topics`

### Relational

| Column      | Type         | Notes       |
| ----------- | ------------ | ----------- |
| ID          | UUID_v7      | Primary Key |
| Name        | text\[128\]  |             |
| Description | text\[1024\] |             |
## `identifiers`

### Relational
Nothing, these will just be collections in the non-relational db
### Non-relational
Collections named after `topics::id` if possible, or `topics:name` if not. Each collection will contain documents that looks like this.
```json
{
	"id": "some-id",
	"expression": "<identifier expression>"
}
```
where `<identifier expression>` is more document description how the identifier works.
## sets

### Relational

| Column      | Name         | Notes         |
| ----------- | ------------ | ------------- |
| set_id      | UUID_v7      | Primary Key   |
| entity_id   | UUID_v7      | Secondary Key |
| topic_id    | UUID_v7      | Secondary Key |
| name        | text\[128\]  |               |
| description | text\[1024\] |               |
### Non-relational
Collections named after `sets:id` or `sets:name`. Each document will look like this.
```json
{
	"id": "some-id",
	"payload": "<more document>",
}
```
# Database Technology
## Relational or Document or Key/Value store?
Document is tempting due to the unknown nature of the `sets:payload` and `identifiers:expression` columns. There are some relational bits to this that give me pause.

I could do this..
## Relational Bits
Have the `topics`, `identifiers`, and `sets` "metadata" live in a relational database. Things like id, name, and description plus any foreign keys to describe a relationship could be kept here.
## Non-relational Bits
The `identifiers:expression` and `sets:payload` could live in a document database, where the collection each 'row' lives in has the name of its respective id.
So `identifiers:expression` would have a collection per topic, where the collection name is "`topic_id`". `sets:payload` would have collections per set, with the collection name of "`set_id`".

## Why?
I think I get the best of both worlds here. The "metadata" will grow a lot slower than the other data. The other data also has the potential to grow quite a bit, particularly set entities. Using a non-relational database makes it easier to scale that data.
I can also see queries being separate - you're either seeing a high level view, in which case you'd just be hitting the relational database, or you're doing a deep dive to view individual documents. In that case you'd most likely have access to the ID or name and can hit the non-relational collection directly without having to do additional queries.