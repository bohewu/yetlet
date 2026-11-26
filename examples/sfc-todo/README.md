# Todo with single-file components

Add a task, mark it done, reopen it, or delete it. Yetlet renders the list on
the server. Each successful form submission redirects to a fresh copy of the
list. JavaScript adds a character counter to the new-task field.

This sample uses the experimental `sfc` feature from the source checkout.
With the repository's Rust toolchain, run this command from the repository root.

```sh
cargo run -p yetlet-sfc-todo --locked
```

Open <http://127.0.0.1:3012/>. Use Ctrl+C to stop the server. Set
`YETLET_SFC_TODO_PORT` to choose a different loopback port, or set it to `0` to
use the port printed in the startup message.

## Follow the code

- [TodoForm.yetlet](src/TodoForm.yetlet) holds the new-task form, styles and
  optional character counter.
- [TodoItem.yetlet](src/TodoItem.yetlet) renders one item from typed props.
  The boolean `completed` prop controls its visible status.
- [build.rs](build.rs) names the two components. Cargo generates their Rust
  modules and assets. [main.rs](src/main.rs) passes props, renders the list,
  handles requests and mounts the generated assets.

The application selects `sfc` only in Cargo's build dependency. At runtime it
uses the normal Yetlet HTML and Web features. The document serves its CSS and
optional script from the same origin under Yetlet's default CSP. The item
component's empty script produces no asset request.

| Method | Path | Behavior |
| --- | --- | --- |
| GET | `/` | Render the add form and the task list. |
| POST | `/todos` | Validate and add a task. |
| POST | `/todos/{id}` | Mark done, reopen, or delete the selected task. |
| GET | `/assets/{name}` | Serve the page stylesheet and generated CSS and JavaScript. |

The sample keeps up to 100 shared tasks in memory and clears them on restart.
Every request must use the loopback host and port printed at startup.
Mutations also require the matching origin.

## Verify locally

```sh
cargo test --locked -p yetlet-sfc-todo
```

The native tests cover the task lifecycle, escaped titles, validation, and the
host and origin checks. To try the browser fallback, disable JavaScript, reload, and add,
complete, reopen, and delete a task with the form buttons. Re-enable JavaScript
and reload to see the counter while typing.
