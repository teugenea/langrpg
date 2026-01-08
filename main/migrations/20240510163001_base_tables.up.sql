CREATE TABLE IF NOT EXISTS world(
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar(255)
);

CREATE TABLE IF NOT EXISTS lvl(
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    world uuid not null references world(id),
    file_name varchar(255)
);

CREATE TABLE IF NOT EXISTS game(
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar(255),
    active boolean,
    stateful boolean,
    world uuid references world(id)
);

CREATE TABLE IF NOT EXISTS item_type(
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar(255) unique not null
);
