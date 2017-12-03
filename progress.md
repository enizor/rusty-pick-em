# Progress File

This file will track progress, keep TODO notes, and log the previous work.

# TODOs

## Backend

- [X] Setup Rocket: have an 'hello-world'

    Using the official tutorial, pretty easy to setup.

- [X] Serve the built frontend

    Based on https://thefullsnack.com/en/rust-for-the-web.html

- [ ] Setup Diesel: create a table 'game' and implement CRUD operations
    - [X] Setup Diesel

        Adapting the official [Getting Started](http://diesel.rs/guides/getting-started).

        Notes: a column without NOT NULL need to be an Option<>, the timestammp is converted to a chrono::DateTime<Utc>. It works because PSQL returns the UTC version of the timestamp

    - [ ] Implement Create
    - [X] Implement Read

        Simple print. Showing the timestamp allows to check the behavior of the timestamps
    - [ ] Implement Update
    - [ ] Implement Destroy

- [ ] Setup the API: allow theses CRUD operations to be accessed via Rocket

- [ ] Define the SQL shema and create it

- [ ] Define the API and document it

- [ ] More to come

## Frontend

- [X] Create a Hello-world frontpage

    Created with vue-cli

- [ ] Create vue/components for the SQL models
