# `todo`

## Setup `sqlx`

1. Declare the database URL

    ```
    export DATABASE_URL="sqlite:todos.db"
    ```

2. Create the database.

    ```
    $ sqlx db create
    ```

3. Run sql migrations

    ```
    $ sqlx migrate run
    ```
