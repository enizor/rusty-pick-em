# Progress File

This file will track progress, keep TODO notes, and log the previous work.

# TODOs

## Backend

- [X] Setup Rocket: have an 'hello-world'

    Using the official tutorial, pretty easy to setup.

- [X] Serve the built frontend

    Based on https://thefullsnack.com/en/rust-for-the-web.html

- [ ] Setup Diesel: create a table 'game' and implement CRUD operations
    - [X] Setup Diesel

        Adapting the official [Getting Started](http://diesel.rs/guides/getting-started).

        Notes: a column without NOT NULL need to be an Option<>, the timestamp is converted to a chrono::DateTime<Utc>.
        It works because PSQL returns the UTC version of the timestamp.

    - [ ] Implement Create
    - [X] Implement Read

        Simple print. Showing the timestamp allows to check the behavior of the timestamps

    - [X] Implement Update

        update-games.rs will update the timestamp of the first game.

        The timezone handling can be checked easily.

    - [ ] Implement Destroy

- [X] Setup the API: allow theses CRUD operations to be accessed via Rocket
    API abandoned in favor of static templates

- [X] Define the SQL shema and create it

- [X] Define the API and document it
    No API

- [ ] Lots of improvements!

## Frontend

- [X] Create a Hello-world frontpage

    Created with vue-cli

- [ ] Create vue/components for the SQL models

Vue.js is abandoned in favor of Handlebars templating for ease of use and quicker development. The primary focus of the project is to learn Rust, and secondary only to train on the frontend
